use defmt::info;
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select, select3};
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::rng::Rng;
use esp_println as _;
use esp_radio::wifi::{
    AccessPointConfig, ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice, WifiEvent,
    WifiStaState, sta_state,
};
use esp_storage::FlashStorage;
use heapless::String;
use logic::{
    config::SERVER_IP,
    wifi::{LoginData, WifiManager, simple::SimpleWifiManager},
};
use sequential_storage::cache::NoCache;

use crate::{
    mk_static,
    shared_flash::SharedFlashInterface,
    storage::CredentialStorage,
    web::services::captive_ssids::{MAX_SSID_COUNT, SsidsList},
};

const SSID: &str = "Cresco";

pub async fn start_wifi(
    radio_init: &'static esp_radio::Controller<'static>,
    wifi: esp_hal::peripherals::WIFI<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> (WifiController<'static>, Stack<'static>, Stack<'static>) {
    let (wifi_controller, interfaces) = esp_radio::wifi::new(radio_init, wifi, Default::default())
        .expect("Failed to initialize Wi-Fi controller");

    let ip_addr = Ipv4Cidr::new(SERVER_IP, 24);
    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: ip_addr,
        gateway: Some(SERVER_IP),
        dns_servers: Default::default(),
    });
    let (ap_stack, ap_runner) = embassy_net::new(
        interfaces.ap,
        net_config,
        mk_static!(StackResources<6>, StackResources::<6>::new()),
        rng.random() as u64 | ((rng.random() as u64) << 32),
    );

    let (sta_stack, sta_runner) = embassy_net::new(
        interfaces.sta,
        embassy_net::Config::dhcpv4(Default::default()),
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        rng.random() as u64 | ((rng.random() as u64) << 32),
    );

    spawner.spawn(net_task(ap_runner)).ok();
    spawner.spawn(net_task(sta_runner)).ok();

    (wifi_controller, ap_stack, sta_stack)
}

pub type ConcreteCredentialStorage<'a> =
    CredentialStorage<SharedFlashInterface<'a, BlockingAsync<FlashStorage<'static>>>, NoCache>;

#[embassy_executor::task]
pub async fn connection(
    mut controller: WifiController<'static>,
    ssids: &'static Mutex<CriticalSectionRawMutex, SsidsList>,
    credentials: &'static Signal<CriticalSectionRawMutex, LoginData>,
    mut credential_storage: ConcreteCredentialStorage<'static>,
) {
    info!("connection task started");
    let mut c = SimpleWifiManager::new();
    let mut event: Option<logic::wifi::WifiEvent> = Some(logic::wifi::WifiEvent::Start);
    loop {
        match c.handle_event(event.take().expect("no event: this should not happen")) {
            logic::wifi::WifiAction::RetrieveCredentials => {
                info!("retrieving credentials");
                event = match credential_storage.get_credentials().await {
                    Ok(credentials) => {
                        info!("found credentials");
                        Some(logic::wifi::WifiEvent::CredentialsFound { credentials })
                    }
                    Err(_) => {
                        info!("did not find credentials");
                        Some(logic::wifi::WifiEvent::CredentialsMissing)
                    }
                };
            }
            logic::wifi::WifiAction::WaitForCredentials => {
                info!("starting ap sta");
                let _ = controller.stop_async().await;
                let config = ModeConfig::ApSta(
                    ClientConfig::default(),
                    AccessPointConfig::default().with_ssid(SSID.into()),
                );
                let _ = controller.set_config(&config);
                let _ = controller.start_async().await;
                let deadline = Instant::now() + Duration::from_secs(60);
                loop {
                    let timeout = Timer::at(deadline);
                    match select3(credentials.wait(), Timer::after_secs(14), timeout).await {
                        Either3::First(cred) => {
                            info!("received credentials");
                            event = Some(logic::wifi::WifiEvent::CredentialsReceived {
                                credentials: cred,
                            });
                            break;
                        }
                        Either3::Second(_) => {
                            if let Ok(networks) = controller
                                .scan_with_config_async(ScanConfig::default().with_max(20))
                                .await
                            {
                                let mut list = ssids.lock().await;
                                list.clear();
                                let mut distinct_ssid_count = 0;
                                for ap in networks {
                                    if let Ok(s) = String::try_from(ap.ssid.as_str())
                                        && distinct_ssid_count < MAX_SSID_COUNT
                                    {
                                        if !list.contains(&s) {
                                            let _ = list.push(s);
                                            distinct_ssid_count += 1;
                                        }
                                    }
                                }
                                info!("found {} networks", list.len());
                            }
                        }
                        Either3::Third(_) => {
                            info!("retrying to get credentials");
                            event = Some(logic::wifi::WifiEvent::Timeout);
                            break;
                        }
                    }
                }
            }
            logic::wifi::WifiAction::EstablishConnection { credentials } => {
                info!("starting client");
                let LoginData { ssid, password } = credentials.clone();
                let _ = controller.stop_async().await;
                let config = ModeConfig::Client(
                    ClientConfig::default()
                        .with_ssid(ssid.as_str().into())
                        .with_password(password.as_str().into()),
                );
                let _ = controller.set_config(&config);
                let _ = controller.start_async().await;
                select(controller.connect_async(), Timer::after_secs(5)).await;
                match sta_state() {
                    WifiStaState::Connected => {
                        info!("connected");
                        if !credential_storage.set_credentials(credentials).await {
                            info!("failed to store credentials");
                        }
                        event = Some(logic::wifi::WifiEvent::ConnectionEstablished);
                    }
                    _ => {
                        info!("failed client connection");
                        event = Some(logic::wifi::WifiEvent::ConnectionNotEstablished);
                        Timer::after_secs(3).await;
                    }
                }
            }
            logic::wifi::WifiAction::WaitForDisconnect => {
                info!("waiting for disconnect");
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                event = Some(logic::wifi::WifiEvent::Disconnected);
                info!("disconnect");
            }
            logic::wifi::WifiAction::Ignore => {
                info!("something went wrong")
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}

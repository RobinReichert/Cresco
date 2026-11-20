use defmt::{debug, info, trace, warn};
use embassy_executor::Spawner;
use embassy_futures::select::{Either3, select, select3};
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::rng::Rng;
use esp_radio::wifi::{
    AccessPointConfig, ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice, WifiEvent,
    WifiStaState,
    event::{self, EventExt},
    sta_state,
};
use heapless::String;
use logic::{
    config::SERVER_IP,
    wifi::{LoginData, WifiManager, simple::SimpleWifiManager},
};

use crate::{
    mk_static,
    storage::ConcreteStorage,
    web::services::captive_ssids::{MAX_SSID_COUNT, SsidsList},
};

const SSID: &str = "Cresco";

pub async fn start_wifi(
    radio_init: &'static esp_radio::Controller<'static>,
    wifi: esp_hal::peripherals::WIFI<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> (WifiController<'static>, Stack<'static>, Stack<'static>) {
    debug!("wifi: initialize wifi controller");
    let (wifi_controller, interfaces) = esp_radio::wifi::new(radio_init, wifi, Default::default())
        .expect("wifi: initialize Wi-Fi controller failed");

    event::StaDisconnected::update_handler(|event| {
        warn!("wifi: station disconnected: {}", event.reason());
    });

    let ip_addr = Ipv4Cidr::new(SERVER_IP, 24);
    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: ip_addr,
        gateway: Some(SERVER_IP),
        dns_servers: Default::default(),
    });
    let (ap_stack, ap_runner) = embassy_net::new(
        interfaces.ap,
        net_config,
        mk_static!(StackResources<4>, StackResources::<4>::new()),
        rng.random() as u64 | ((rng.random() as u64) << 32),
    );

    let (sta_stack, sta_runner) = embassy_net::new(
        interfaces.sta,
        embassy_net::Config::dhcpv4(Default::default()),
        mk_static!(StackResources<6>, StackResources::<6>::new()),
        rng.random() as u64 | ((rng.random() as u64) << 32),
    );

    spawner.spawn(net_task(ap_runner)).ok();
    spawner.spawn(net_task(sta_runner)).ok();

    (wifi_controller, ap_stack, sta_stack)
}

#[embassy_executor::task]
pub async fn connection(
    mut controller: WifiController<'static>,
    ssids: &'static Mutex<CriticalSectionRawMutex, SsidsList>,
    credentials: &'static Signal<CriticalSectionRawMutex, LoginData>,
    storage: &'static ConcreteStorage,
) {
    info!("wifi connection: start task");
    let mut c = SimpleWifiManager::new();
    let mut event: Option<logic::wifi::WifiEvent> = Some(logic::wifi::WifiEvent::Start);
    loop {
        match c.handle_event(
            event
                .take()
                .expect("wifi connection: no event: this should not happen"),
        ) {
            logic::wifi::WifiAction::RetrieveCredentials => {
                debug!("wifi connection: retrieve credentials");
                event = match storage.get_credentials().await {
                    Ok(credentials) => {
                        debug!("wifi connection: retrieve credentials successfull");
                        Some(logic::wifi::WifiEvent::CredentialsFound { credentials })
                    }
                    Err(e) => {
                        debug!("wifi connection: retrieve credentials failed: {}", e);
                        Some(logic::wifi::WifiEvent::CredentialsMissing)
                    }
                };
            }
            logic::wifi::WifiAction::WaitForCredentials => {
                info!("wifi connection: start ap sta");
                let _ = controller.stop_async().await;
                let config = ModeConfig::ApSta(
                    ClientConfig::default(),
                    AccessPointConfig::default().with_ssid(SSID.into()),
                );
                let _ = controller.set_config(&config);
                let _ = controller.start_async().await;
                let deadline = Instant::now() + Duration::from_secs(60);
                loop {
                    debug!("wifi connection: wait for credentials");
                    let timeout = Timer::at(deadline);
                    match select3(credentials.wait(), Timer::after_secs(14), timeout).await {
                        Either3::First(cred) => {
                            info!("wifi connection: received credentials");
                            event = Some(logic::wifi::WifiEvent::CredentialsReceived {
                                credentials: cred,
                            });
                            break;
                        }
                        Either3::Second(_) => {
                            debug!("wifi connection: scan available networks");
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
                                        trace!("wifi connection: found network: {}", s);
                                        if !list.contains(&s) {
                                            let _ = list.push(s);
                                            distinct_ssid_count += 1;
                                        }
                                    }
                                }
                                debug!("wifi network: found {} networks", list.len());
                            }
                        }
                        Either3::Third(_) => {
                            debug!("wifi network: retry to get credentials");
                            event = Some(logic::wifi::WifiEvent::Timeout);
                            break;
                        }
                    }
                }
            }
            logic::wifi::WifiAction::EstablishConnection { credentials } => {
                debug!("wifi connection: start client");
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
                        info!("wifi connection: connect successfull");
                        debug!("wifi connection: store credentials");
                        if !storage.set_credentials(credentials).await {
                            warn!("wifi connection: store credentials failed");
                        }
                        event = Some(logic::wifi::WifiEvent::ConnectionEstablished);
                    }
                    _ => {
                        info!("wifi connection: client connection failed");
                        event = Some(logic::wifi::WifiEvent::ConnectionNotEstablished);
                        Timer::after_secs(3).await;
                    }
                }
            }
            logic::wifi::WifiAction::WaitForDisconnect => {
                debug!("wifi connection: wait for disconnect");
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                event = Some(logic::wifi::WifiEvent::Disconnected);
                debug!("wifi connection: disconnect successfull");
            }
            logic::wifi::WifiAction::Ignore => {
                warn!("wifi connection: this should not happen")
            }
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    info!("net: start task");
    runner.run().await
}

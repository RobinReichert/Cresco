use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_println as _;
use esp_radio::wifi::{
    AccessPointConfig, ModeConfig, WifiApState, WifiController, WifiDevice, WifiEvent,
};
use logic::config::SERVER_IP;

use crate::{dhcp, dns, mk_static};

const SSID: &str = "Cresco";

pub async fn start_wifi(
    radio_init: &'static esp_radio::Controller<'static>,
    wifi: esp_hal::peripherals::WIFI<'static>,
    rng: Rng,
    spawner: &Spawner,
) -> Stack<'static> {
    let (wifi_controller, interfaces) = esp_radio::wifi::new(radio_init, wifi, Default::default())
        .expect("Failed to initialize Wi-Fi controller");

    let wifi_interface = interfaces.ap;

    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let ip_addr = Ipv4Cidr::new(SERVER_IP, 24);

    let net_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: ip_addr,
        gateway: Some(SERVER_IP),
        dns_servers: Default::default(),
    });

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<4>, StackResources::<4>::new()),
        net_seed,
    );
    spawner.spawn(net_task(runner)).ok();
    spawner.spawn(connection(wifi_controller)).ok();
    spawner.spawn(dhcp::dhcp_task(stack)).ok();
    spawner.spawn(dns::dns_task(stack)).ok();

    wait_for_connection(stack).await;

    stack
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("start connection task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::ap_state() {
            WifiApState::Started => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config =
                ModeConfig::AccessPoint(AccessPointConfig::default().with_ssid(SSID.into()));
            controller.set_config(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}

async fn wait_for_connection(stack: Stack<'_>) {
    info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    info!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            info!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

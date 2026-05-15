#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
        holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use cross::{
    dhcp::dhcp_task,
    dns::dns_task,
    mk_static,
    web::{self, captive_app::CaptiveApp, services::captive_ssids::SsidsList},
    wifi::{self, connection},
};
use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use heapless::{String, Vec};
use logic::wifi::LoginData;
use picoserve::{AppBuilder, AppRouter};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let radio_init = &*mk_static!(
        esp_radio::Controller<'static>,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );
    let rng = Rng::new();

    let (wifi_controller, ap_stack, sta_stack) =
        wifi::start_wifi(radio_init, peripherals.WIFI, rng, &spawner).await;

    let ssids: Mutex<CriticalSectionRawMutex, SsidsList> = Mutex::new(Vec::new());
    let ssids: &'static _ = mk_static!(Mutex<CriticalSectionRawMutex, SsidsList>, ssids);
    let credentials: Signal<CriticalSectionRawMutex, LoginData> = Signal::new();
    let credentials: &'static _ =
        mk_static!(Signal<CriticalSectionRawMutex, LoginData>, credentials);

    spawner
        .spawn(connection(wifi_controller, &ssids, &credentials))
        .ok();

    spawner.spawn(dhcp_task(ap_stack)).ok();
    spawner.spawn(dns_task(ap_stack)).ok();

    let captive_app = mk_static!(
        AppRouter<CaptiveApp>,
        CaptiveApp {
            ssids: &ssids,
            credentials: &credentials
        }
        .build_app()
    );

    for task_id in 0..web::WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web::web_task(task_id, ap_stack, captive_app));
    }

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

use crate::web::{captive_app::CaptiveApp, cresco_app::CrescoApp};
use embassy_time::Duration;
use picoserve::{self, AppRouter, Config, Timeouts};

const PORT: u16 = 80;

pub mod captive_app;
pub mod cresco_app;
pub mod pages;
pub mod services;

pub const CAPTIVE_APP_TASK_POOL_SIZE: usize = 2;
pub const CRESCO_APP_TASK_POOL_SIZE: usize = 3;

static CONFIG: Config<Duration> = Config::new(Timeouts {
    start_read_request: Some(Duration::from_secs(5)),
    read_request: Some(Duration::from_secs(5)),
    write: Some(Duration::from_secs(20)),
    persistent_start_read_request: None,
});

#[embassy_executor::task(pool_size = CAPTIVE_APP_TASK_POOL_SIZE)]
pub async fn captive_web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<CaptiveApp<'static>>,
) {
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(app, &CONFIG, &mut http_buffer)
        .listen_and_serve(task_id, stack, PORT, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

#[embassy_executor::task(pool_size = CRESCO_APP_TASK_POOL_SIZE)]
pub async fn cresco_web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<CrescoApp>,
) {
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(app, &CONFIG, &mut http_buffer)
        .listen_and_serve(
            task_id + CAPTIVE_APP_TASK_POOL_SIZE,
            stack,
            PORT,
            &mut tcp_rx_buffer,
            &mut tcp_tx_buffer,
        )
        .await
        .into_never()
}

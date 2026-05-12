use crate::web::captive_app::CaptiveApp;
use embassy_time::Duration;
use picoserve::{self, AppRouter, Config, Timeouts};

const PORT: u16 = 80;

pub mod captive_app;
pub mod pages;
pub mod services;

pub const WEB_TASK_POOL_SIZE: usize = 2;

static CONFIG: Config<Duration> = Config::new(Timeouts {
    start_read_request: Some(Duration::from_secs(5)),
    read_request: Some(Duration::from_secs(1)),
    write: Some(Duration::from_secs(1)),
    persistent_start_read_request: None,
});

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<CaptiveApp>,
) {
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(app, &CONFIG, &mut http_buffer)
        .listen_and_serve(task_id, stack, PORT, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

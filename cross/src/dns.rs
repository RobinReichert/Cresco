use embassy_net::{Stack, udp::PacketMetadata, udp::UdpSocket};
use esp_println as _;
use logic::dns::{DnsAction, DnsServer, poison::PoisonedDnsServer};

#[embassy_executor::task]
pub async fn dns_task(stack: Stack<'static>) {
    let mut dns_server_logic = PoisonedDnsServer::new();

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket.bind(53).expect("DNS bind failed");

    let mut in_buf = [0u8; 1024];

    loop {
        if let Ok((n, remote)) = socket.recv_from(&mut in_buf).await {
            let response = dns_server_logic.handle_message(&in_buf[..n]);
            if let DnsAction::SendPacket { payload, len } = response {
                let _ = socket.send_to(&payload[..len], remote).await;
            }
        }
    }
}

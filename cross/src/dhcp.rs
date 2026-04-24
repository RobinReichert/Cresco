use defmt::info;
use embassy_net::{Stack, udp::UdpSocket, udp::PacketMetadata};
use embassy_time::Instant;
use logic::dhcp::{DhcpServer, DhcpAction, simple::SimpleDhcpServer};
use esp_println as _;

#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) {
    let mut dhcp_logic = SimpleDhcpServer::new();

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta, &mut rx_buffer,
        &mut tx_meta, &mut tx_buffer,
    );

    socket.bind(67).expect("DHCP bind failed");

    let mut in_buf = [0u8; 1024];
    let mut out_buf = [0u8; 576];

    loop {
        if let Ok((n, _)) = socket.recv_from(&mut in_buf).await {
            let response = dhcp_logic.handle_message(&in_buf[..n], Instant::now().as_secs());
            if let DhcpAction::SendPacket { payload, len, remote } = response {
                let remote_endpoint = (remote, 68);
                let _ = socket.send_to(&payload[..len], remote_endpoint).await;
            }
        }
    }
}

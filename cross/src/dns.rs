use core::net::Ipv4Addr;
use defmt::{info, warn};
use embassy_futures::select::{Either, select};
use embassy_net::{IpAddress, IpEndpoint, Stack, udp::PacketMetadata, udp::UdpSocket};
use esp_println as _;
use logic::dns::{DnsAction, DnsServer, mdns::MdnsServer, poison::PoisonedDnsServer};

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

const MDNS_GROUP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_PORT: u16 = 5353;

#[embassy_executor::task]
pub async fn mdns_task(stack: Stack<'static>) {
    let mut mdns_server = MdnsServer::new();

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    let mut in_buf = [0u8; 1024];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(MDNS_PORT).expect("mDNS bind failed");

    let destination = IpEndpoint::new(IpAddress::Ipv4(MDNS_GROUP), MDNS_PORT);

    loop {
        stack.wait_config_up().await;

        if let Err(e) = stack.join_multicast_group(MDNS_GROUP) {
            warn!("mDNS: joining multicast group failed: {:?}", e);
        }
        info!(
            "mDNS: serving, IP {:?}",
            stack.config_v4().map(|c| c.address)
        );

        loop {
            let event = select(socket.recv_from(&mut in_buf), stack.wait_config_down()).await;

            let n = match event {
                Either::First(Ok((n, _remote))) => n,
                Either::First(Err(_)) => continue,
                Either::Second(()) => break,
            };

            let Some(config) = stack.config_v4() else {
                continue;
            };

            if let DnsAction::SendPacket { payload, len } =
                mdns_server.handle_message(&in_buf[..n], config.address.address())
            {
                let _ = socket.send_to(&payload[..len], destination).await;
            }
        }

        info!("mDNS: STA address lost, waiting for reconnect");
    }
}

use defmt::{debug, info, trace};
use embassy_futures::select::{Either, select};
use embassy_net::{
    IpAddress, Stack,
    udp::{PacketMetadata, UdpSocket},
};
use logic::dns::{
    DnsAction, DnsServer,
    mdns::{MDNS_GROUP, MDNS_PORT, MdnsServer},
    poison::PoisonedDnsServer,
};

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

    debug!("dns poisoner: bind socket to port 53");
    socket.bind(53).expect("DNS bind failed");

    let mut in_buf = [0u8; 1024];

    loop {
        if let Ok((n, source)) = socket.recv_from(&mut in_buf).await {
            trace!("dns poisoner: receive {}", in_buf[..n]);
            let IpAddress::Ipv4(addr) = source.endpoint.addr;
            let response =
                dns_server_logic.handle_message(&in_buf[..n], (addr, source.endpoint.port));
            if let DnsAction::SendPacket {
                payload,
                len,
                remote,
            } = response
            {
                trace!("dns poisoner: respond {} to {}", payload[..len], remote);
                let _ = socket.send_to(&payload[..len], remote).await;
            }
        }
    }
}

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

    debug!("mDns: bind socket to port {}", MDNS_PORT);
    socket.bind(MDNS_PORT).expect("mDNS bind failed");

    loop {
        debug!("mDns: wait for connection");
        stack.wait_config_up().await;
        debug!("mDns: connection established");

        if let Err(e) = stack.join_multicast_group(MDNS_GROUP) {
            debug!("mDNS: join multicast group failed: {:?}", e);
        }
        info!("mDNS: serve, IP {:?}", stack.config_v4().map(|c| c.address));

        loop {
            let event = select(socket.recv_from(&mut in_buf), stack.wait_config_down()).await;

            let (n, source) = match event {
                Either::First(Ok((n, remote))) => (n, remote),
                Either::First(Err(e)) => {
                    trace!("mDns: recv_from failed: {}", e);
                    continue;
                }
                Either::Second(()) => break,
            };

            trace!("mDns: receive: {}", in_buf[..n]);

            let Some(config) = stack.config_v4() else {
                debug!("mDns: find IP address failed");
                continue;
            };

            let IpAddress::Ipv4(addr) = source.endpoint.addr;
            if let DnsAction::SendPacket {
                payload,
                len,
                remote,
            } = mdns_server.handle_message(
                &in_buf[..n],
                config.address.address(),
                (addr, source.endpoint.port),
            ) {
                trace!("mDns: respond {} to {}", payload[..len], remote);
                let _ = socket.send_to(&payload[..len], remote).await;
            }
        }

        debug!("mDNS: connection lost");
    }
}

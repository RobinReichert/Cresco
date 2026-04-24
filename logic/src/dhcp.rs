
use defmt::info;
use core::net::Ipv4Addr;
use smoltcp::wire::{DhcpPacket, DhcpRepr};

#[derive(defmt::Format)]
pub enum DhcpAction {
    SendPacket {
        payload: [u8; 576],
        len: usize,
        remote: Ipv4Addr
    },
    Ignore,
}

pub trait DhcpServer {
    fn handle_message(
        &mut self,
        buffer: &[u8],
        time: u64,
    ) -> DhcpAction;
}

pub mod simple {

    use smoltcp::{wire::DhcpMessageType};

    use crate::config::{CLIENT_IP, LEASE_DURATION, SERVER_IP, SUBNET_MASK};

    use super::*;

    #[derive(Debug)]
    enum SimpleDhcpState {
        Idle,
        Offered {
            transaction_id: u32,
            timestamp: u64,

        },
        Bound {
            mac: [u8; 6],
            timestamp: u64,
        },
    }

    pub struct SimpleDhcpServer {
        state: SimpleDhcpState,
    }

    impl DhcpServer for SimpleDhcpServer {

        fn handle_message(
            &mut self,
            buffer: &[u8],
            time: u64,
        ) -> DhcpAction {
            let packet = match DhcpPacket::new_checked(buffer) {
                Ok(p) => p,
                Err(_) => return DhcpAction::Ignore,
            };
            let repr = match DhcpRepr::parse(&packet) {
                Ok(r) => r,
                Err(_) => return DhcpAction::Ignore,
            };
            match (&self.state, repr.message_type) {
                (SimpleDhcpState::Idle, DhcpMessageType::Discover) => {
                    let (action, state) = handle_discover(&repr, time);
                    self.state = state;
                    action
                },
                (SimpleDhcpState::Offered{ transaction_id, timestamp }, DhcpMessageType::Discover) => {
                    let (action, state) = handle_rediscover_from_offered(&repr, *transaction_id, time, *timestamp);
                    self.state = state;
                    action
                }
                (SimpleDhcpState::Offered { transaction_id, timestamp }, DhcpMessageType::Request) => {
                    let (action, state) = handle_request(&repr, *transaction_id, time, *timestamp);
                    self.state = state;
                    action
                },
                (SimpleDhcpState::Bound{ mac, timestamp }, DhcpMessageType::Discover) => {
                    let (action, state) = handle_rediscover_from_bound(&repr, *mac, time, *timestamp);
                    self.state = state;
                    action
                }
                (SimpleDhcpState::Bound{ mac, timestamp }, DhcpMessageType::Request) => {
                    let (action, state) = handle_renew(&repr, *mac, time, *timestamp);
                    self.state = state;
                    action
                }
                (SimpleDhcpState::Bound { mac, timestamp }, DhcpMessageType::Release) => {
                    let (action, state) = handle_release(&repr, *mac, *timestamp);
                    self.state = state;
                    action
                },
                (_, _) => DhcpAction::Ignore,
            }
        }

    }

    impl SimpleDhcpServer {

        pub fn new() -> Self {
            SimpleDhcpServer { state: SimpleDhcpState::Idle }
        }
    }

    fn new_repr(message_type: DhcpMessageType, response_to: DhcpRepr) -> DhcpRepr {
        DhcpRepr {
            message_type: message_type,
            transaction_id: response_to.transaction_id,
            secs: 0,
            client_hardware_address: response_to.client_hardware_address,
            client_ip: Ipv4Addr::UNSPECIFIED,
            your_ip: CLIENT_IP,
            server_ip: SERVER_IP,
            relay_agent_ip: Ipv4Addr::UNSPECIFIED,
            broadcast: false,
            requested_ip: None,
            client_identifier: None,
            server_identifier: Some(SERVER_IP),
            parameter_request_list: None,
            max_size: None,
            subnet_mask: Some(SUBNET_MASK),
            router: Some(SERVER_IP),
            lease_duration: Some(LEASE_DURATION),
            renew_duration: None,
            rebind_duration: None,
            dns_servers: None,
            additional_options: &[],
        }
    }

    fn handle_discover(discover_message: &DhcpRepr, time: u64) -> (DhcpAction, SimpleDhcpState) {
        let offer = new_repr(DhcpMessageType::Offer, discover_message.clone());
        let mut payload = [0u8; 576];
        let mut packet = DhcpPacket::new_unchecked(&mut payload);
        match offer.emit(&mut packet) {
            Ok(_) => (),
            Err(_) => return (DhcpAction::Ignore, SimpleDhcpState::Idle),
        };
        let action = DhcpAction::SendPacket { 
            payload: payload, 
            len: offer.buffer_len(), 
            remote: Ipv4Addr::BROADCAST, 
        };

        let new_state = SimpleDhcpState::Offered {
            transaction_id: discover_message.transaction_id,
            timestamp: time,
        };
        info!("received discover. sending offer");
        (action, new_state)
    }

    fn handle_request(request_message: &DhcpRepr, transaction_id: u32, time: u64, previous_time: u64) -> (DhcpAction, SimpleDhcpState) {
        if request_message.transaction_id != transaction_id {
            return (DhcpAction::Ignore, SimpleDhcpState::Offered { transaction_id: transaction_id , timestamp: previous_time})
        }
        let ack = new_repr(DhcpMessageType::Ack, request_message.clone());
        let mut payload = [0u8; 576];
        let mut packet = DhcpPacket::new_unchecked(&mut payload);
        match ack.emit(&mut packet) {
            Ok(_) => (),
            Err(_) => return (DhcpAction::Ignore, SimpleDhcpState::Idle),
        };
        let action = DhcpAction::SendPacket { 
            payload: payload, 
            len: ack.buffer_len(), 
            remote: Ipv4Addr::BROADCAST, 
        };
        let client_mac = request_message.client_hardware_address.as_bytes();
        let mut mac_array = [0u8; 6];
        mac_array.copy_from_slice(&client_mac[..6]);
        let new_state = SimpleDhcpState::Bound {
            mac: mac_array, 
            timestamp: time,
        };
        info!("received request. sending ack");
        (action, new_state)
    }

    fn handle_release(release_message: &DhcpRepr, mac: [u8; 6], previous_time: u64) -> (DhcpAction, SimpleDhcpState) {
        let new_state = if release_message.client_hardware_address.as_bytes() == mac {
            info!("received release from client, transitioning to idle");
            SimpleDhcpState::Idle
        }else{
            info!("received release from invalid address");
            SimpleDhcpState::Bound { mac: mac, timestamp: previous_time }
        };
        (DhcpAction::Ignore, new_state)
    }

    fn handle_renew(request_message: &DhcpRepr, mac: [u8; 6], time: u64, previous_time: u64) -> (DhcpAction, SimpleDhcpState) {
        let (response, new_timestamp, remote) = if request_message.client_hardware_address.as_bytes() == mac {
            info!("extending lease duration");
            (new_repr(DhcpMessageType::Ack, request_message.clone()), time, CLIENT_IP)
        }else{
            info!("received invalid request");
            (new_repr(DhcpMessageType::Nak, request_message.clone()), previous_time, Ipv4Addr::BROADCAST)
        };
        let mut payload = [0u8; 576];
        let mut packet = DhcpPacket::new_unchecked(&mut payload);
        match response.emit(&mut packet) {
            Ok(_) => (),
            Err(_) => return (DhcpAction::Ignore, SimpleDhcpState::Idle),
        };
        let action = DhcpAction::SendPacket { 
            payload: payload, 
            len: response.buffer_len(), 
            remote: remote, 
        };
        let new_state = SimpleDhcpState::Bound {
            mac: mac, 
            timestamp: new_timestamp,
        };
        (action, new_state)
    }

    fn handle_rediscover_from_offered(discover_message: &DhcpRepr,transaction_id: u32, time: u64, previous_time: u64) -> (DhcpAction, SimpleDhcpState) {
        if time < previous_time + (LEASE_DURATION as u64) {
            info!("ip is already leased");
            let new_state = SimpleDhcpState::Offered { transaction_id: transaction_id, timestamp: previous_time }; 
            return (DhcpAction::Ignore, new_state);
        }
        let offer = new_repr(DhcpMessageType::Offer, discover_message.clone());
        let mut payload = [0u8; 576];
        let mut packet = DhcpPacket::new_unchecked(&mut payload);
        match offer.emit(&mut packet) {
            Ok(_) => (),
            Err(_) => return (DhcpAction::Ignore, SimpleDhcpState::Idle),
        };
        let action = DhcpAction::SendPacket { 
            payload: payload, 
            len: offer.buffer_len(), 
            remote: Ipv4Addr::BROADCAST, 
        };

        let new_state = SimpleDhcpState::Offered {
            transaction_id: discover_message.transaction_id,
            timestamp: time,
        };
        info!("ip lease duration has expired. leasing to new device");
        (action, new_state)
    }

    fn handle_rediscover_from_bound(discover_message: &DhcpRepr,mac: [u8; 6], time: u64, previous_time: u64) -> (DhcpAction, SimpleDhcpState) {
        if time < previous_time + (LEASE_DURATION as u64) {
            info!("ip is already leased");
            let new_state = SimpleDhcpState::Bound {
                mac: mac, 
                timestamp: previous_time,
            };
            return (DhcpAction::Ignore, new_state);
        }
        let offer = new_repr(DhcpMessageType::Offer, discover_message.clone());
        let mut payload = [0u8; 576];
        let mut packet = DhcpPacket::new_unchecked(&mut payload);
        match offer.emit(&mut packet) {
            Ok(_) => (),
            Err(_) => return (DhcpAction::Ignore, SimpleDhcpState::Idle),
        };
        let action = DhcpAction::SendPacket { 
            payload: payload, 
            len: offer.buffer_len(), 
            remote: Ipv4Addr::BROADCAST, 
        };

        let new_state = SimpleDhcpState::Offered {
            transaction_id: discover_message.transaction_id,
            timestamp: time,
        };
        info!("ip lease duration has expired. leasing to new device");
        (action, new_state)
    }

    #[cfg(test)]
    mod test {
        use smoltcp::wire::EthernetAddress;

        use super::*;

        fn new_test_repr(message_type: DhcpMessageType, transaction_id: u32, mac: EthernetAddress) -> DhcpRepr<'static> {
            DhcpRepr {
                message_type: message_type,
                transaction_id: transaction_id,
                secs: 0,
                client_hardware_address: mac,
                client_ip: Ipv4Addr::UNSPECIFIED,
                your_ip: Ipv4Addr::UNSPECIFIED,
                server_ip: SERVER_IP,
                relay_agent_ip: Ipv4Addr::UNSPECIFIED,
                broadcast: true,
                requested_ip: None,
                client_identifier: None,
                server_identifier: Some(SERVER_IP),
                parameter_request_list: None,
                max_size: None,
                subnet_mask: Some(SUBNET_MASK),
                router: Some(SERVER_IP),
                lease_duration: Some(LEASE_DURATION),
                renew_duration: None,
                rebind_duration: None,
                dns_servers: None,
                additional_options: &[],
            }
        }

        #[test]
        fn test_discover_from_idle_should_offer() {
            let mut fsm = SimpleDhcpServer::new();
            let msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut payload = [0u8; 576];
            let mut packet = DhcpPacket::new_unchecked(&mut payload);
            msg.emit(&mut packet).expect("failed to encode message");
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&payload, 0) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Offer);
                assert_eq!(remote, Ipv4Addr::BROADCAST);
            }else {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_request_with_correct_xid_should_bind() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&request_payload, 0) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Ack);
                assert_eq!(remote, Ipv4Addr::BROADCAST);
            }else {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_request_with_wrong_xid_should_ignore() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 4321, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            if let DhcpAction::SendPacket { .. } = fsm.handle_message(&request_payload, 0) {
                panic!("expected Ignore got sendMessage");
            }
        }

        #[test]
        fn test_release_from_correct_address_should_reset() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            fsm.handle_message(&request_payload, 0);
            let release_msg = new_test_repr(DhcpMessageType::Release, 234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut release_payload = [0u8; 576];
            let mut release_packet = DhcpPacket::new_unchecked(&mut release_payload);
            release_msg.emit(&mut release_packet).expect("failed to encode request message");
            if let DhcpAction::SendPacket { .. } = fsm.handle_message(&release_payload, 0) {
                panic!("expected Ignore got sendMessage");
            }
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&discover_payload, 0) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Offer);
                assert_eq!(remote, Ipv4Addr::BROADCAST);
            }else {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_release_from_wrong_address_should_ignore() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            fsm.handle_message(&request_payload, 0);
            let release_msg = new_test_repr(DhcpMessageType::Release, 4321, EthernetAddress([6, 5, 4, 3, 2, 1]));
            let mut release_payload = [0u8; 576];
            let mut release_packet = DhcpPacket::new_unchecked(&mut release_payload);
            release_msg.emit(&mut release_packet).expect("failed to encode request message");
            if let DhcpAction::SendPacket { .. } = fsm.handle_message(&release_payload, 0) {
                panic!("expected Ignore got sendMessage");
            }
            if let DhcpAction::SendPacket { .. } = fsm.handle_message(&discover_payload, 0) {
                panic!("expected Ignore got sendMessage");
            }
        }

        #[test]
        fn test_renew_from_correct_address_should_ack() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            fsm.handle_message(&request_payload, 0);
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&request_payload, (LEASE_DURATION / 2) as u64) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Ack);
                assert_eq!(remote, CLIENT_IP);
            }else {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_renew_from_wrong_address_should_nak() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            fsm.handle_message(&request_payload, 0);
            let renew_msg = new_test_repr(DhcpMessageType::Request, 4321, EthernetAddress([6, 5, 4, 3, 2, 1]));
            let mut renew_payload = [0u8; 576];
            let mut renew_packet = DhcpPacket::new_unchecked(&mut renew_payload);
            renew_msg.emit(&mut renew_packet).expect("failed to encode renew message");
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&renew_payload, (LEASE_DURATION / 2) as u64) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Nak);
                assert_eq!(remote, Ipv4Addr::BROADCAST);
            }else {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_rediscover_from_offered_after_timeout_should_offer() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&discover_payload, (LEASE_DURATION * 2) as u64) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Offer);
                assert_eq!(remote, Ipv4Addr::BROADCAST);
            }else {
                panic!("expected sendPacket got Ignore"); }
        }

        #[test]
        fn test_rediscover_from_offered_before_timeout_should_ignore() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            if let DhcpAction::SendPacket { .. } = fsm.handle_message(&discover_payload, (LEASE_DURATION / 2) as u64) {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_rediscover_from_bound_after_timeout_should_ack() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            fsm.handle_message(&request_payload, 0);
            if let DhcpAction::SendPacket { payload, remote, .. } = fsm.handle_message(&discover_payload, (LEASE_DURATION * 2) as u64) {
                let packet = DhcpPacket::new_checked(&payload).expect("failed to decode result");
                let repr = DhcpRepr::parse(&packet).expect("could not convert packet to repr");
                assert_eq!(repr.message_type, DhcpMessageType::Offer);
                assert_eq!(remote, Ipv4Addr::BROADCAST);
            }else {
                panic!("expected sendPacket got Ignore");
            }
        }

        #[test]
        fn test_rediscover_from_bound_before_timeout_should_ignore() {
            let mut fsm = SimpleDhcpServer::new();
            let discover_msg = new_test_repr(DhcpMessageType::Discover, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut discover_payload = [0u8; 576];
            let mut discover_packet = DhcpPacket::new_unchecked(&mut discover_payload);
            discover_msg.emit(&mut discover_packet).expect("failed to encode discover message");
            fsm.handle_message(&discover_payload, 0);
            let request_msg = new_test_repr(DhcpMessageType::Request, 1234, EthernetAddress([1, 2, 3, 4, 5, 6]));
            let mut request_payload = [0u8; 576];
            let mut request_packet = DhcpPacket::new_unchecked(&mut request_payload);
            request_msg.emit(&mut request_packet).expect("failed to encode request message");
            fsm.handle_message(&request_payload, 0);
            if let DhcpAction::SendPacket { .. } = fsm.handle_message(&discover_payload, (LEASE_DURATION / 2) as u64) {
                panic!("expected sendPacket got Ignore");
            }
        }
    }

}

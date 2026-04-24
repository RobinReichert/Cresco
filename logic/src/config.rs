use smoltcp::wire::Ipv4Address;

pub const SERVER_IP: Ipv4Address = Ipv4Address::new(192, 168, 1, 1);
pub const SUBNET_MASK: Ipv4Address = Ipv4Address::new(255, 255, 255, 0);
pub const CLIENT_IP: Ipv4Address = Ipv4Address::new(192, 168, 1, 2);
pub const LEASE_DURATION: u32 = 60;

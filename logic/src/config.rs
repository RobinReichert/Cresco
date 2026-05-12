use smoltcp::wire::Ipv4Address;

macro_rules! define_device {
    ($a:literal, $b:literal, $c:literal, $d:literal) => {
        pub const SERVER_IP: Ipv4Address = Ipv4Address::new($a, $b, $c, $d);
        pub const SERVER_URL: &str = concat!("http://", $a, ".", $b, ".", $c, ".", $d, "/");
    };
}

define_device!(192, 168, 1, 1);
pub const SUBNET_MASK: Ipv4Address = Ipv4Address::new(255, 255, 255, 0);
pub const CLIENT_IP: Ipv4Address = Ipv4Address::new(192, 168, 1, 2);
pub const LEASE_DURATION: u32 = 60;

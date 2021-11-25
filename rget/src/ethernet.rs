use rand;
use std::fmt::Display;
use std::fmt;

use rand::RngCore;
use smoltcp::wire;

#[derive(Debug)]
pub struct MacAddress([u8; 6]);

impl Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octet = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            octet[0], octet[1], octet[2], octet[3], octet[4], octet[5],
        )
    }
}

impl MacAddress {
    pub fn new() -> MacAddress {
        let mut octets: [u8; 6] = [0; 6];
        rand::thread_rng().fill_bytes(&mut octets); // 1. 随机生成
        octets[0] |= 0b_0000_0010; // 2
        octets[1] &= 0b_1111_1110; // 3
        MacAddress { 0: octets }
    }
}

impl Into<wire::EthernetAddress> for MacAddress {
    fn into(self) -> wire::EthernetAddress {
        wire::EthernetAddress { 0: self.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_address_should_work() {
        let addr = MacAddress::new();
        assert_eq!(addr.0.len(), 6);
        let wire_addr: wire::EthernetAddress = addr.into();
        assert_eq!(wire_addr.0.len(), 6)
    }
}

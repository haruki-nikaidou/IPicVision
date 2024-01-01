use serde::{Serialize, Deserialize};

#[derive(PartialEq, Clone, Copy)]
pub struct Ipv4 {
    address: [u8;4]
}

impl Ipv4 {
    pub fn mask(&self, mask: &[u8;4] ) -> [u8;4] {
        let mut result: [u8;4] = [0;4];
        for i in 0..4 {
            result[i] = self.address[i] & mask[i];
        }
        result
    }
    pub fn compare(a: &Self, b: &Self, mask: &[u8;4]) -> bool {
        let a_masked = a.mask(mask);
        let b_masked = b.mask(mask);
        a_masked == b_masked
    }
    pub fn new(addr: [u8;4]) -> Self {
        Ipv4 {
            address: addr
        }
    }
    pub fn from_string(ip_str: &str) -> Option<Self> {
        let mut result: [u8;4] = [0;4];
        let mut i = 0;
        for octet in ip_str.split(".") {
            if i > 3 {
                return None;
            }
            match octet.parse::<u8>() {
                Ok(octet) => result[i] = octet,
                Err(_) => return None
            }
            i += 1;
        }
        Some(Ipv4::new(result))
    }
    pub fn cidr_to_mask(cidr: u8) -> [u8;4] {
        let mut result: [u8;4] = [0;4];
        let mut i = 0;
        while i < cidr {
            let octet = i / 8;
            let bit = i % 8;
            result[octet as usize] |= 1 << (7 - bit);
            i += 1;
        }
        result
    }
    pub fn to_string(&self) -> String {
        self.address.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(".")
    }
}

impl Serialize for Ipv4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
            Ok(serializer.serialize_str(&self.address.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("."))?)
    }
}

impl<'de> Deserialize<'de> for Ipv4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
            let s = String::deserialize(deserializer)?;
            match Ipv4::from_string(&s) {
                Some(ip) => Ok(ip),
                None => Err(serde::de::Error::custom("invalid ipv4 address"))
            }
    }
}
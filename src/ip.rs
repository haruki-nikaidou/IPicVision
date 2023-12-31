#[derive(PartialEq)]
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
    pub fn from_string(ip_str: &String) -> Option<Self> {
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
}
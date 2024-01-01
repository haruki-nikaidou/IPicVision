use ipinfo::{self, IpInfoConfig, IpInfo};

use crate::ip::Ipv4;

/// Token for ipinfo.io
const IP_INFO_TOKEN:&str = "";

/// Turn on geoip feature
const GEO_ENABLED: bool = false;

pub async fn get_ip_country(ip: &Ipv4) -> Option<String> {
    if !GEO_ENABLED {
        return None;
    }
    let config = IpInfoConfig {
        token: Some(IP_INFO_TOKEN.to_string()),
        ..Default::default()
    };

    let mut ipinfo = IpInfo::new(config)
        .expect("Ip Info should construct");

    let res = ipinfo.lookup(ip.to_string().as_str()).await;

    match res {
        Ok(info) => {
            Some(info.country)
        },
        Err(e) => {
            println!("Failed to get ip info: {:?}", e);
            None
        }
    }
}
use ipinfo::{self, IpInfoConfig, IpInfo};

use crate::ip::Ipv4;
use tracing::{warn, info, error};

/// Token for ipinfo.io
const IP_INFO_TOKEN:&str = "";

/// Turn on geoip feature
const GEO_ENABLED: bool = false;

pub async fn get_ip_country(ip: &Ipv4) -> Option<String> {
    info!("Getting ip info for {}", ip.to_string());
    if !GEO_ENABLED {
        warn!("Geoip feature is not enabled but is used. Please enable it in geo.rs");
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
            error!("Failed to get ip info: {:?}", e);
            None
        }
    }
}
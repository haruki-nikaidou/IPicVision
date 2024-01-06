use std::net::IpAddr;
use ipinfo::{self, IpInfo, IpInfoConfig};

use tracing::error;

pub async fn get_ip_country(ip: &IpAddr, token: &String) -> Option<String> {
    let config = IpInfoConfig {
        token: Some(token.clone()),
        ..Default::default()
    };
    let mut client = match IpInfo::new(config) {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create ipinfo client: {:?}", e);
            return None;
        }
    };
    let res = client.lookup(ip.to_string().as_str()).await;

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
use std::net::IpAddr;
use ipinfo::{self, IpInfo};

use tracing::error;

pub async fn get_ip_country(ip: &IpAddr, client: &mut IpInfo) -> Option<String> {
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
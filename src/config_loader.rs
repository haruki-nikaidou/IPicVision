use crate::rule::TrafficMatcherList;
use serde::Deserialize;
use tracing::error;
use serde_json::Error;

const CONFIG_PATH: &str = "./config.json";

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub traffic_matchers: TrafficMatcherList,
    pub ip_info_enable: Option<bool>,
    pub ip_info_token: Option<String>,
    pub listen_addr: String,
}

pub fn parse_config(config: &str) -> Result<Config, Error> {
    let config: Config = serde_json::from_str(config)?;
    Ok(config)
}

pub fn load_config() -> Result<Config, ()> {
    let file = match std::fs::read_to_string(CONFIG_PATH) {
        Ok(file) => file,
        Err(_) => {
            error!("Failed to read config file");
            return Err(());
        }
    };
    let config = file;
    match parse_config(&config) {
        Ok(config) => Ok(config),
        Err(e) => {
            error!("Failed to parse config: {:?}", e);
            Err(())
        }
    }
}
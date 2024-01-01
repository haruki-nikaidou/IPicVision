use crate::rule::TrafficMatcherList;
use serde::Deserialize;
use tracing::error;
use serde_json::Error;

const CONFIG_PATH: &str = "./config.json";

pub fn parse_config(config: &str) -> Result<TrafficMatcherList, Error> {
    TrafficMatcherList::deserialize(&mut serde_json::Deserializer::from_str(config))
}

pub fn load_config() -> Result<TrafficMatcherList, ()> {
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
use crate::rule::TrafficMatcherList;
use serde::Deserialize;

const CONFIG_PATH: &str = "./config.json";

pub fn parse_config(config: &str) -> Result<TrafficMatcherList, serde_json::Error> {
    TrafficMatcherList::deserialize(&mut serde_json::Deserializer::from_str(config))
}

pub fn load_config() -> Result<TrafficMatcherList, serde_json::Error> {
    let config = std::fs::read_to_string(CONFIG_PATH).expect("Failed to read config file");
    parse_config(&config)
}
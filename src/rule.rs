use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr};
use rand::Rng;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use tracing::{error, info, warn};
use crate::config_loader::Config;
use crate::geo::get_ip_country;

#[derive(Debug, Clone, PartialEq)]
pub enum ImageInfo {
    Path(String),
    Url(String)
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum TrafficMatchRule {
    #[serde(rename = "ipv4_exact")]
    Ipv4Exact(Ipv4Addr),

    #[serde(rename = "ipv4_masked")]
    Ipv4Masked { ip: Ipv4Addr, mask: Ipv4Addr },

    #[serde(rename = "ipv4_cidr")]
    Ipv4Cidr(Ipv4Addr, u8),

    #[serde(rename = "region")]
    Region(String),

    #[serde(rename = "ipv4_default")]
    Ipv4Default,

    #[serde(rename = "ipv6_default")]
    Ipv6Default,

    #[serde(rename = "default")]
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageInfoSelectStrategy {
    FixToOne(ImageInfo),
    Random(Vec<ImageInfo>),
}

fn parse_image_info(s: String) -> ImageInfo {
    if s.starts_with("http://") {
        warn!("Using http url is not recommended");
        ImageInfo::Url(s)
    } else if s.starts_with("https://") {
        ImageInfo::Url(s)
    } else {
        ImageInfo::Path(s)
    }
}

impl<'de> Deserialize<'de> for ImageInfoSelectStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let v: Value = Deserialize::deserialize(deserializer)?;
        match v {
            Value::String(s) => Ok(ImageInfoSelectStrategy::FixToOne(parse_image_info(s))),
            Value::Array(arr) => {
                let mut images = Vec::new();
                for item in arr {
                    let image = match item {
                        Value::String(s) => parse_image_info(s),
                        _ => {
                            return Err(serde::de::Error::custom("Invalid image info"));
                        }
                    };
                    images.push(image);
                }
                Ok(ImageInfoSelectStrategy::Random(images))
            }
            _ => {
                Err(serde::de::Error::custom("Invalid image info"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrafficMatcher(TrafficMatchRule, ImageInfoSelectStrategy);

pub type TrafficMatcherList = Vec<TrafficMatcher>;
pub type TrafficMatchFn = dyn (Fn(&IpAddr) -> Option<ImageInfo>) + Send + Sync + 'static;


impl<'de> Deserialize<'de> for TrafficMatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            role: TrafficMatchRule,
            image: ImageInfoSelectStrategy,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(TrafficMatcher(helper.role, helper.image))
    }
}

fn match_ipv4_exact(ip: &Ipv4Addr, rule: &Ipv4Addr) -> bool {
    ip == rule
}

fn match_ipv4_masked(ip: &Ipv4Addr, rule: &Ipv4Addr, mask: &Ipv4Addr) -> bool {
    let ip_octets = ip.octets();
    let rule_octets = rule.octets();
    let mask_octets = mask.octets();

    for i in 0..4 {
        if ip_octets[i] & mask_octets[i] != rule_octets[i] & mask_octets[i] {
            return false;
        }
    }

    true
}

fn match_ipv4_cidr(ip: &Ipv4Addr, rule: &Ipv4Addr, cidr: u8) -> bool {
    let ip_int = u32::from(*ip);
    let rule_int = u32::from(*rule);
    let mask = !0u32 << (32 - cidr);

    (ip_int & mask) == (rule_int & mask)
}

fn match_region(_ip: &IpAddr, _rule: &String, enable: bool, token: &String) -> bool {
    info!("Getting ip info for {}", _ip.to_string());
    if !enable {
        warn!("Ip info is not enabled");
        return false;
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    let country = rt.block_on(get_ip_country(_ip, token));
    match country {
        Some(country) => {
            info!("Got ip info for {}: {}", _ip.to_string(), country);
            country == *_rule
        }
        None => {
            error!("Failed to get ip info for {}", _ip.to_string());
            false
        }
    }
}

fn random_select(images: &Vec<ImageInfo>) -> ImageInfo {
    let mut rng = rand::rng();
    let index = rng.random_range(0..images.len());
    images[index].clone()
}

pub fn generate_match_fn(config: Config) -> Arc<TrafficMatchFn> {
    let rule_list = config.traffic_matchers;
    let enable = if config.ip_info_enable.is_none() {
        false
    } else {
        config.ip_info_enable.unwrap()
    };
    let token = config.ip_info_token;
    if enable && token.is_none() {
        error!("Ip info is enabled but token is not configured");
        panic!();
    }
    let token = token.unwrap_or_else(|| "".to_string());
    Arc::new(move |ip| {
        for TrafficMatcher(rule, strategy) in rule_list.iter() {
            let is_match;
            if match ip {
                IpAddr::V4(_) => false,
                IpAddr::V6(_) => true,
            } {
                is_match = match rule {
                    TrafficMatchRule::Region(rule) => match_region(ip, rule, enable, &token),
                    TrafficMatchRule::Ipv6Default => true,
                    TrafficMatchRule::Ipv4Default => false,
                    TrafficMatchRule::Default => false,
                    _ => false,
                };
            } else {
                let ip = match ip {
                    IpAddr::V4(ip) => ip,
                    IpAddr::V6(_) => unreachable!(),
                };
                is_match = match rule {
                    TrafficMatchRule::Ipv4Exact(rule) => match_ipv4_exact(ip, rule),
                    TrafficMatchRule::Ipv4Masked { ip: rule, mask } => match_ipv4_masked(ip, rule, mask),
                    TrafficMatchRule::Ipv4Cidr(rule, cidr) => match_ipv4_cidr(ip, rule, *cidr),
                    TrafficMatchRule::Region(rule) => match_region(&IpAddr::V4(*ip), rule, enable, &token),
                    TrafficMatchRule::Ipv4Default => true,
                    TrafficMatchRule::Ipv6Default => false,
                    TrafficMatchRule::Default => true,
                };
            }

            if is_match {
                return match strategy {
                    ImageInfoSelectStrategy::FixToOne(info) => Some(info.clone()),
                    ImageInfoSelectStrategy::Random(images) => Some(random_select(images)),
                };
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_info() {
        let path = "path/to/image.png".to_string();
        let url = "https://example.com/image.png".to_string();
        let http_url = "http://example.com/image.png".to_string();
        let https_url = "https://example.com/image.png".to_string();
        assert_eq!(parse_image_info(path), ImageInfo::Path("path/to/image.png".to_string()));
        assert_eq!(parse_image_info(url), ImageInfo::Url("https://example.com/image.png".to_string()));
        assert_eq!(parse_image_info(http_url), ImageInfo::Url("http://example.com/image.png".to_string()));
        assert_eq!(parse_image_info(https_url), ImageInfo::Url("https://example.com/image.png".to_string()));
    }

    #[test]
    fn test_deserialize_image_info_select_strategy() {
        let fix_to_one = r#""path/to/image.png""#;
        let random = r#"["path/to/image.png", "https://example.com/image.png"]"#;
        let fix_to_one_deserialized: ImageInfoSelectStrategy = serde_json::from_str(fix_to_one).unwrap();
        let random_deserialized: ImageInfoSelectStrategy = serde_json::from_str(random).unwrap();
        assert_eq!(
            fix_to_one_deserialized,
            ImageInfoSelectStrategy::FixToOne(ImageInfo::Path("path/to/image.png".to_string()))
        );
        assert_eq!(
            random_deserialized,
            ImageInfoSelectStrategy::Random(
                vec![
                    ImageInfo::Path("path/to/image.png".to_string()),
                    ImageInfo::Url("https://example.com/image.png".to_string()),
                ]
            )
        );
    }

    #[test]
    fn test_traffic_match_rule_deserialize() {
        let ipv4_exact = r#"
        {
            "ipv4_exact": "192.168.1.1"
        }"#;
        let ipv4_exact_result: TrafficMatchRule = TrafficMatchRule::Ipv4Exact(Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(serde_json::from_str::<TrafficMatchRule>(ipv4_exact).unwrap(), ipv4_exact_result);


        let ipv4_masked = r#"
        {
            "ipv4_masked": {
                "ip": "192.168.1.1",
                "mask": "255.255.0.0"
            }
        }"#;
        let ipv4_masked_result: TrafficMatchRule = TrafficMatchRule::Ipv4Masked {
            ip: Ipv4Addr::new(192, 168, 1, 1),
            mask: Ipv4Addr::new(255, 255, 0, 0),
        };
        assert_eq!(serde_json::from_str::<TrafficMatchRule>(ipv4_masked).unwrap(), ipv4_masked_result);
    }
}


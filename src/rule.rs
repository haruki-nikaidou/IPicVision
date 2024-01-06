use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr};
use std::ops::DerefMut;
use ipinfo::IpInfo;
use rand::Rng;
use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeStruct};
use tracing::{info, warn};
use crate::config_loader::Config;

#[derive(Clone, Serialize, Deserialize)]
pub enum ImageInfo {
    Path(String),
    Url(String)
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TrafficMatchRule {
    #[serde(rename = "ipv4_exact")]
    Ipv4Exact(Ipv4Addr),

    #[serde(rename = "ipv4_masked")]
    Ipv4Masked{ip: Ipv4Addr, mask: Ipv4Addr},

    #[serde(rename = "ipv4_cidr")]
    Ipv4Cidr(Ipv4Addr, u8),

    #[serde(rename = "region")]
    Region(String),

    #[serde(rename = "ipv4_default")]
    Ipv4Default,

    #[serde(rename = "ipv6_default")]
    Ipv6Default,

    #[serde(rename = "region_default")]
    Default,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename = "image")]
pub enum ImageInfoSelectStrategy {
    #[serde(rename = "one")]
    FixToOne(ImageInfo),

    #[serde(rename = "random_list")]
    Random(Vec<ImageInfo>),
}

pub struct TrafficMatcher (TrafficMatchRule, ImageInfoSelectStrategy);
pub type TrafficMatcherList = Vec<TrafficMatcher>;
pub type TrafficMatchFn = dyn (FnMut(&IpAddr) -> Option<ImageInfo>) + Send + Sync + 'static;

impl Serialize for TrafficMatcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TrafficMatcher", 2)?;
        state.serialize_field("role", &self.0)?;
        state.serialize_field("image", &self.1)?;
        state.end()
    }
}

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

fn match_region(_ip: &IpAddr, _rule: &String, client: &mut Option<IpInfo>) -> bool {
    info!("Getting ip info for {}", _ip.to_string());
    match client {
        Some(client) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let geo_ip_result = rt.block_on(crate::geo::get_ip_country(_ip, client));
            match geo_ip_result {
                Some(country) => {
                    info!("Got ip info for {}: {}", _ip.to_string(), country);
                    country == *_rule
                },
                None => false
            }
        },
        None => {
            warn!("Ip info client is not configured");
            return false;
        }
    }
}

fn random_select(images: &Vec<ImageInfo>) -> ImageInfo {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..images.len());
    images[index].clone()
}

pub fn generate_match_fn(config: Config) -> Arc<TrafficMatchFn> {
    let rule_list = config.traffic_matchers;
    let enable_geo = config.ip_info_enable;
    let geo_token = config.ip_info_token;
    let ip_info_config = ipinfo::IpInfoConfig {
        token: Some(geo_token),
        ..Default::default()
    };
    let mut ip_info = if enable_geo {
        Some(IpInfo::new(ip_info_config).expect("Ip Info should construct"))
    } else {
        None
    };
    Arc::new(move |ip| {
        for TrafficMatcher(rule, strategy) in rule_list.iter() {
            let is_match;
            if match ip {
                IpAddr::V4(_) => false,
                IpAddr::V6(_) => true,
            } {
                is_match = match rule {
                    TrafficMatchRule::Region(rule) => match_region(ip, rule, &mut ip_info),
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
                    TrafficMatchRule::Ipv4Masked{ip: rule, mask} => match_ipv4_masked(ip, rule, mask),
                    TrafficMatchRule::Ipv4Cidr(rule, cidr) => match_ipv4_cidr(ip, rule, *cidr),
                    TrafficMatchRule::Region(rule) => match_region(&IpAddr::V4(*ip), rule, &mut ip_info),
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

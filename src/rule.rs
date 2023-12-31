use std::sync::Arc;

use crate::ip::Ipv4;
use rand::Rng;
use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeStruct};

#[derive(Clone, Serialize, Deserialize)]
pub enum ImageInfo {
    Path(String),
    Url(String)
}

#[derive(Serialize, Deserialize, Clone)]
pub enum TrafficMatchRule {
    #[serde(rename = "ipv4_exact")]
    Ipv4Exact(Ipv4),

    #[serde(rename = "ipv4_masked")]
    Ipv4Masked(Ipv4, [u8;4]),

    #[serde(rename = "ipv4_cidr")]
    Ipv4Cidr(Ipv4, u8),

    #[serde(rename = "region")]
    Region(String),
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
pub type TrafficMatchFn = dyn Fn(&Ipv4) -> Option<ImageInfo>;

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

fn match_ipv4_exact(ip: &Ipv4, rule: &Ipv4) -> bool {
    ip == rule
}

fn match_ipv4_masked(ip: &Ipv4, rule: &Ipv4, mask: &[u8;4]) -> bool {
    Ipv4::compare(ip, rule, mask)
}

fn match_ipv4_cidr(ip: &Ipv4, rule: &Ipv4, cidr: u8) -> bool {
    let mask = Ipv4::cidr_to_mask(cidr);
    Ipv4::compare(ip, rule, &mask)
}

fn match_region(_ip: &Ipv4, _rule: &String) -> bool {
    todo!()
}

fn random_select(images: &Vec<ImageInfo>) -> ImageInfo {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..images.len());
    images[index].clone()
}

pub fn generate_match_fn(rule_list: TrafficMatcherList) -> Arc<TrafficMatchFn> {
    Arc::new(move |ip| {
        for TrafficMatcher(rule, strategy) in rule_list.iter() {
            let is_match = match rule {
                TrafficMatchRule::Ipv4Exact(rule) => match_ipv4_exact(ip, rule),
                TrafficMatchRule::Ipv4Masked(rule, mask) => match_ipv4_masked(ip, rule, mask),
                TrafficMatchRule::Ipv4Cidr(rule, cidr) => match_ipv4_cidr(ip, rule, *cidr),
                TrafficMatchRule::Region(rule) => match_region(ip, rule),
            };

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

use crate::ip::Ipv4;
use rand::Rng;

#[derive(Clone)]
pub enum ImageInfo {
    Path(String),
    Url(String)
}

pub enum TrafficMatchRule {
    Ipv4Exact(Ipv4),
    Ipv4Masked(Ipv4, [u8;4]),
    Ipv4Cidr(Ipv4, u8),
    Region(String),
}

pub enum ImageInfoSelectStrategy {
    FixToOne(ImageInfo),
    Random(Vec<ImageInfo>),
}

pub type TrafficMatcher = (TrafficMatchRule, ImageInfoSelectStrategy);
pub type TrafficMatcherList = Vec<TrafficMatcher>;
pub type TrafficMatchFn = dyn Fn(&Ipv4) -> Option<ImageInfo>;

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

pub fn generate_match_fn(rule_list: TrafficMatcherList) -> Box<TrafficMatchFn> {
    Box::new(move |ip| {
        for (rule, strategy) in rule_list.iter() {
            match rule {
                TrafficMatchRule::Ipv4Exact(rule) => {
                    if match_ipv4_exact(ip, rule) {
                        match strategy {
                            ImageInfoSelectStrategy::FixToOne(info) => {
                                return Some(info.clone());
                            },
                            ImageInfoSelectStrategy::Random(images) => {
                                return Some(random_select(images));
                            }
                        }
                    }
                },
                TrafficMatchRule::Ipv4Masked(rule, mask) => {
                    if match_ipv4_masked(ip, rule, mask) {
                        match strategy {
                            ImageInfoSelectStrategy::FixToOne(info) => {
                                return Some(info.clone());
                            },
                            ImageInfoSelectStrategy::Random(images) => {
                                return Some(random_select(images));
                            }
                        }
                    }
                },
                TrafficMatchRule::Ipv4Cidr(rule, cidr) => {
                    if match_ipv4_cidr(ip, rule, cidr.clone()) {
                        match strategy {
                            ImageInfoSelectStrategy::FixToOne(info) => {
                                return Some(info.clone());
                            },
                            ImageInfoSelectStrategy::Random(images) => {
                                return Some(random_select(images));
                            }
                        }
                    }
                },
                TrafficMatchRule::Region(rule) => {
                    if match_region(ip, rule) {
                        match strategy {
                            ImageInfoSelectStrategy::FixToOne(info) => {
                                return Some(info.clone());
                            },
                            ImageInfoSelectStrategy::Random(images) => {
                                return Some(random_select(images));
                            }
                        }
                    }
                }
            }
        }
        return None;
    })
}
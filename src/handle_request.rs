use actix_web::{Responder, HttpResponse};

use crate::ip::{Ipv4, self};
use crate::rule::ImageInfo;
use std::sync::Arc;

pub async fn handle_request(
    req: actix_web::HttpRequest,
    get_image_info: Arc<dyn Fn(&Ipv4) -> Option<ImageInfo>>
) -> impl Responder {
    let ip_addr = auto_get_real_ip(&req);

    if ip_addr.is_none() {
        return HttpResponse::BadRequest().finish();
    }

    if let Some(info) = get_image_info(&ip_addr.unwrap()) {
        match info {
            ImageInfo::Path(path) => {
                let file_content = match std::fs::read(&path) {
                    Ok(content) => content,
                    Err(_) => return HttpResponse::NotFound().finish(),
                };
                HttpResponse::Ok()
                    .content_type("image/png")
                    .body(file_content)
            }
            ImageInfo::Url(url) => HttpResponse::Found()
                .insert_header(("Location", url))
                .finish(),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

fn auto_get_real_ip(req: &actix_web::HttpRequest) -> Option<Ipv4> {
    let cf_header = req.headers().get("CF-Connecting-IP");
    let x_forwarded_for_header = req.headers().get("X-Forwarded-For");
    let proxy_header = match (cf_header, x_forwarded_for_header) {
        (Some(cf_header), _) => cf_header.to_str(),
        (_, Some(x_forwarded_for_header)) => x_forwarded_for_header.to_str(),
        _ => Ok(""),
    }.unwrap().to_string();

    let is_proxy = !proxy_header.is_empty();

    if is_proxy {
        return Ipv4::from_string(&proxy_header)
    } else {
        let ip_addr = match req.peer_addr() {
            Some(addr) => addr.ip(),
            None => return None,
        };
    
        let ip_addr = match ip_addr {
            std::net::IpAddr::V4(ipv4) => ip::Ipv4::from_string(ipv4.to_string().as_str()).unwrap(),
            std::net::IpAddr::V6(_) => return None,
        };
        return Some(ip_addr);
    }
}
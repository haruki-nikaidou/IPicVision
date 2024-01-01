use actix_web::{Responder, HttpResponse};

use crate::ip::{Ipv4, self};
use crate::rule::ImageInfo;
use std::sync::Arc;
use tracing::info;

pub async fn handle_request(
    req: actix_web::HttpRequest,
    get_image_info: Arc<dyn Fn(&Ipv4) -> Option<ImageInfo>>
) -> impl Responder {
    let request_from_proxy = is_proxy(&req);
    let ip_addr = auto_get_real_ip(&req);

    if ip_addr.is_none() {
        info!("Failed to get ip address from request. This should not happen.");
        return HttpResponse::BadRequest().finish();
    }

    if let Some(info) = get_image_info(&ip_addr.unwrap()) {
        match info {
            ImageInfo::Path(path) => {
                let file_content = match std::fs::read(&path) {
                    Ok(content) => content,
                    Err(_) => return HttpResponse::NotFound().finish(),
                };
                request_log(request_from_proxy, &ip_addr.unwrap(), &path);
                HttpResponse::Ok()
                    .content_type("image/png")
                    .body(file_content)
            }
            ImageInfo::Url(url) => {
                request_log(request_from_proxy, &ip_addr.unwrap(), &url);
                HttpResponse::Found()
                .insert_header(("Location", url))
                .finish()
            },
        }
    } else {
        request_log(request_from_proxy, &ip_addr.unwrap(), &"\"No image found\"".to_string());
        HttpResponse::NotFound().finish()
    }
}

fn is_proxy(req: &actix_web::HttpRequest) -> bool {
    let cf_header = req.headers().get("CF-Connecting-IP");
    let x_forwarded_for_header = req.headers().get("X-Forwarded-For");
    let proxy_header = match (cf_header, x_forwarded_for_header) {
        (Some(cf_header), _) => cf_header.to_str(),
        (_, Some(x_forwarded_for_header)) => x_forwarded_for_header.to_str(),
        _ => Ok(""),
    }.unwrap().to_string();

    !proxy_header.is_empty()
}

fn auto_get_real_ip(req: &actix_web::HttpRequest) -> Option<Ipv4> {
    if is_proxy(req) {
        let cf_header = req.headers().get("CF-Connecting-IP");
        let x_forwarded_for_header = req.headers().get("X-Forwarded-For");
        let proxy_header = match (cf_header, x_forwarded_for_header) {
            (Some(cf_header), _) => cf_header.to_str(),
            (_, Some(x_forwarded_for_header)) => x_forwarded_for_header.to_str(),
            _ => Ok(""),
        }.unwrap().to_string();
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


fn request_log(is_proxy: bool, real_ip: &Ipv4, image_info: &String) {
    if is_proxy {
        info!("Request from {} via reverse proxy", real_ip.to_string());
    } else {
        info!("Request from {}", real_ip.to_string());
    }
    info!("Response to {} with {}", real_ip.to_string(), image_info);
}
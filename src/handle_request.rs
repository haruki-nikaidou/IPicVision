use std::net::{IpAddr};
use std::str::FromStr;
use actix_web::{Responder, HttpResponse};

use crate::rule::{ImageInfo, TrafficMatchFn};
use std::sync::Arc;
use tracing::info;

pub async fn handle_request(
    req: actix_web::HttpRequest,
    get_image_info: Arc<TrafficMatchFn>
) -> impl Responder {
    let client_ip = get_client_ip(&req).0;
    let request_from_proxy = get_client_ip(&req).1;

    if client_ip.is_none() {
        info!("Failed to get ip address from request. This should not happen.");
        return HttpResponse::BadRequest().finish();
    }

    if let Some(info) = get_image_info(&client_ip.unwrap()) {
        match info {
            ImageInfo::Path(path) => {
                let file_content = match std::fs::read(&path) {
                    Ok(content) => content,
                    Err(_) => return HttpResponse::NotFound().finish(),
                };
                request_log(request_from_proxy, &client_ip.unwrap(), &path);
                HttpResponse::Ok()
                    .content_type("image/png")
                    .body(file_content)
            }
            ImageInfo::Url(url) => {
                request_log(request_from_proxy, &client_ip.unwrap(), &url);
                HttpResponse::Found()
                .insert_header(("Location", url))
                .finish()
            },
        }
    } else {
        request_log(request_from_proxy, &client_ip.unwrap(), &"\"No image found\"".to_string());
        HttpResponse::NotFound().finish()
    }
}


/// Retrieves the client's IP address from an HttpRequest.
///
/// This function attempts to extract the client's IP address from various headers
/// commonly set by reverse proxies (e.g., Cloudflare, Nginx). If these headers are not
/// present, it falls back to the peer address of the connection.
///
/// # Arguments
/// * `req` - A reference to the HttpRequest object.
///
/// # Returns
/// An `(Option<IpAddr>,bool)` which is `Some(IpAddr)` if an IP address can be determined, or `None` otherwise.
/// if the second value is true, the request is from a reverse proxy.
fn get_client_ip(req: &actix_web::HttpRequest) -> (Option<IpAddr>,bool) {
    // Try to get the IP address from the `CF-Connecting-IP` header (used by Cloudflare)
    if let Some(ip) = req.headers().get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .and_then(|ip| IpAddr::from_str(ip).ok()) {
        (Some(ip), true)
        // If the above fails, try to get the IP address from the `X-Real-IP` header (commonly used by reverse proxies)
    } else if let Some(ip) = req.headers().get("X-Real-IP")
        .and_then(|v| v.to_str().ok())
        .and_then(|ip| IpAddr::from_str(ip).ok()) {
        (Some(ip), true)
        // If the above fails, try to get the IP address from the `X-Forwarded-For` header.
        // This header can contain a list of IPs, so we take the first one.
    } else if let Some(ip) = req.headers().get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok()) {
        let ip = ip.split(',').next().and_then(|ip| IpAddr::from_str(ip.trim()).ok());
        (ip, true)
        // If none of the headers are present, fall back to the IP address of the peer.
    } else {
        (req.peer_addr().map(|addr| addr.ip()),false)
    }
}


fn request_log(is_proxy: bool, client_ip: &IpAddr, image_info: &String) {
    if is_proxy {
        info!("Request from {} via reverse proxy", client_ip.to_string());
    } else {
        info!("Request from {}", client_ip.to_string());
    }
    info!("Response to {} with {}", client_ip.to_string(), image_info);
}
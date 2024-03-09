use std::net::{IpAddr};
use std::sync::Arc;
use actix_web::{get, HttpResponse, Responder, web};
use tracing::{info};
use crate::rule::{ImageInfo, TrafficMatchFn};

#[get("/img")]
pub async fn handle_redirect_request(
    req: actix_web::HttpRequest,
    get_image_info: web::Data<Arc<TrafficMatchFn>>
) -> impl Responder {
    let connection_info = req.connection_info();
    let real_ip = match connection_info.realip_remote_addr() {
        Some(ip) => ip,
        None => {
            return HttpResponse::BadRequest().finish();
        }
    };
    let real_ip = match real_ip.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(_) => {
            return HttpResponse::BadRequest().finish();
        }
    };
    if let Some(info) = get_image_info(&real_ip) {
        match info {
            ImageInfo::Path(path) => {
                let file_content = match std::fs::read(&path) {
                    Ok(content) => content,
                    Err(_) => return HttpResponse::NotFound().finish(),
                };
                request_log(&real_ip, &path);
                HttpResponse::Ok()
                    .content_type("image/png")
                    .body(file_content)
            }
            ImageInfo::Url(url) => {
                request_log(&real_ip, &url);
                HttpResponse::Found()
                    .insert_header(("Location", url))
                    .finish()
            },
        }
    } else {
        request_log(&real_ip, &"\"No image found\"".to_string());
        HttpResponse::NotFound().finish()
    }
}

fn request_log(client_ip: &IpAddr, image_info: &String) {
    info!("Response to {} with {}", client_ip.to_string(), image_info);
}
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ip::Ipv4;
use rule::ImageInfo;
use std::sync::Arc;

mod ip;
mod rule;
mod config_loader;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config_loader::load_config().expect("Failed to load config");
    let get_image_info = Arc::new(rule::generate_match_fn(config));
    HttpServer::new(move || {
        let get_image_info = get_image_info.clone();
        App::new().route("/img", web::get().to(move |req| handle_request(req, get_image_info.clone())))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn handle_request(req: actix_web::HttpRequest, get_image_info: Arc<Box<dyn Fn(&Ipv4) -> Option<ImageInfo>>>) -> impl Responder {
    let ip_addr = match req.peer_addr() {
        Some(addr) => addr.ip(),
        None => return HttpResponse::BadRequest().finish(),
    };

    let ip_addr = match ip_addr {
        std::net::IpAddr::V4(ipv4) => ip::Ipv4::from_string(ipv4.to_string().as_str()).unwrap(),
        std::net::IpAddr::V6(_) => return HttpResponse::BadRequest().finish(),
    };

    if let Some(info) = get_image_info(&ip_addr) {
        match info {
            ImageInfo::Path(path) => {
                let file_content = match std::fs::read(&path) {
                    Ok(content) => content,
                    Err(_) => return HttpResponse::NotFound().finish(),
                };
                HttpResponse::Ok().content_type("image/png").body(file_content)
            }
            ImageInfo::Url(url) => HttpResponse::Found().insert_header(("Location", url)).finish(),
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}

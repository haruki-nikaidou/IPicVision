use actix_web::{web, App, HttpServer};
use handle_request::handle_request;

mod config_loader;
mod ip;
mod rule;
mod handle_request;
mod geo;

const LISTEN_ADDR: &str = "127.0.0.1:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config_loader::load_config().expect("Failed to load config");

    let get_image_info = rule::generate_match_fn(config);
    HttpServer::new(move || {
        let get_image_info = get_image_info.clone();
        App::new().route(
            "/img",
            web::get().to(move |req| handle_request(req, get_image_info.clone())),
        )
    })
    .bind(LISTEN_ADDR)?
    .run()
    .await
}



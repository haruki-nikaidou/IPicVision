use actix_web::{web, App, HttpServer};
use handle_request::handle_request;
use tracing::{info, error};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod config_loader;
mod rule;
mod handle_request;
mod geo;

fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_tracing();
    info!("Starting server...");
    let config = match config_loader::load_config() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {:?}", e);
            return Ok(());
        }
    };
    let listen_addr = config.listen_addr.clone();
    info!("Listening on {}", &listen_addr);
    let get_image_info = rule::generate_match_fn(config);
    info!("Config loaded");
    HttpServer::new(move || {
        let get_image_info = get_image_info.clone();
        App::new().route(
            "/img",
            web::get().to(move |req| handle_request(req, get_image_info.clone())),
        )
    })
    .bind(listen_addr)?
    .run()
    .await
}



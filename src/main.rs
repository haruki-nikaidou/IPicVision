use actix_web::{web, App, HttpServer};
use tracing::{info, error};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use crate::handle_request::handle_redirect_request;

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
    let get_image_info = rule::generate_match_fn(config);

    info!("Listening on {}", &listen_addr);

    info!("Config loaded");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(get_image_info.clone()))
            .service(handle_redirect_request)
    })
    .bind(listen_addr)?
    .run()
    .await
}



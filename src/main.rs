use poem::{
    listener::TcpListener,
    middleware::{CatchPanic, Tracing},
    session::{CookieConfig, CookieSession},
    web::cookie::CookieKey,
    EndpointExt, Server,
};
use service::SignService;
use std::sync::Arc;
use tokio::sync::Mutex;

use db::MemDb;
use web_app::WebApp;

mod db;
mod service;
mod template;
mod web_app;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let db = MemDb::new();
    let sign_service = SignService::default();
    let app = WebApp::new();

    let router = WebApp::setup_route()
        .data(Arc::new(Mutex::new(app)))
        .data(Arc::new(Mutex::new(db)))
        .data(Arc::new(Mutex::new(sign_service)))
        .with(CookieSession::new(
            CookieConfig::private(CookieKey::generate()).secure(false),
        ))
        .with(Tracing)
        .with(CatchPanic::new())
        .catch_all_error(web_app::custom_error);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("waas")
        .run(router)
        .await
}

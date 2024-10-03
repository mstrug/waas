use poem::{
    get, handler, listener::TcpListener, middleware::Tracing, web::Path, EndpointExt, Route, Server,
    session::{CookieConfig, CookieSession, Session},
};
use poem::web::cookie::CookieKey;
use std::sync::RwLock;
use web_app::WebApp;
use std::sync::{Arc};
use std::cell::RefCell;
use tokio::sync::Mutex;


mod web_app;
mod api;
mod db;
mod service;



#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let app = WebApp::new();
    let router = WebApp::setup_route()
        .data(Arc::new(Mutex::new(app)))
        .with(CookieSession::new(CookieConfig::private( CookieKey::generate() )))
        .with(Tracing);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("waas")
        .run(router)
        .await
}

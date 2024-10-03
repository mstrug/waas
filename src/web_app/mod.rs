use poem::middleware::AddDataEndpoint;
use poem::{
    get, handler,
    http::{header, StatusCode},
    listener::TcpListener,
    middleware::AddData,
    session::{CookieConfig, CookieSession, Session},
    web::{Data, Form, Html},
    EndpointExt, IntoResponse, Response, Result, Route, Server,
};
use poem::post;
use poem::Error;
use pwhash::bcrypt::*;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

mod template;
use template::*;

use crate::db::UserId;
use crate::db::{DbInterface, MemDb};

pub struct WebApp {
    current_user: Option<UserId>,
}

#[derive(Deserialize)]
struct LoginParams {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct SignMessageParams {
    message: String,
}
    
#[handler]
fn view_login() -> impl IntoResponse {
    Html(format!("{}{}{}{}", 
        HTML_HEAD, 
        HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, ""), 
        HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_CONTENT_LOGIN),
        HTML_BODY_FOOTER))
}

#[handler]
async fn view_login_validate(
    Form(params): Form<LoginParams>,
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
) -> impl IntoResponse {
    let pass_hash = WebApp::hash_password(&params.password);
    println!("{} {} {}", params.username, params.password, pass_hash);

    if let Ok(user_id) = db
        .lock()
        .await
        .validate_user_password(&params.username, &pass_hash)
        .await
    {
        session.set("username", params.username);
        state.lock().await.set_current_user(Some(user_id));
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/")
            .finish()
    } else {
        Html(format!("{}{}{}{}", 
        HTML_HEAD, 
        HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, HTML_NAVBAR_MENU_ITEM_LOGIN), 
        HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_WRONG_PASS),
        HTML_BODY_FOOTER))
        .into_response()
    }
}


#[handler]
async fn view_sign_message(
    Form(params): Form<SignMessageParams>,
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
) -> impl IntoResponse {

    Html(format!("{}{}{}{}", 
    HTML_HEAD, 
    HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, &format!("{}{}", HTML_NAVBAR_MENU_ITEM_LOGOUT, HTML_NAVBAR_MENU_ITEM_DISCARD_KEY)), 
    HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_CONTENT_SIGN_ONGOING),
    HTML_BODY_FOOTER))
    .into_response()
}

#[handler]
async fn view_index(
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
) -> impl IntoResponse {
    //println!("session empty: {}", session.is_empty());

    if let Some(user_id) = state.lock().await.current_user {
        let key_available = db.lock().await.get_user_key(user_id).await.is_ok();
        let username = session.get::<String>("username").unwrap();

        let (menu_item, body_content) = if !key_available {
            (HTML_NAVBAR_MENU_ITEM_GENERATE_KEY, HTML_BODY_CONTENT_NO_KEY.replace(HTML_USERNAME_PLACEHOLDER, &username))
        } else {
            (HTML_NAVBAR_MENU_ITEM_DISCARD_KEY, HTML_BODY_CONTENT_SIGN_MESSAGE.to_string())
        };

        Html(format!("{}{}{}{}", 
            HTML_HEAD, 
            HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, &format!("{}{}", HTML_NAVBAR_MENU_ITEM_LOGOUT, menu_item)), 
            HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, &body_content),
            HTML_BODY_FOOTER))
        .into_response()
    } else {
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .finish()
    }
}

#[handler]
async fn view_logout(session: &Session, state: Data<&Arc<Mutex<WebApp>>>) -> impl IntoResponse {
    session.purge();
    state.lock().await.set_current_user(None);
    println!("loggedout");
    println!("session empty: {}", session.is_empty());
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/")
        .finish()
}

pub async fn custom_error(err: Error) -> impl IntoResponse {
    Html(format!("{}{}{}{}", 
            HTML_HEAD, 
            HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, ""), 
            HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, &HTML_BODY_CONTENT_ANY_ERROR.replace(HTML_ERROR_PLACEHOLDER, &err.to_string())),
            HTML_BODY_FOOTER))
        .into_response()
}

impl WebApp {
    pub fn new() -> Self {
        Self { current_user: None }
    }

    fn set_current_user(&mut self, user: Option<UserId>) {
        self.current_user = user;
    }

    fn hash_password(pass: &str) -> String {
        let setup = BcryptSetup {
            salt: Some("gifLHpZdNAixJzy36HyOcK"),
            cost: Some(5),
            variant: Some(BcryptVariant::V2y),
        };

        hash_with(setup, pass).unwrap()
    }

    pub fn setup_route() -> Route {
        Route::new()
            .at("/", get(view_index))
            .at("/login", get(view_login).post(view_login_validate))
            .at("/logout", get(view_logout))
            .at("/sign", post(view_sign_message))
    }
}

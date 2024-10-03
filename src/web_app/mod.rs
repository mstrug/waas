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
struct SigninParams {
    username: String,
    password: String,
}

    
#[handler]
pub fn view_entry() -> impl IntoResponse {
    Html(format!("{}{}{}{}", 
        HTML_HEAD, 
        HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, ""), 
        HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_CONTENT_LOGIN),
        HTML_BODY_FOOTER))
}

#[handler]
pub async fn view_entry_handle(
    Form(params): Form<SigninParams>,
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
        Html(
            r#"
            <!DOCTYPE html>
            <html>
            <head><meta charset="UTF-8"><title>Example Session Auth</title></head>
            <body>
            no such user or wrong password
            </body>
            </html>
            "#,
        )
        .into_response()
    }
}

#[handler]
async fn index(
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

        // Html(format!(
        //     r#"
        //     <!DOCTYPE html>
        //     <html>
        //     <head><meta charset="UTF-8"><title>Example Session Auth</title></head>
        //     <body>
        //     <div>hello {username}, you are viewing a restricted resource</div>
        //     <a href="/logout">click here to logout</a><br>
        //     <a href="/key/generate">click here to generate the key</a><br>
        //     {sign_message}
        //     </body>
        //     </html>
        //     "#
        // ))
        .into_response()
    } else {
        Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .finish()
    }
}

#[handler]
async fn logout(session: &Session, state: Data<&Arc<Mutex<WebApp>>>) -> impl IntoResponse {
    session.purge();
    state.lock().await.set_current_user(None);
    println!("loggedout");
    println!("session empty: {}", session.is_empty());
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/")
        .finish()
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
            .at("/", get(index))
            .at("/login", get(view_entry).post(view_entry_handle))
            .at("/logout", get(logout))
    }
}

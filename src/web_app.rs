use poem::middleware::AddDataEndpoint;
use poem::post;
use poem::Error;
use poem::{
    get, handler,
    http::{header, StatusCode},
    listener::TcpListener,
    middleware::AddData,
    session::{CookieConfig, CookieSession, Session},
    web::{Data, Form, Html, Path},
    web::sse::{Event, SSE},
    EndpointExt, IntoResponse, Response, Result, Route, Server,
};
use pwhash::bcrypt::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures_util::stream;
use std::time::Instant;
use tokio::time::Duration;
use futures_util::{stream::BoxStream, Stream, StreamExt};

use super::template::*;
use super::db::UserId;
use super::db::{DbInterface, MemDb};
use super::service::SignService;

#[derive(Default)]
pub struct WebApp {
    current_user: Option<UserId>,
    pending_messages: HashMap<UserId, String>,
    signed_messages: HashMap<UserId, String>,
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
    Html(format!(
        "{}{}{}{}",
        HTML_HEAD,
        HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, ""),
        HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_CONTENT_LOGIN),
        HTML_BODY_FOOTER
    ))
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
        Html(format!(
            "{}{}{}{}",
            HTML_HEAD,
            HTML_BODY_NAVBAR.replace(
                HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                HTML_NAVBAR_MENU_ITEM_LOGIN
            ),
            HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_WRONG_PASS),
            HTML_BODY_FOOTER
        ))
        .into_response()
    }
}

#[handler]
async fn view_sign_message(
    Form(params): Form<SignMessageParams>,
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
    sign_service: Data<&Arc<Mutex<SignService>>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    if let Some(user_id) = state.current_user {
        if state.pending_messages.get(&user_id).is_some() {
            return custom_error(Error::from_string("User already waits for message sign", StatusCode::NOT_FOUND)).await.into_response()
        }

        if let Ok(key) = db.lock().await.get_user_key(user_id).await {
            //let signed_message = sign_service.lock().await.sign_message(&params.message, &key).await;
            
            state.pending_messages.insert(user_id, params.message);

            Html(format!(
                "{}{}{}{}{}",
                HTML_HEAD,
                HTML_BODY_NAVBAR.replace(
                    HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                    &format!(
                        "{}{}",
                        HTML_NAVBAR_MENU_ITEM_LOGOUT, HTML_NAVBAR_MENU_ITEM_DISCARD_KEY
                    )
                ),
                HTML_BODY_CONTENT.replace(
                    HTML_BODY_CONTENT_PLACEHOLDER, HTML_BODY_CONTENT_SIGN_ONGOING
                ),
                format!(r##" <script>
                    var eventSource = new EventSource('event/{user_id}');
                    eventSource.onmessage = function(event) {{
                        console.log("Received event");
                        eventSource.close();
                        console.log(event.data);
                        const obj = JSON.parse(event.data);
                        console.log(obj);

                        const elem = document.getElementById("sign_progress");
                        elem.value = 100;

                        if (obj.error === "none") {{
                            console.log("redirecting");
                            sign_done();
                        }} else {{
                            console.log(obj.error); // todo
                        }}
                    }}
                    async function sign_done() {{
                        await new Promise(resolve => setTimeout(resolve, 500));
                        window.location.href = "/message-signed"
                    }}
                    </script>
                "##),
                HTML_BODY_FOOTER
            ))
            .into_response()
        } else {
            custom_error(Error::from_string("Key not found", StatusCode::NOT_FOUND)).await.into_response()
        }
    } else {
        custom_error(Error::from_string("User not found", StatusCode::NOT_FOUND)).await.into_response()
    }
}

#[handler]
async fn view_message_signed(
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
    sign_service: Data<&Arc<Mutex<SignService>>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;
    if let Some(user_id) = state.current_user {
        if let Some(msg) = state.signed_messages.remove(&user_id) {
            Html(format!(
                "{}{}{}{}",
                HTML_HEAD,
                HTML_BODY_NAVBAR.replace(
                    HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                    &format!(
                        "{}{}{}",
                        HTML_NAVBAR_MENU_ITEM_LOGOUT, HTML_NAVBAR_MENU_ITEM_SIGN_MESSAGE, HTML_NAVBAR_MENU_ITEM_DISCARD_KEY,
                    )
                ),
                HTML_BODY_CONTENT.replace(
                    HTML_BODY_CONTENT_PLACEHOLDER, &HTML_BODY_CONTENT_MESSAGE_SIGNED.replace(HTML_BODY_CONTENT_INTERNAL_PLACEHOLDER, &msg)
                ),
                HTML_BODY_FOOTER
            ))
            .into_response()
        } else {
            custom_error(Error::from_string("User doesn't have any signed messages", StatusCode::NOT_FOUND)).await.into_response()
        }
    } else {
        custom_error(Error::from_string("User not found", StatusCode::NOT_FOUND)).await.into_response()
    }
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
            (
                HTML_NAVBAR_MENU_ITEM_GENERATE_KEY,
                HTML_BODY_CONTENT_NO_KEY.replace(HTML_USERNAME_PLACEHOLDER, &username),
            )
        } else {
            (
                HTML_NAVBAR_MENU_ITEM_DISCARD_KEY,
                HTML_BODY_CONTENT_SIGN_MESSAGE.to_string(),
            )
        };

        Html(format!(
            "{}{}{}{}",
            HTML_HEAD,
            HTML_BODY_NAVBAR.replace(
                HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                &format!("{}{}", HTML_NAVBAR_MENU_ITEM_LOGOUT, menu_item)
            ),
            HTML_BODY_CONTENT.replace(HTML_BODY_CONTENT_PLACEHOLDER, &body_content),
            HTML_BODY_FOOTER
        ))
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
    Html(format!(
        "{}{}{}{}",
        HTML_HEAD,
        HTML_BODY_NAVBAR.replace(HTML_NAVBAR_MENU_ITEM_PLACEHOLDER, ""),
        HTML_BODY_CONTENT.replace(
            HTML_BODY_CONTENT_PLACEHOLDER,
            &HTML_BODY_CONTENT_ANY_ERROR.replace(HTML_ERROR_PLACEHOLDER, &err.to_string())
        ),
        HTML_BODY_FOOTER
    ))
    .into_response()
}

#[handler]
async fn event(Path(user_id): Path<UserId>, state: Data<&Arc<Mutex<WebApp>>>, db: Data<&Arc<Mutex<MemDb>>>, sign_service: Data<&Arc<Mutex<SignService>>>) -> SSE {
    println!("called event endpoint: {}", user_id);
    // todo: check session
    let mut state = state.lock().await;

    let event = if let Some(msg) = state.pending_messages.remove(&user_id) {
        let key = db.lock().await.get_user_key(user_id).await.unwrap();
        if let Ok(output) = sign_service.lock().await.sign_message(&msg, &key).await {
            state.signed_messages.insert(user_id, output);
            Event::message(format!(r##"{{"user_id": {user_id}, "error": "none"}}"##))
        } else {
            Event::message(format!(r##"{{"user_id": {user_id}, "error": "Signing of message failed!"}}"##))
        }
    } else {
        Event::message(format!(r##"{{"user_id": {user_id}, "error": "No pending message for current user!"}}"##))
    };

    SSE::new(
        stream::once( async move {event})
    )
}


impl WebApp {
    pub fn new() -> Self {
        Self::default()
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
            .at("/event/:user_id", get(event))
            .at("/message-signed", get(view_message_signed))
    }
}

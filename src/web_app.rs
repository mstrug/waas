use futures_util::stream;
use poem::{
    get, handler,
    http::{header, StatusCode},
    post,
    session::Session,
    web::sse::{Event, SSE},
    web::{Data, Form, Html, Path},
    Error, IntoResponse, Response, Route,
};
use pwhash::bcrypt::*;
use rand::Rng;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use super::db::{MemDb, UserId};
use super::service::SignService;
use super::template::*;

#[derive(Default)]
pub struct WebApp {
    // Map of currently logged users and cookie session
    current_users: HashMap<String, UserId>,
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

    if let Some(pass_hash) = pass_hash {
        if let Ok(user_id) = db
            .lock()
            .await
            .validate_user_password(&params.username, &pass_hash)
        {
            let user_session: String = (0..16)
                .map(|_| char::from(rand::thread_rng().gen_range(32..127)))
                .collect();
            session.set("user_session", &user_session);

            state
                .lock()
                .await
                .current_users
                .insert(user_session, user_id);

            return Response::builder()
                .status(StatusCode::FOUND)
                .header(header::LOCATION, "/")
                .finish();
        }
    }

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

#[handler]
async fn view_sign_message(
    Form(params): Form<SignMessageParams>,
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;

    if let Some(user_session) = session.get::<String>("user_session") {
        if let Some(user_id) = state.current_users.get(&user_session) {
            let user_id = user_id.clone();
            if state.pending_messages.get(&user_id).is_some() {
                return custom_error(Error::from_string(
                    "User already waits for message sign",
                    StatusCode::NOT_FOUND,
                ))
                .await
                .into_response();
            }

            if db.lock().await.get_user_key(user_id).is_ok() {
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
                        HTML_BODY_CONTENT_PLACEHOLDER,
                        HTML_BODY_CONTENT_SIGN_ONGOING
                    ),
                    format!(
                        r##" <script>
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
                "##
                    ),
                    HTML_BODY_FOOTER
                ))
                .into_response()
            } else {
                custom_error(Error::from_string("Key not found", StatusCode::NOT_FOUND))
                    .await
                    .into_response()
            }
        } else {
            custom_error(Error::from_string("User not found", StatusCode::NOT_FOUND))
                .await
                .into_response()
        }
    } else {
        custom_error(Error::from_string(
            "User session not found",
            StatusCode::NOT_FOUND,
        ))
        .await
        .into_response()
    }
}

#[handler]
async fn view_message_signed(
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
) -> impl IntoResponse {
    let mut state = state.lock().await;

    if let Some(user_session) = session.get::<String>("user_session") {
        if let Some(user_id) = state.current_users.get(&user_session) {
            let user_id = user_id.clone();
            if let Some(msg) = state.signed_messages.remove(&user_id) {
                Html(format!(
                    "{}{}{}{}",
                    HTML_HEAD,
                    HTML_BODY_NAVBAR.replace(
                        HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                        &format!(
                            "{}{}{}",
                            HTML_NAVBAR_MENU_ITEM_LOGOUT,
                            HTML_NAVBAR_MENU_ITEM_SIGN_MESSAGE,
                            HTML_NAVBAR_MENU_ITEM_DISCARD_KEY,
                        )
                    ),
                    HTML_BODY_CONTENT.replace(
                        HTML_BODY_CONTENT_PLACEHOLDER,
                        &HTML_BODY_CONTENT_MESSAGE_SIGNED
                            .replace(HTML_BODY_CONTENT_INTERNAL_PLACEHOLDER, &msg)
                    ),
                    HTML_BODY_FOOTER
                ))
                .into_response()
            } else {
                custom_error(Error::from_string(
                    "User doesn't have any signed messages",
                    StatusCode::NOT_FOUND,
                ))
                .await
                .into_response()
            }
        } else {
            custom_error(Error::from_string("User not found", StatusCode::NOT_FOUND))
                .await
                .into_response()
        }
    } else {
        custom_error(Error::from_string(
            "User session not found",
            StatusCode::NOT_FOUND,
        ))
        .await
        .into_response()
    }
}

#[handler]
async fn view_index(
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
) -> impl IntoResponse {
    let go_to_login_view = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/login")
        .finish();

    if let Some(user_session) = session.get::<String>("user_session") {
        if let Some(user_id) = state.lock().await.current_users.get(&user_session) {
            let key_available = db.lock().await.get_user_key(*user_id).is_ok();

            let (menu_item, body_content) = if !key_available {
                let user_name = db
                    .lock()
                    .await
                    .get_user_name(*user_id)
                    .unwrap_or("Unknown user".to_string());
                (
                    HTML_NAVBAR_MENU_ITEM_GENERATE_KEY,
                    HTML_BODY_CONTENT_NO_KEY.replace(HTML_USERNAME_PLACEHOLDER, &user_name),
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
            // session cookit exists but user not logged in
            session.remove(&user_session);
            go_to_login_view
        }
    } else {
        go_to_login_view
    }
}

#[handler]
async fn view_logout(session: &Session, state: Data<&Arc<Mutex<WebApp>>>) -> impl IntoResponse {
    if let Some(user_session) = session.get::<String>("user_session") {
        state.lock().await.current_users.remove(&user_session);
    }

    session.purge();
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/")
        .finish()
}

#[handler]
async fn view_generate_key(
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
    sign_service: Data<&Arc<Mutex<SignService>>>,
) -> impl IntoResponse {
    if let Some(user_session) = session.get::<String>("user_session") {
        if let Some(user_id) = state.lock().await.current_users.get(&user_session) {
            if db.lock().await.get_user_key(*user_id).is_ok() {
                custom_error(Error::from_string(
                    "User already has a key",
                    StatusCode::NOT_FOUND,
                ))
                .await
                .into_response()
            } else {
                let key = sign_service.lock().await.generate_key();
                db.lock().await.add_user_key(*user_id, &key).ok();
                Html(format!(
                    "{}{}{}{}",
                    HTML_HEAD,
                    HTML_BODY_NAVBAR.replace(
                        HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                        &format!(
                            "{}{}{}",
                            HTML_NAVBAR_MENU_ITEM_LOGOUT,
                            HTML_NAVBAR_MENU_ITEM_SIGN_MESSAGE,
                            HTML_NAVBAR_MENU_ITEM_DISCARD_KEY
                        )
                    ),
                    HTML_BODY_CONTENT.replace(
                        HTML_BODY_CONTENT_PLACEHOLDER,
                        HTML_BODY_CONTENT_KEY_GENERATED
                    ),
                    HTML_BODY_FOOTER
                ))
                .into_response()
            }
        } else {
            custom_error(Error::from_string("User not found", StatusCode::NOT_FOUND))
                .await
                .into_response()
        }
    } else {
        custom_error(Error::from_string(
            "User session not found",
            StatusCode::NOT_FOUND,
        ))
        .await
        .into_response()
    }
}

#[handler]
async fn view_discard_key(
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
) -> impl IntoResponse {
    if let Some(user_session) = session.get::<String>("user_session") {
        if let Some(user_id) = state.lock().await.current_users.get(&user_session) {
            if !db.lock().await.get_user_key(*user_id).is_ok() {
                custom_error(Error::from_string(
                    "User doesn't have a key",
                    StatusCode::NOT_FOUND,
                ))
                .await
                .into_response()
            } else {
                db.lock().await.discard_user_key(*user_id).ok();
                Html(format!(
                    "{}{}{}{}",
                    HTML_HEAD,
                    HTML_BODY_NAVBAR.replace(
                        HTML_NAVBAR_MENU_ITEM_PLACEHOLDER,
                        &format!(
                            "{}{}",
                            HTML_NAVBAR_MENU_ITEM_LOGOUT, HTML_NAVBAR_MENU_ITEM_GENERATE_KEY
                        )
                    ),
                    HTML_BODY_CONTENT.replace(
                        HTML_BODY_CONTENT_PLACEHOLDER,
                        HTML_BODY_CONTENT_KEY_DISCARDED
                    ),
                    HTML_BODY_FOOTER
                ))
                .into_response()
            }
        } else {
            custom_error(Error::from_string("User not found", StatusCode::NOT_FOUND))
                .await
                .into_response()
        }
    } else {
        custom_error(Error::from_string(
            "User session not found",
            StatusCode::NOT_FOUND,
        ))
        .await
        .into_response()
    }
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
async fn favicon() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .content_type("image/vnd.microsoft.icon")
        .body(vec![
            0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0x20, 0, 0x30, 0, 0, 0, 0x16, 0, 0, 0, 0x28, 0, 0,
            0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0x20, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0xC2, 0x1E, 0, 0,
            0x2C, 0x1E, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFC, 0xFE, 0xFC, 0xFF, 0, 0, 0, 0,
        ])
        .into_response()
}

#[handler]
async fn event(
    Path(user_id): Path<UserId>,
    session: &Session,
    state: Data<&Arc<Mutex<WebApp>>>,
    db: Data<&Arc<Mutex<MemDb>>>,
    sign_service: Data<&Arc<Mutex<SignService>>>,
) -> SSE {
    println!("1");
    let event = if let Some(user_session) = session.get::<String>("user_session") {
        println!("1");
        let mut state = state.lock().await;
        if let Some(user_id_from_state) = state.current_users.get(&user_session) {
            println!("1");
            if user_id == *user_id_from_state {
                println!("1");

                if let Some(msg) = state.pending_messages.remove(&user_id) {
                    if let Ok(key) = db.lock().await.get_user_key(user_id) {
                        if let Ok(output) = sign_service.lock().await.sign_message(&msg, &key).await
                        {
                            state.signed_messages.insert(user_id, output);
                            Event::message(format!(
                                r##"{{"user_id": {user_id}, "error": "none"}}"##
                            ))
                        } else {
                            Event::message(format!(
                                r##"{{"user_id": {user_id}, "error": "Signing of message failed!"}}"##
                            ))
                        }
                    } else {
                        Event::message(format!(
                            r##"{{"user_id": {user_id}, "error": "Key not found!"}}"##
                        ))
                    }
                } else {
                    Event::message(format!(
                        r##"{{"user_id": {user_id}, "error": "No pending message for current user!"}}"##
                    ))
                }
            } else {
                Event::message(format!(
                    r##"{{"user_id": {user_id}, "error": "User not matched with the session!"}}"##
                ))
            }
        } else {
            Event::message(format!(
                r##"{{"user_id": {user_id}, "error": "No current user!"}}"##
            ))
        }
    } else {
        Event::message(format!(
            r##"{{"user_id": {user_id}, "error": "No session for current user!"}}"##
        ))
    };

    SSE::new(stream::once(async move { event }))
}

impl WebApp {
    pub fn new() -> Self {
        Self::default()
    }

    fn hash_password(pass: &str) -> Option<String> {
        let setup = BcryptSetup {
            salt: Some("gifLHpZdNAixJzy36HyOcK"),
            cost: Some(5),
            variant: Some(BcryptVariant::V2y),
        };

        hash_with(setup, pass).ok()
    }

    pub fn setup_route() -> Route {
        Route::new()
            .at("/", get(view_index))
            .at("/login", get(view_login).post(view_login_validate))
            .at("/logout", get(view_logout))
            .at("/sign", post(view_sign_message))
            .at("/event/:user_id", get(event))
            .at("/message-signed", get(view_message_signed))
            .at("/key/generate", get(view_generate_key))
            .at("/key/discard", get(view_discard_key))
            .at("/favicon.ico", get(favicon))
    }
}

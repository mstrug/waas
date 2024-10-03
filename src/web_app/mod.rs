use poem::{
    get, handler, http::{header, StatusCode}, 
    listener::TcpListener, middleware::AddData, session::{CookieConfig, CookieSession, Session}, 
    web::{Form, Html, Data}, EndpointExt, IntoResponse, Response, Result, Route, Server
};
use std::sync::{Arc};
use serde::Deserialize;
use poem::middleware::AddDataEndpoint;
use tokio::sync::Mutex;

mod view_entry;
mod view_main;
use view_entry::*;

use crate::db::{DbInterface, MemDb};
use crate::db::UserId;


pub struct WebApp {
    value: String,
    current_user: Option<UserId>
}


#[derive(Deserialize)]
struct SigninParams {
    username: String,
    password: String,
}

#[handler]
pub async fn view_entry_handle(Form(params): Form<SigninParams>, session: &Session, state: Data<&Arc<Mutex<WebApp>>>, db: Data<&Arc<Mutex<MemDb>>>) -> impl IntoResponse {

    if let Ok(user_id) = db.lock().await.validate_user_password(&params.username, &params.password).await {
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
    no such user
    </body>
    </html>
    "#,
        )
        .into_response()
    }


    // if (params.username == "test" || params.username == "test2" ) && params.password == "123456" {
    //     session.set("username", params.username);
    //     Response::builder()
    //         .status(StatusCode::FOUND)
    //         .header(header::LOCATION, "/")
    //         .finish()
    // } else {
    //     Html(
    //         r#"
    // <!DOCTYPE html>
    // <html>
    // <head><meta charset="UTF-8"><title>Example Session Auth</title></head>
    // <body>
    // no such user
    // </body>
    // </html>
    // "#,
    //     )
    //     .into_response()
    // }
}

#[handler]
async fn index(session: &Session, state: Data<&Arc<Mutex<WebApp>>>, db: Data<&Arc<Mutex<MemDb>>>) -> impl IntoResponse {
    //println!("session empty: {}", session.is_empty());

    if let Some(user_id) = state.lock().await.current_user {
        let sign_message = if let Ok(_) = db.lock().await.get_user_key(user_id).await {
            format!(r#"<a href="/key/sign">click here to sign message</a>"#)
        } else {
            String::new()
        };
    
        let username = session.get::<String>("username").unwrap();
    
        Html(format!(
            r#"
    <!DOCTYPE html>
    <html>
    <head><meta charset="UTF-8"><title>Example Session Auth</title></head>
    <body>
    <div>hello {username}, you are viewing a restricted resource</div>
    <a href="/logout">click here to logout</a><br>
    <a href="/key/generate">click here to generate the key</a><br>
    {sign_message}
    </body>
    </html>
    "#
        ))
        .into_response()
        }
        else {

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
        Self { value: String::from("ASD"),
        current_user: None}
    }

    fn set_current_user(&mut self, user: Option<UserId>) {
        self.current_user = user;
    }

    fn test(&mut self) {
        println!("{}", self.value);
        self.value = String::from("SSS");
        println!("{}", self.value);
    }

    pub fn setup_route() -> Route {
        //let state = Arc::new(Self{});
    
        Route::new()
            .at("/", get(index))
            .at("/login", get(view_entry).post(view_entry_handle) )
            .at("/logout", get(logout) )
            
            //.at("/hello/:name", get(hello))
            //.at("/login/", get(login))
            //.at("/logout/", get(logout))
        
    }
}
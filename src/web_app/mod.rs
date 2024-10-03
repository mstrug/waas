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



#[derive(Default)]
pub struct WebApp {
    value: String,
}


#[derive(Deserialize)]
struct SigninParams {
    username: String,
    password: String,
}

#[handler]
pub async fn view_entry_handle(Form(params): Form<SigninParams>, session: &Session, state: Data<&Arc<Mutex<WebApp>>>) -> impl IntoResponse {
    state.0.lock().await.test();
    if (params.username == "test" || params.username == "test2" ) && params.password == "123456" {
        session.set("username", params.username);
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
}

#[handler]
fn index(session: &Session) -> impl IntoResponse {
    //println!("session empty: {}", session.is_empty());
    match session.get::<String>("username") {
        Some(username) => Html(format!(
            r#"
    <!DOCTYPE html>
    <html>
    <head><meta charset="UTF-8"><title>Example Session Auth</title></head>
    <body>
    <div>hello {username}, you are viewing a restricted resource</div>
    <a href="/logout">click here to logout</a>
    <a href="/key/generate">click here to generate the key</a>
    <a href="/key/sign">click here to sign message</a>
    </body>
    </html>
    "#
        ))
        .into_response(),
        None => Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/login")
            .finish(),
    }
}


#[handler]
fn logout(session: &Session) -> impl IntoResponse {
    session.purge();
    println!("loggedout");
    println!("session empty: {}", session.is_empty());
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/")
        .finish()
}



impl WebApp {

    pub fn new() -> Self {
        Self { value: String::from("ASD")}
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
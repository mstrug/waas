use poem::{
    get, handler,
    http::{header, StatusCode},
    listener::TcpListener,
    session::{CookieConfig, CookieSession, Session},
    web::{Form, Html},
    EndpointExt, IntoResponse, Response, Result, Route, Server,
};
use serde::Deserialize;


#[handler]
pub fn view_entry() -> impl IntoResponse {
    Html(
        r#"
    <!DOCTYPE html>
    <html>
    <head><meta charset="UTF-8"><title>Example Session Auth</title></head>
    <body>
    <form action="/login" method="post">
        <div>
            <label>Username:<input type="text" name="username" value="test" /></label>
        </div>
        <div>
            <label>Password:<input type="password" name="password" value="123456" /></label>
        </div>
        <button type="submit">Login</button>
    </form>
    </body>
    </html>
    "#,
    )
}






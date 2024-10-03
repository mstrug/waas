use poem::{
    get, handler, listener::TcpListener, middleware::Tracing, web::Path, EndpointExt, Route, Server,
};


#[handler]
fn view_entry_show() -> String {
    format!("view entry")
}



use std::sync::Mutex;
use app::App;

pub mod app;
pub mod http;
pub mod http1;
pub mod websocket;
pub mod context;
pub mod utils;

pub extern crate dc_macro;

pub fn spawn_server (app: &'static Mutex<App>) {
    app::server::start_server(app);
}

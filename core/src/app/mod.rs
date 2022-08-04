use std::io::Error;
use crate::websocket::WebSocketEndpoints;
use self::router::Router;

pub mod server;
pub mod router;

pub struct Settings {
    pub bind_address: &'static str,
	pub debug: bool
}

pub trait AppHandler: Send + Sync + 'static {
	fn on_listen (_error: Option<Error>) {}
	fn on_stop () {}
}

pub struct App {
	pub settings: Settings,
	pub ws_endpoints: WebSocketEndpoints,
	pub router: Router
}

impl App {
	pub fn new (settings: Settings) -> Self {
		App {
			settings,
			ws_endpoints: WebSocketEndpoints::empty(),
			router: Router::empty()
		}
	}
}

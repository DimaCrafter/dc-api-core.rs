use crate::websocket::WebSocketEndpoints;
use self::router::Router;

pub mod config;
pub mod server;
pub mod router;

pub struct App {
	pub ws_endpoints: WebSocketEndpoints,
	pub router: Router
}

impl App {
	pub fn new () -> Self {
		App {
			ws_endpoints: WebSocketEndpoints::empty(),
			router: Router::empty()
		}
	}
}

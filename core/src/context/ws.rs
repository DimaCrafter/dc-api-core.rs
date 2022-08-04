use std::{collections::HashMap, net::TcpStream};
use bufstream::BufStream;
use tungstenite::{WebSocket, protocol::Role};
use crate::http::entity::{Request, HttpConnection};
use super::http::HttpContext;

pub struct SocketContext {
	pub http: HttpContext,
	pub stream: WebSocket<BufStream<TcpStream>>
}

impl SocketContext {
	pub fn from<Connection: HttpConnection> (connection: Connection, req: Request) -> Self {
		let stream = connection.into_stream();
		let ws_stream = WebSocket::from_raw_socket(stream, Role::Server, None);

		SocketContext {
			http: HttpContext::from(req, HashMap::new()),
			stream: ws_stream
		}
	}

	pub fn text (&mut self, event: &str, message: &str) {
		let mut payload = event.to_string();
		payload += ":";
		payload += "[\"";
		payload += &message.replace("\"", "\\\"");
		payload += "\"]";

		// todo: report write error?
		let _ = self.stream.write_message(tungstenite::Message::text(payload));
	}
}

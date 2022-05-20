pub mod codes;
pub mod entity;

use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use crate::get_app;
use crate::http::{entity::*, codes::HttpCode};
use crate::websocket::{v5_dispatch_websocket, v5_maintain_websocket};

pub struct ParsedHttpConnection {
    inner: Box<dyn HttpConnection>,
    pub req: Option<Request>
}

impl Deref for ParsedHttpConnection {
    type Target = dyn HttpConnection;
    fn deref (&self) -> &Self::Target { self.inner.deref() }
}

impl DerefMut for ParsedHttpConnection {
    fn deref_mut(&mut self) -> &mut Self::Target { self.inner.deref_mut() }
}

impl ParsedHttpConnection {
    pub fn new (connection: Box<dyn HttpConnection>, req: Request) -> Self {
        ParsedHttpConnection { inner: connection, req: Some(req) }
    }

    pub fn into_stream (self) -> BufStream<TcpStream> {
        self.inner.into_stream()
    }
}

pub async fn proceed_connection<Http: HttpEngine + Send> (socket: (TcpStream, SocketAddr)) {
    let mut connection = Http::handle_connection(socket).await;

    match connection.parse().await {
        ParsingResult::Complete(req) => {
            if is_connection_upgrade(&req) {
                if is_websocket_upgrade(&req) {
                    // let mut parsed_connection = ParsedHttpConnection::new(connection, req);
                    match v5_dispatch_websocket(&req) {
                        Ok((endpoint, res)) => {
                            // ! TODO: HANDLE THIS RESULT !
                            connection.respond_js(res).await;
                            v5_maintain_websocket(connection, req, endpoint).await;
                        }
                        Err(res) => {
                            connection.respond_js(res).await;
                        }
                    }
                    // get_app().dispatch_websocket.call(Ok(ParsedHttpConnection::new(connection, req)), ThreadsafeFunctionCallMode::NonBlocking);
                } else {
                    let _ = connection.respond(Response::from_status(HttpCode::BadRequest)).await;
                }
            } else {
                get_app().dispatch_request.call(Ok(ParsedHttpConnection::new(connection, req)), ThreadsafeFunctionCallMode::NonBlocking);
            }
        }
        ParsingResult::Partial => {}
        ParsingResult::Error(res_code) => {
            let _ = connection.respond(Response::from_status(res_code)).await;
        }
        ParsingResult::Invalid => {
            connection.disconnect().await;
        }
    }
}

fn is_connection_upgrade (req: &Request) -> bool {
    matches!(req.headers.get("connection"), Some(value) if value == "Upgrade")
}

fn is_websocket_upgrade (req: &Request) -> bool {
    matches!(req.headers.get("upgrade"), Some(value) if value == "websocket")
}

pub mod codes;
pub mod entity;

use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use tokio::net::TcpStream;
use crate::get_app;
use crate::http::entity::*;

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
}

pub async fn proceed_connection<Http: HttpEngine + Send> (socket: (TcpStream, SocketAddr)) {
    let mut connection = Http::handle_connection(socket).await;

    match connection.parse().await {
        ParsingResult::Complete(req) => {
            {
                let app = get_app();
                app.dispatch_request.call(Ok(ParsedHttpConnection::new(connection, req)), ThreadsafeFunctionCallMode::NonBlocking);
            }
        }
        ParsingResult::Partial => {}
        ParsingResult::Error(res_code) => {
            connection.respond(Response {
                code: res_code,
                headers: HttpHeaders::empty(),
                payload: None
            }).await;
        }
        ParsingResult::Invalid => {
             connection.disconnect().await;
        }
    }
}

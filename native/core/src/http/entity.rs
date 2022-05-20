use core::slice;
use std::io::Error;
use std::net::{SocketAddr, IpAddr};
use futures::TryFutureExt;
use tokio::io::BufStream;
use tokio::net::TcpStream;
use async_trait::async_trait;
use crate::http::codes::HttpCode;

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    POST,
    OPTIONS
}

impl HttpMethod {
    pub fn from_str (method: &str) -> Option<HttpMethod> {
        match method {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            _ => None
        }
    }
}

// TODO: &str
#[derive(Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String
}

#[derive(Debug)]
pub struct HttpHeaders {
    contents: Vec<HttpHeader>
}

impl HttpHeaders {
    pub fn empty () -> Self {
        HttpHeaders {
            contents: Vec::new()
        }
    }

    pub fn with_type (content_type: &str) -> Self {
        HttpHeaders {
            contents: vec![HttpHeader {
                name: "content-type".to_string(),
                value: content_type.to_string()
            }]
        }
    }

    pub fn push (&mut self, name: String, value: String) {
        let lower_name = name.to_ascii_lowercase();
        if let Some(header) = self.contents.iter_mut().find(|h| h.name == lower_name) {
            header.value = value.trim_start().to_string();
        } else {
            self.contents.push(HttpHeader {
                name: lower_name,
                value: value.trim_start().to_string()
            });
        }
    }

    pub fn push_default (&mut self, name: String, value: String) {
        let lower_name = name.to_ascii_lowercase();
        if let None = self.contents.iter_mut().find(|h| h.name == lower_name) {
            self.contents.push(HttpHeader { name: lower_name, value });
        }
    }

    pub fn remove (&mut self, name: String) {
        let lower_name = name.to_ascii_lowercase();
        if let Some(i) = self.contents.iter().position(|h| h.name == lower_name) {
            self.contents.remove(i);
        }
    }

    pub fn get (&self, name: &str) -> Option<String> {
        let lower_name = name.to_ascii_lowercase();
        for header in self {
            if header.name == lower_name {
                return Some(header.value.clone())
            }
        }

        return None;
    }
}

impl<'a> IntoIterator for &'a HttpHeaders {
    type Item = &'a HttpHeader;
    type IntoIter = slice::Iter<'a, HttpHeader>;

    fn into_iter (self) -> Self::IntoIter {
        return self.contents.iter();
    }
}

#[async_trait]
pub trait HttpConnection: Send + Sync {
    fn get_address (&self) -> IpAddr;
    fn into_stream (self: Box<Self>) -> BufStream<TcpStream>;

    async fn parse (&mut self) -> ParsingResult;
    async fn respond (&mut self, res: Response) -> Result<(), Error>;
    async fn disconnect (&mut self);
}

// TODO: DRY
impl dyn HttpConnection {
    pub async fn respond_js (&mut self, res: Response) -> napi::Result<()> {
        return self
            .respond(res)
            .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err.to_string()))
            .await;
    }
}
impl dyn HttpConnection + Send {
    pub async fn respond_js (&mut self, res: Response) -> napi::Result<()> {
        return self
            .respond(res)
            .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err.to_string()))
            .await;
    }
}

pub type BoxedHttpConnection = Box<dyn HttpConnection + Send>;

#[async_trait]
pub trait HttpEngine {
    async fn handle_connection (socket: (TcpStream, SocketAddr)) -> BoxedHttpConnection;
}

pub enum ParsingResult {
    Complete(Request),
    Partial,
    Error(HttpCode),
    Invalid
}

pub struct Response {
    pub code: HttpCode,
    pub headers: HttpHeaders,
    pub payload: ResponseType
}

impl Response {
    pub fn from_error (err: napi::Error) -> Self {
        Response {
            code: HttpCode::InternalServerError,
            headers: HttpHeaders::with_type("text/plain"),
            payload: ResponseType::Payload(err.to_string().as_bytes().to_vec())
        }
    }

    pub fn from_code (code: HttpCode, message: &str) -> Self {
        Response {
            code,
            headers: HttpHeaders::with_type("text/plain"),
            payload: ResponseType::Payload(message.as_bytes().to_vec())
        }
    }

    pub fn from_status (code: HttpCode) -> Self {
        Response {
            code,
            headers: HttpHeaders::empty(),
            payload: ResponseType::NoContent
        }
    }
}

pub struct Request {
    pub path: String,
    pub query: String,
    pub headers: HttpHeaders,
    pub method: HttpMethod,
}

impl Request {
    pub fn new (method: HttpMethod, url_path: String) -> Self {
        let (path, query) = match url_path.split_once('?') {
            Some((path, query_string)) => (path.to_string(), query_string.to_string()),
            None => (url_path, String::new())
        };

        Self {
            path,
            query,
            method,
            headers: HttpHeaders::empty()
        }
    }
}

pub enum ResponseType {
    NoContent,
    Payload(Vec<u8>),
    Upgrade,
    Drop
}

use core::slice;
use std::io::Error;
use std::net::{SocketAddr, IpAddr};
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
    pub fn from_str (method: String) -> Option<HttpMethod> {
        match method.as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            _ => None
        }
    }
}

pub struct HttpHeader {
    pub name: String,
    pub value: String
}

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
            header.value = value;
        } else {
            self.contents.push(HttpHeader { name: lower_name, value });
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
pub trait HttpEngine {
    async fn handle_connection (socket: (TcpStream, SocketAddr)) -> Box<dyn HttpConnection + Send>;
}

pub enum ParsingResult {
    Complete(Request),
    Partial,
    Error(HttpCode),
    Invalid
}

#[async_trait]
pub trait HttpConnection: Send + Sync {
    fn get_address (&self) -> IpAddr;
    async fn parse (&mut self) -> ParsingResult;
    async fn respond (&mut self, res: Response) -> Result<(), Error>;
    async fn disconnect (&mut self);
}

pub struct Response {
    pub code: HttpCode,
    pub headers: HttpHeaders,
    pub payload: Option<Vec<u8>>
}

impl Response {
    pub fn from_error (err: napi::Error) -> Self {
        Response {
            code: HttpCode::InternalServerError,
            headers: HttpHeaders::with_type("text/plain"),
            payload: Some(err.to_string().as_bytes().to_vec())
        }
    }

    pub fn from_code (code: HttpCode, message: &str) -> Self {
        Response {
            code,
            headers: HttpHeaders::with_type("text/plain"),
            payload: Some(message.as_bytes().to_vec())
        }
    }
}

pub struct Request {
    pub path: String,
    pub headers: HttpHeaders,
    pub method: HttpMethod,
}

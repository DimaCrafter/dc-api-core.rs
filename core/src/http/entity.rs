use core::slice;
use std::io::Error;
use std::net::{SocketAddr, IpAddr, TcpStream};
use bufstream::BufStream;
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

    pub fn from_type (content_type: &str) -> Self {
        HttpHeaders::empty().with_type(content_type)
    }

    pub fn with_type (mut self, content_type: &str) -> Self {
        self.set_default("content-type".to_string(), content_type.to_string());
        return self;
    }

    pub fn set (&mut self, name: String, value: String) {
        if let Some(header) = self.contents.iter_mut().find(|h| h.name == name) {
            header.value = value.trim_start().to_string();
        } else {
            self.contents.push(HttpHeader { name, value });
        }
    }

    pub fn set_normal (&mut self, name: String, value: String) {
        self.set(name.to_ascii_lowercase(), value.trim_start().to_string());
    }

    pub fn set_default (&mut self, name: String, value: String) {
        if let None = self.contents.iter_mut().find(|h| h.name == name) {
            self.contents.push(HttpHeader { name, value });
        }
    }

    pub fn remove (&mut self, name: String) {
        let lower_name = name.to_ascii_lowercase();
        if let Some(i) = self.contents.iter().position(|h| h.name == lower_name) {
            self.contents.remove(i);
        }
    }

    pub fn get (&self, name: &str) -> Option<String> {
        for header in self {
            if header.name == name {
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

pub trait HttpConnection: Sized + Send + Sync {
    fn get_address (&self) -> IpAddr;
    fn into_stream (self) -> BufStream<TcpStream>;

    fn parse (&mut self) -> ParsingResult;
    fn respond (&mut self, res: Response) -> Result<(), Error>;
    fn disconnect (self) -> Result<(), Error>;
}

pub type BoxedHttpConnection = Box<dyn HttpConnection + Send>;

pub trait HttpEngine<Connection: HttpConnection> {
    fn handle_connection (socket: (TcpStream, SocketAddr)) -> Connection;
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
    pub fn from_code (code: HttpCode, message: &str) -> Self {
        Response {
            code,
            headers: HttpHeaders::from_type("text/plain"),
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

    pub fn drop () -> Self {
        Response {
            code: HttpCode::GatewayTimeout,
            headers: HttpHeaders::empty(),
            payload: ResponseType::Drop
        }
    }
}

#[derive(Debug)]
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

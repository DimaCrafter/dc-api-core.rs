use std::{collections::HashMap, net::IpAddr};
use crate::http::{entity::{HttpHeaders, Request, Response, ResponseType, HttpConnection}, codes::HttpCode};

#[derive(Debug)]
pub struct HttpContext {
	pub req: Request,
	pub params: HashMap<String, String>,
	pub address: IpAddr,
	pub res_headers: HttpHeaders
}

impl HttpContext {
	pub fn from<Connection: HttpConnection> (connection: &Connection, req: Request, params: HashMap<String, String>) -> Self {
		HttpContext {
			req,
			params,
			address: connection.get_address(),
			res_headers: HttpHeaders::empty()
		}
	}

	#[inline]
	pub fn get_header (&self, name: &str) -> Option<String> {
		return self.req.headers.get(name);
	}

	#[inline]
	pub fn get_header_default (&self, name: &str, default: String) -> String {
		return self.get_header(name).unwrap_or(default);
	}

	#[inline]
	pub fn set_header (&mut self, name: &str, value: String) {
		self.res_headers.set(name.to_string(), value);
	}

	pub fn text (self, message: &str) -> Response {
		Response {
			code: HttpCode::OK,
			headers: self.res_headers.with_type("text/plain"),
			payload: ResponseType::Payload(message.into())
		}
	}

	pub fn text_status (self, message: &str, code: HttpCode) -> Response {
		Response {
			code,
			headers: self.res_headers.with_type("text/plain"),
			payload: ResponseType::Payload(message.into())
		}
	}

	pub fn redirect (mut self, target: &str) -> Response {
		self.res_headers.set("location".to_string(), target.to_string());

		Response {
			code: HttpCode::TemporaryRedirect,
			headers: self.res_headers,
			payload: ResponseType::NoContent
		}
	}

	pub fn drop (self) -> Response {
		Response::drop()
	}
}

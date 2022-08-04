use std::{collections::HashMap};
use crate::http::{entity::{HttpHeaders, Request, Response, ResponseType}, codes::HttpCode};

pub struct HttpContext {
	pub req: Request,
	pub params: HashMap<String, String>
}

impl HttpContext {
	pub fn from (req: Request, params: HashMap<String, String>) -> Self {
		HttpContext { req, params }
	}

	pub fn text (self, message: &str) -> Response {
		Response {
			code: HttpCode::OK,
			headers: HttpHeaders::with_type("text/plain"),
			payload: ResponseType::Payload(message.into())
		}
	}

	pub fn text_status (self, message: &str, code: HttpCode) -> Response {
		Response {
			code,
			headers: HttpHeaders::with_type("text/plain"),
			payload: ResponseType::Payload(message.into())
		}
	}
}

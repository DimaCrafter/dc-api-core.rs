use std::process;
use json::JsonValue;
use crate::{app::config::Config, utils::{json_read_array, log::log_error}};
use super::entity::{Request, Response};

static mut CORS: Option<CorsConfig> = None;
pub struct CorsConfig {
	origin: String,
	methods: String,
	headers: String,
	ttl: String
}

impl CorsConfig {
	fn init () -> &'static mut CorsConfig {
		let config = Config::branch("cors");
		let origin = config["origin"].as_str().unwrap_or("").to_string();

		fn bake_error<'a> (msg: &'a str) -> impl (Fn() -> &'static str) + 'a {
			return move || {
				log_error(&format!("Config parsing error: {}", msg));
				process::exit(-1);
			};
		}

		let methods = json_read_array(
			&config["methods"],
			JsonValue::as_str,
			bake_error("cors.methods[...] must be a string")
		)
		.map(|list| list.join(","))
		.unwrap_or_else(|| "GET,POST".to_string());

		let headers = json_read_array(
			&config["headers"],
			JsonValue::as_str,
			bake_error("cors.headers[...] must be a string")
		)
		.map(|list| list.join(","))
		.unwrap_or_else(|| "content-type,session".to_string());

		let ttl = config["ttl"].as_u32().map(|v| v.to_string()).unwrap_or("86400".to_string());

		let policy = CorsConfig { origin, methods, headers, ttl };
		return unsafe { CORS.insert(policy) };
	}

	pub fn get () -> &'static mut Self {
		if let Some(policy) = unsafe { CORS.as_mut() } {
			return policy;
		} else {
			return Self::init();
		}
	}
}

pub struct Cors {
	origin: String
}

impl Cors {
	pub fn new (req: &Request) -> Self {
		Cors {
			origin: req.headers.get("origin").unwrap_or("*".to_string())
		}
	}

	pub fn apply_origin_check (self, res: &mut Response, default: &String) {
		if self.origin.is_empty() {
			res.headers.set("Access-Control-Allow-Origin".to_string(), default.clone());
		} else {
			res.headers.set("Access-Control-Allow-Origin".to_string(), self.origin);
		}
	}

	pub fn apply_normal (self, res: &mut Response) {
		let policy = CorsConfig::get();
		res.headers.set("Access-Control-Expose-Headers".to_string(), policy.headers.clone());
		self.apply_origin_check(res, &policy.origin);
	}

	pub fn apply_preflight (self, res: &mut Response) {
		let policy = CorsConfig::get();
		res.headers.set("Access-Control-Allow-Methods".to_string(), policy.methods.clone());
		res.headers.set("Access-Control-Allow-Headers".to_string(), policy.headers.clone());
		res.headers.set("Access-Control-Max-Age".to_string(), policy.ttl.clone());
		self.apply_origin_check(res, &policy.origin);
	}
}

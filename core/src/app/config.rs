use std::{fs, process, env};
use json::JsonValue;
use crate::utils::{log::log_error_lines, json_access};

static mut CONFIG: Option<Config> = None;
pub struct Config {
	obj: JsonValue,

	pub host: String,
	pub port: u16
}

impl Config {
	fn init () -> &'static mut Self {
		let raw = match fs::read_to_string("config.json") {
			Ok(content) => content,
			Err(err) => {
				log_error_lines("Config reading error", err.to_string());
				process::exit(-1);
			}
		};

		let obj = match json::parse(&raw) {
			Ok(value) => value,
			Err(err) => {
				log_error_lines("Config parsing error", err.to_string());
				process::exit(-1);
			}
		};

		let port = {
			let mut env = None;
			if let Ok(env_port_str) = env::var("PORT") {
				if let Ok(env_port) = env_port_str.parse::<u16>() {
					env = Some(env_port);
				}
			}

			if let Some(port) = env {
				port
			} else {
				let port = &obj["port"];
				port.as_u16().unwrap_or(8081)
			}
		};

		let host = &obj["host"];
		let host = host.as_str().unwrap_or("0.0.0.0").to_string();

		let config = Config { obj, host, port };
		return unsafe { CONFIG.insert(config) };
	}

	pub fn get () -> &'static mut Self {
		if let Some(config) = unsafe { CONFIG.as_mut() } {
			return config;
		} else {
			return Self::init();
		}
	}
}

#[inline]
pub fn config_path (path: &str) -> &mut JsonValue {
	return json_access(&mut Config::get().obj, path);
}

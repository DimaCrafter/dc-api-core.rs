use std::env;

const RESET: &'static str = "\x1B[0m";
const BOLD: &'static str = "\x1B[1m";
static mut THEME: Option<ShellTheme> = None;
struct ShellTheme {
	get: fn (ShellColor, u8) -> String
}

enum ShellColor { Info, Ok, Warning, Error }

// todo: simplify & add caching (?)
impl ShellTheme {
	fn init () -> &'static Self {
		let theme;
		if let Some(palette) = env::var_os("COLORTERM") {
			if palette == "truecolor" || palette == "x24" {
				theme = Self::rgb();
			} else if palette == "256color" {
				theme = Self::named();
			} else {
				theme = Self::ansi();
			}
		} else {
			theme = Self::ansi();
		}

		return unsafe { THEME.insert(theme) };
	}

	fn rgb () -> Self {
		ShellTheme {
			get: |color, offset| {
				let code = match color {
					ShellColor::Info => "0;192;25",
					ShellColor::Ok => "0;192;64",
					ShellColor::Warning => "255;112;0",
					ShellColor::Error => "224;0;0"
				};

				return format!("\x1B[{};2;{}m", offset + 8, code);
			}
		}
	}

	fn named () -> Self {
		ShellTheme {
			get: |color, offset| {
				let code = match color {
					ShellColor::Info => "39",
					ShellColor::Ok => "35",
					ShellColor::Warning => "202",
					ShellColor::Error => "160"
				};

				return format!("\x1B[{};5;{}m", offset + 8, code);
			}
		}
	}

	fn ansi () -> Self {
		ShellTheme {
			get: |color, offset| {
				let code = match color {
					ShellColor::Info => 6,
					ShellColor::Ok => 2,
					ShellColor::Warning => 3,
					ShellColor::Error => 1
				};

				return format!("\x1B[{}m", offset + code);
			}
		}
	}

	pub fn current () -> &'static Self {
		if let Some(theme) = unsafe { THEME.as_ref() } {
			return theme;
		} else {
			return Self::init();
		}
	}

	pub fn get (color: ShellColor, is_bg: bool) -> String {
		let theme = Self::current();
		return (theme.get)(color, if is_bg { 40 } else { 30 });
	}
}

pub fn log_info (msg: &str) {
	println!("{}{} INFO {} {}", ShellTheme::get(ShellColor::Info, true), BOLD, RESET, msg);
}

pub fn log_success (msg: &str) {
	println!("{}{} OK {} {}", ShellTheme::get(ShellColor::Ok, true), BOLD, RESET, msg);
}

pub fn log_warning (msg: &str) {
	println!("{}{} WARN {} {}", ShellTheme::get(ShellColor::Warning, true), BOLD, RESET, msg);
}

pub fn log_error (msg: &str) {
	println!("{}{} ERR {} {}", ShellTheme::get(ShellColor::Error, true), BOLD, RESET, msg);
}

pub fn log_error_lines (msg: &str, lines: String) {
	log_error(msg);
	for line in lines.split('\n') {
		println!(" {}│{} {}", ShellTheme::get(ShellColor::Error, false), RESET, line);
	}

	println!(" {}└─{}", ShellTheme::get(ShellColor::Error, false), RESET);
}

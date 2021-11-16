use std::path::PathBuf;

// replace this with spwn's error system

pub enum PckpError {
	CustomError {
		message: String,
		note: Option<String>,
		in_package: Option<String>
	},
	ConfigError {
		message: String,
		note: Option<String>,
		pos: Option<(usize, usize)>,
		file: PathBuf
	}
}


impl PckpError {
	pub fn custom_with_note(message: String, in_package: Option<String>, note: Option<String>) -> PckpError {
		PckpError::CustomError { message, note, in_package }
	}
	pub fn custom(message: String, in_package: Option<String>) -> PckpError {
		PckpError::CustomError { message, note: None, in_package }
	}
	pub fn config(message: String, file: PathBuf, pos: Option<(usize, usize)>) -> PckpError {
		PckpError::ConfigError { message, note: None, pos, file }
	}
	pub fn config_with_note(message: String, file: PathBuf, pos: Option<(usize, usize)>, note:Option<String>) -> PckpError {
		PckpError::ConfigError { message, note, pos, file }
	}
	pub fn to_string(&self) -> String {
		match self {
			PckpError::CustomError { message, note, in_package } => {
				let ipkg = if let Some(pkg) = &in_package {
					format!(" in package '{}'", pkg)
				} else {
					String::new()
				};

				let note = if let Some(n) = &note {
					format!("\nNote: {}", n)
				} else {
					String::new()
				};

				return format!("PckpError{}: {}{}", ipkg, message, note);
			},
			PckpError::ConfigError {message, note, pos, file} => {
				let note = if let Some(n) = &note {
					format!("\nNote: {}", n)
				} else {
					String::new()
				};

				let ps = if let Some(p) = &pos {
					format!(" on line {} column {}", p.0, p.1)
				} else {
					String::new()
				};

				return format!("PckpConfigError: {}\nError located in file {}{}{}", message, file.clone().into_os_string().into_string().unwrap(), ps, note);
			}
		}
	}
}
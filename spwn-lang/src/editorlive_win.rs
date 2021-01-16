use named_pipe::PipeClient;
use std::ffi::OsStr;
use std::time::Duration;
use std::io::Write;

pub fn editor_paste(message: &str) -> Result<bool, String> {
	let data = message.as_bytes();
	let pipe_name = OsStr::new("\\\\.\\pipe\\GDPipe");

	match PipeClient::connect_ms(pipe_name, 5) {
		Ok(mut client) => {
			client.set_write_timeout(Some(Duration::new(0,5)));
			match client.write(data) {
				Ok(_) => (),
				Err(e) => {
					return Err(format!("Could not send a message to GD with this error: {:?}", e));
				}
			};
			Ok(true)
		}
		Err(e) => Err("Could not make a connection to GD, try injecting the live editor library into geometry dash".to_string())
	}
}
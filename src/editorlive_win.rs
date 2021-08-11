use named_pipe::PipeClient;
use std::ffi::OsStr;
use std::io::Write;
use std::time::Duration;

pub fn editor_paste(message: &str) -> Result<bool, String> {
    let pipe_name = OsStr::new("\\\\.\\pipe\\GDPipe");

    match PipeClient::connect_ms(pipe_name, 5) {
		Ok(mut client) => {
			client.set_write_timeout(Some(Duration::new(1,0)));
			let split = message.split(';').collect::<Vec<&str>>();
			for iter in split.chunks(2) {
				let mut data = iter.join(";").to_string();

				if data.ends_with(';') {
					data.pop();
				}
				match client.write(format!("{};",data).as_bytes()) {
					Ok(_) => (),
					Err(e) => {
						return Err(format!("Could not send a message to GD with this error: {:?}", e));
					}
				};
			}
			Ok(true)
		}
		Err(_) => Err("Could not make a connection to GD, try injecting the live editor library into geometry dash".to_string())
	}
}

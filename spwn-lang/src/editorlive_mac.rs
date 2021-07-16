extern crate libc;

use std::ptr::null;

use core_foundation_sys::data as cfd;
use core_foundation_sys::messageport as mp;
use core_foundation_sys::string as cfst;

fn new_port() -> Result<mp::CFMessagePortRef, String> {
    unsafe {
        let cfstr = cfst::CFStringCreateWithCString(
            null(),
            "314GDL\0".as_ptr() as *const libc::c_char,
            cfst::kCFStringEncodingUTF8,
        );
        let port = mp::CFMessagePortCreateRemote(null(), cfstr);
        if port.is_null() || mp::CFMessagePortIsValid(port) == 0 {
            return Err("Could not make a connection to GD".to_string());
        }
        Ok(port)
    }
}
fn create_data(value: &str) -> Result<cfd::CFDataRef, String> {
    unsafe {
        let cdr = cfd::CFDataCreate(null(), value.as_ptr() as *const u8, value.len() as isize);
        if cdr.is_null() {
            return Err("Could not create data".to_string());
        }
        Ok(cdr)
    }
}
pub fn editor_paste(message: &str) -> Result<bool, String> {
    unsafe {
        let data = create_data(message);
        let port = new_port();
        match (data.clone(), port) {
            (Err(a), _) | (_, Err(a)) => {
                return Err(a);
            }
            _ => {
                let port = new_port().unwrap();
                let result = mp::CFMessagePortSendRequest(
                    port,
                    0x1,
                    data.unwrap(),
                    1.0,
                    10.0,
                    null(),
                    &mut create_data("").unwrap(),
                );
                if result != 0 {
                    return Err(format!("Could not send message, error code {}", result));
                }

                Ok(true)
            }
        }
    }
}

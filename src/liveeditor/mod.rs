use std::io::ErrorKind;

use delve::EnumDisplay;
use serde::Deserialize;
use thiserror::Error;

use crate::util::hyperlink;

#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "windows")]
const PORT: u16 = 1313;

#[derive(EnumDisplay, Clone)]
pub enum Message<'a> {
    AddObjects(&'a str),
    RemoveObjectsByGroup(usize),
}

// TODO(future): make this less manual in the future if there are ever more options
impl<'a> From<Message<'a>> for String {
    fn from(m: Message<'a>) -> Self {
        match m {
            Message::RemoveObjectsByGroup(group) => {
                format!(r#"{{"type": "REMOVE_OBJECTS_GROUP", "value": {group}}}"#)
            },
            Message::AddObjects(ls) => {
                format!(r#"{{"type": "ADD_OBJECTS_STRING", "value": "{ls}"}}"#)
            },
        }
    }
}

#[derive(Deserialize)]
pub struct LiveEditorResult {
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error(
        "Failed to connect to the Live Editor server ({}). Ensure that the Live Editor mod is installed, the editor is currently open, and there are no other active connections. ({})",
        .0,
        hyperlink::<_, &str>("https://github.com/iAndyHD3/WSLiveEditor", None)
    )]
    FailedToConnect(ErrorKind),

    #[error("Live Editor server returned an invalid json response.")]
    InvalidJsonResult,

    #[error("Live Editor server returned an error{}", match .0 {
        Some(s) => format!(": {s}"),
        None => ".".to_string()
    })]
    LiveEditorError(Option<String>),

    #[error("{}", .0)]
    Other(websocket::WebSocketError),
}

impl From<websocket::WebSocketError> for WebSocketError {
    fn from(err: websocket::WebSocketError) -> Self {
        match err {
            websocket::WebSocketError::IoError(ioerr)
                if matches!(
                    ioerr.kind(),
                    ErrorKind::AddrNotAvailable
                        | ErrorKind::AddrInUse
                        | ErrorKind::ConnectionRefused
                ) =>
            {
                Self::FailedToConnect(ioerr.kind())
            },
            // TODO(future): make more common ws errors clearer
            _ => Self::Other(err),
        }
    }
}

use std::io::ErrorKind;

use delve::EnumDisplay;
use serde::Deserialize;

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

// TODO: make this less manual in the future if there are ever more options
impl<'a> From<Message<'a>> for String {
    fn from(m: Message<'a>) -> Self {
        match m {
            Message::RemoveObjectsByGroup(group) => {
                format!(r#"{{"type": "remove_objects", "value": "{group}", "filter": "group"}}"#)
            },
            Message::AddObjects(ls) => {
                format!(r#"{{"type": "add_objects_string", "value": "{ls}"}}"#)
            },
        }
    }
}

#[derive(Deserialize)]
pub struct LiveEditorResult {
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(EnumDisplay)]
pub enum WebSocketError {
    #[delve(display = |kind| format!(
        "Failed to connect to the Live Editor server ({kind}). Make sure the live editor mod is installed and there is no other active connections. ({})",
        hyperlink::<_, &str>("https://github.com/iAndyHD3/WSLiveEditor", None)
    ))]
    FailedToConnect(ErrorKind),

    #[delve(display = "Live Editor server returned an invalid json response.")]
    InvalidJsonResult,

    #[delve(display = |o: &Option<String>| format!("Live Editor server returned an error{}", match o {
        Some(s) => format!(": {s}"),
        None => ".".to_string()
    }))]
    LiveEditorError(Option<String>),

    #[delve(display = |e| format!("{e}"))]
    Other(websocket::WebSocketError),
}

impl std::fmt::Debug for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
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
            // TODO: make more common ws errors clearer
            _ => Self::Other(err),
        }
    }
}

impl std::error::Error for WebSocketError {}

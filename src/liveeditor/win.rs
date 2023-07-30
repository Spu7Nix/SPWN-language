use std::net::TcpStream;

use websocket::sync::{Reader, Writer};
use websocket::{ClientBuilder, OwnedMessage};

use super::{LiveEditorResult, Message, WebSocketError, PORT};

pub struct LiveEditorClient {
    receiver: Reader<TcpStream>,
    sender: Writer<TcpStream>,
}

impl LiveEditorClient {
    fn try_wait_for_response(&mut self) -> Result<(), WebSocketError> {
        for message in self.receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => return Err(e.into()),
            };

            match message {
                OwnedMessage::Text(t) if &t == "hello there" => {},
                OwnedMessage::Text(t) => {
                    let r: LiveEditorResult =
                        serde_json::from_str(&t).map_err(|_| WebSocketError::InvalidJsonResult)?;

                    if !r.ok {
                        return Err(WebSocketError::LiveEditorError(r.error));
                    }

                    return Ok(());
                },
                _ => unreachable!("BUG: non-text message received"),
            }
        }

        Ok(())
    }

    pub fn try_create_client() -> Result<Self, WebSocketError> {
        let client = ClientBuilder::new(&format!("ws://127.0.0.1:{PORT}"))
            .expect("BUG: invalid websocket url")
            .connect_insecure()
            .map_err(<WebSocketError as From<websocket::WebSocketError>>::from)?;

        let (receiver, sender) = client.split().unwrap();

        Ok(LiveEditorClient { sender, receiver })
    }

    pub fn try_send_message(&mut self, message: Message) -> Result<(), WebSocketError> {
        self.sender
            .send_message(&OwnedMessage::Text(message.into()))
            .map_err(<WebSocketError as From<websocket::WebSocketError>>::from)?;

        self.try_wait_for_response()
    }
}

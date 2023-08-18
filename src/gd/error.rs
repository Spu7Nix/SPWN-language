use std::io;
use std::string::FromUtf8Error;

use base64::DecodeError;
use block_modes::BlockModeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LevelError {
    #[error("This level exceeds the {0} limit")]
    ExceedsIDLimit(&'static str),
    #[error("This level exceeds the {id} limit ({amount} / {max})")]
    ExceedsIDLimitByAmount {
        id: &'static str,
        max: u16,
        amount: usize,
    },

    #[error("Level is not initialized. Please open the level, place some objects, then save and quit to initialize the level")]
    UninitializedLevel,

    #[error(r#"Level "{0}" was not found"#)]
    UnknownLevel(String),
    #[error(r#"No levels found. Please create a new level for SPWN to add to."#)]
    NoLevels,

    #[error("Error reading XML at position {position} ({error})")]
    XMLErrorAtPosition {
        position: usize,
        error: quick_xml::Error,
    },

    #[error("Error decrypting save file ({0})")]
    AESDecryptionError(#[from] BlockModeError),
    #[error("Error decoding save file ({0})")]
    Base64DecodeError(#[from] DecodeError),

    #[error("Invalid UTF-8 in level string")]
    InvalidUTF8(#[from] FromUtf8Error),

    #[error("Error reading save file ({0})")]
    IoError(#[from] io::Error),
}

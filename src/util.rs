pub type RandomState = ahash::RandomState;
pub type Interner = lasso::Rodeo<lasso::Spur, RandomState>;

use colored::Colorize;
use serde::{Deserialize, Serialize};

pub fn hyperlink<T: ToString, U: ToString>(url: T, text: Option<U>) -> String {
    let text = match text {
        Some(t) => t.to_string(),
        None => url.to_string(),
    };

    format!("\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", url.to_string(), text)
        .blue()
        .underline()
        .bold()
        .to_string()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digest(#[serde(with = "hex_serde")] [u8; 16]);

impl From<md5::Digest> for Digest {
    fn from(value: md5::Digest) -> Self {
        Self(value.0)
    }
}

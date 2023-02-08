pub type RandomState = ahash::RandomState;
pub type Interner = lasso::Rodeo<lasso::Spur, RandomState>;

use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

pub fn hyperlink<T: ToString, U: ToString>(url: T, text: Option<U>) -> String {
    let mtext = match &text {
        Some(t) => t.to_string(),
        None => url.to_string(),
    };

    match std::env::var("NO_COLOR").ok() {
        Some(_) => {
            if text.is_some() {
                format!("[{}]({mtext})", url.to_string())
            } else {
                url.to_string()
            }
        }
        None => format!("\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", url.to_string(), mtext)
            .blue()
            .underline()
            .bold()
            .to_string(),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digest(#[serde(with = "hex_serde")] [u8; 16]);

impl From<md5::Digest> for Digest {
    fn from(value: md5::Digest) -> Self {
        Self(value.0)
    }
}

pub fn hex_to_rgb(hex: u32) -> (u8, u8, u8) {
    if hex > 0xffffff {
        panic!("invalid hex number")
    }

    (
        (hex >> 16) as u8,
        ((hex % 0x10000) >> 8) as u8,
        (hex % 0x100) as u8,
    )
}

pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());

    let (r, g, b) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = v - c;
    let (r, g, b) = (r + m, g + m, b + m);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub trait HexColorize {
    fn color_hex(self, c: u32) -> ColoredString;
    fn on_color_hex(self, c: u32) -> ColoredString;
}

impl<T: Colorize> HexColorize for T {
    fn color_hex(self, c: u32) -> ColoredString {
        let (r, g, b) = hex_to_rgb(c);
        self.truecolor(r, g, b)
    }

    fn on_color_hex(self, c: u32) -> ColoredString {
        let (r, g, b) = hex_to_rgb(c);
        self.on_truecolor(r, g, b)
    }
}

#[derive(Debug)]
pub struct BasicError(pub(crate) String);
impl std::error::Error for BasicError {}

impl std::fmt::Display for BasicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type RandomState = ahash::RandomState;
pub type Interner = lasso::Rodeo<lasso::Spur, RandomState>;

use colored::Colorize;
use serde::de::Visitor;

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

// gotta do this skrunkly stuff cause md5::Digest does not implement serde
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Digest(pub [u8; 16]);

impl From<md5::Digest> for Digest {
    fn from(value: md5::Digest) -> Self {
        Self(value.0)
    }
}

impl serde::Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

struct DigestVisitor;

impl<'de> serde::Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(DigestVisitor)
    }
}

impl<'de> Visitor<'de> for DigestVisitor {
    type Value = Digest;

    fn expecting(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
        panic!("idk")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() > 16 {
            panic!("digest too large")
        }

        // SAFETY:
        // this is safe since we check the length above (cant be more than 16)
        Ok(Digest(unsafe { *v.as_ptr().cast::<[u8; 16]>() }))
    }
}

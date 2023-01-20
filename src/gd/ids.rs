use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Hash)]
pub enum IDClass {
    Group = 0,
    Color = 1,
    Block = 2,
    Item = 3,
}

impl IDClass {
    pub fn letter(&self) -> &str {
        match self {
            IDClass::Group => "g",
            IDClass::Color => "c",
            IDClass::Block => "b",
            IDClass::Item => "i",
        }
    }
}

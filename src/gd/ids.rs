use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, Hash)]
pub enum IDClass {
    Group = 0,
    Color = 1,
    Block = 2,
    Item = 3,
}

impl IDClass {
    pub fn suffix(&self) -> &str {
        match self {
            IDClass::Group => "g",
            IDClass::Color => "c",
            IDClass::Block => "b",
            IDClass::Item => "i",
        }
    }
}

pub type ArbitraryId = u16;
pub type SpecificId = u16;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Id {
    Specific(SpecificId),
    Arbitrary(ArbitraryId), // will be given specific ids at the end of compilation
}

impl Id {
    pub fn new(id: SpecificId) -> Id {
        //creates new specific group
        Id::Specific(id)
    }

    pub fn next_free(counter: &mut ArbitraryId) -> Id {
        //creates new specific group
        (*counter) += 1;

        Id::Arbitrary(*counter)
    }

    pub fn fmt(&self, suffix: &'static str) -> String {
        match self {
            Id::Specific(n) => format!("{}{}", n, suffix),
            Id::Arbitrary(n) => format!("{}?{}", n, suffix),
        }
    }
}

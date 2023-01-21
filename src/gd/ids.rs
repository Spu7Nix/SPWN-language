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
}

// impl std::fmt::Debug for Id {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self.id {
//             Id::Specific(n) => f.write_str(&format!("{}{}", n, $short_name)),
//             Id::Arbitrary(n) => f.write_str(&format!("{}?{}", n, $short_name)),
//         }
//     }
// }

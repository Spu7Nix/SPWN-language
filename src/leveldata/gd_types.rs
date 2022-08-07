pub type ArbitraryId = u16;
pub type SpecificId = u16;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Id {
    Specific(SpecificId),
    Arbitrary(ArbitraryId),
}
impl Id {
    pub fn to_str(self) -> String {
        match self {
            Id::Specific(n) => n.to_string(),
            Id::Arbitrary(n) => format!("{}?", n),
        }
    }
}

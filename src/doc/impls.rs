use super::attributes::Doc;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Impl {
    pub doc: Option<Vec<Doc>>,
}

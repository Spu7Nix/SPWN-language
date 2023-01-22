use crate::gd::ids::Id;

pub struct Context {
    pub group: Id,
    pub pos: usize,
    pub func: usize,
}

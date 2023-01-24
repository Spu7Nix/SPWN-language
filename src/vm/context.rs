use std::cmp::Ordering;
use std::cmp::PartialOrd;

use crate::gd::ids::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Context {
    pub group: Id,
    pub pos: usize,
    pub recursion_depth: usize,
    pub func: usize,
}

// sort by pos, then by recursion depth
impl PartialOrd for Context {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.pos.cmp(&other.pos) {
            Ordering::Equal => self.recursion_depth.partial_cmp(&other.recursion_depth),
            x => Some(x),
        }
    }
}

impl Ord for Context {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos
            .cmp(&other.pos)
            .then(self.recursion_depth.cmp(&other.recursion_depth))
    }
}

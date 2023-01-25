use std::cmp::{Ordering, PartialOrd, Reverse};
use std::collections::BinaryHeap;

use super::interpreter::FuncCoord;
use crate::gd::ids::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub group: Id,
    pub pos: usize,
    pub recursion_depth: usize,
    pub func: FuncCoord,
}
// yore sot sitnky 😍😍😍😍😍😍😍😂😂😂😂😻😻😻😻😻❤️❤️❤️❤️❤️❤️😭💷💷💷💷💵💵🚘🚘🚘😉😉😉
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
        Reverse(self.pos)
            .cmp(&Reverse(other.pos))
            .then(Reverse(self.recursion_depth).cmp(&Reverse(other.recursion_depth)))
    }
}

/// all the contexts!!!
struct FullContext {
    // literally gonna use a binary heap!!! even though theres gonna be like max 4 of them!!
    // max my dick!
    contexts: BinaryHeap<Context>,
}

impl FullContext {
    fn current(&self) -> &Context {
        self.contexts.peek().unwrap()
    }

    fn increment_current(&mut self) {
        let mut current = self.contexts.peek_mut().unwrap();
        current.pos += 1;
    }
}

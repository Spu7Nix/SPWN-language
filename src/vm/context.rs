use std::cmp::{Ordering, PartialOrd};
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;

use slotmap::{new_key_type, SlotMap};

use super::interpreter::{FuncCoord, ValueKey};
use super::opcodes::Register;
use crate::gd::ids::Id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallStackItem {
    pub func: FuncCoord,
    pub ip: usize,
    pub return_dest: Register,
    pub call_key: CallKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub group: Id,
    pub recursion_depth: usize,
    pub memory: usize,

    pub pos_stack: Vec<CallStackItem>,

    pub registers: Vec<Vec<ValueKey>>,
}
// yore sot sitnky 😍😍😍😍😍😍😍😂😂😂😂😻😻😻😻😻❤️❤️❤️❤️❤️❤️😭💷💷💷💷💵💵🚘🚘🚘😉😉😉
// sort by pos, then by recursion depth
impl PartialOrd for Context {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Context {
    fn cmp(&self, other: &Self) -> Ordering {
        self.recursion_depth.cmp(&other.recursion_depth).then(
            self.pos_stack
                .last()
                .unwrap()
                .ip
                .cmp(&other.pos_stack.last().unwrap().ip)
                .reverse(),
        )
    }
}

new_key_type! {
    pub struct CallKey;
}

/// all the contexts!!!pub
#[derive(Debug)]
pub struct FullContext {
    contexts: BinaryHeap<Context>,
    pub have_not_returned: SlotMap<CallKey, ()>,
}

impl FullContext {
    pub fn new() -> Self {
        let mut contexts = BinaryHeap::new();
        contexts.push(Context {
            group: Id::Specific(0),
            recursion_depth: 0,
            memory: 0,
            registers: vec![],
            pos_stack: vec![],
        });
        Self {
            contexts,
            have_not_returned: SlotMap::default(),
        }
    }

    pub fn current(&self) -> &Context {
        self.contexts.peek().unwrap()
    }

    pub fn increment_current(&mut self, func_len: usize) {
        {
            let mut current = self.current_mut();
            let ip = &mut current.pos_stack.last_mut().unwrap().ip;
            *ip += 1;

            if *ip >= func_len {
                current.pos_stack.pop();
            }
        }
        if self.current().pos_stack.is_empty() {
            self.contexts.pop();
        }
    }

    pub fn jump_current(&mut self, pos: usize) {
        self.current_mut().pos_stack.last_mut().unwrap().ip = pos
    }

    pub fn current_mut(&mut self) -> PeekMut<Context> {
        self.contexts.peek_mut().unwrap()
    }

    pub fn ip(&self) -> usize {
        self.current().pos_stack.last().unwrap().ip
    }

    pub fn valid(&self) -> bool {
        !self.contexts.is_empty()
    }

    pub fn yeet_current(&mut self) {
        self.contexts.pop();
    }
}

impl<'a> super::interpreter::Vm<'a> {
    pub fn split_current_context(&mut self) {
        let current = self.contexts.current();
        let mut new = current.clone();

        // lord forgive me for what i am about to do
        for regs in &mut new.registers {
            for reg in regs {
                *reg = self.deep_clone_key_insert(*reg);
            }
        }
        self.contexts.contexts.push(new);
    }
}

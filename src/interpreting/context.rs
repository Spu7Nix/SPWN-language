use std::cmp::{Ordering, PartialOrd};
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;

use ahash::AHashMap;

use super::opcodes::Register;
use super::vm::{FuncCoord, ValueKey, Vm};
use crate::gd::ids::Id;
use crate::sources::CodeArea;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallInfo {
    pub func: FuncCoord,
    // is Some in all cases except assign operator overloading because bukny
    pub return_dest: Option<Register>,
    pub call_area: Option<CodeArea>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub ip: usize,

    pub group_stack: Vec<Id>,
    pub registers: Vec<Vec<ValueKey>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ip: 0,
            group_stack: vec![Id::Specific(0)],
            registers: vec![],
        }
    }

    pub fn hash(&self) -> u64 {
        0
    }
}

impl PartialOrd for Context {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Context {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ip.cmp(&other.ip).reverse()
    }
}

/// all the contexts!!!pub
#[derive(Debug)]
pub struct FullContext {
    pub contexts: BinaryHeap<Context>,

    pub call_info: CallInfo,
    pub have_returned: bool,
}

impl FullContext {
    pub fn new(initial: Context, call_info: CallInfo) -> Self {
        let mut contexts = BinaryHeap::new();
        contexts.push(initial);
        Self {
            contexts,
            have_returned: false,
            call_info,
        }
    }

    pub fn current(&self) -> &Context {
        self.contexts.peek().unwrap()
    }

    pub fn jump_current(&mut self, pos: usize) {
        self.current_mut().ip = pos
    }

    pub fn current_mut(&mut self) -> PeekMut<Context> {
        self.contexts.peek_mut().unwrap()
    }

    pub fn ip(&self) -> usize {
        self.current().ip
    }

    pub fn valid(&self) -> bool {
        !self.contexts.is_empty()
    }

    pub fn yeet_current(&mut self) -> Option<Context> {
        self.contexts.pop()
    }

    pub fn set_group_and_push(&mut self, group: Id) {
        let mut current = self.current_mut();
        current.group_stack.push(group);
    }

    pub fn pop_group(&mut self) -> Id {
        let mut current = self.current_mut();
        current.group_stack.pop().unwrap()
    }

    pub fn pop_groups_until(&mut self, group: Id) {
        let mut current = self.current_mut();
        while current.group_stack.pop().unwrap() != group {}
        current.group_stack.push(group);
    }

    pub fn group(&self) -> Id {
        *self.current().group_stack.last().unwrap()
    }
}

impl<'a> Vm<'a> {
    pub fn split_current_context(&mut self) {
        let current = self.contexts.current();
        let mut new = current.clone();

        // lord forgive me for what i am about to do

        let mut clone_map = AHashMap::new();

        for regs in &mut new.registers {
            for reg in regs {
                let k = match clone_map.get(reg) {
                    Some(k) => *k,
                    None => {
                        let k = self.deep_clone_key_insert(*reg);
                        clone_map.insert(*reg, k);
                        k
                    },
                };

                *reg = k;
            }
        }
        self.contexts.last_mut().contexts.push(new);
    }
}

#[derive(Debug)]
pub struct ContextStack(pub Vec<FullContext>);

impl ContextStack {
    pub fn last(&self) -> &FullContext {
        self.0.last().unwrap()
    }

    pub fn last_mut(&mut self) -> &mut FullContext {
        self.0.last_mut().unwrap()
    }

    pub fn current(&self) -> &Context {
        self.last().current()
    }

    // pub fn increment_current(&mut self, func_len: usize) {
    //     self.last_mut().increment_current(func_len)
    // }

    pub fn jump_current(&mut self, pos: usize) {
        self.last_mut().jump_current(pos)
    }

    pub fn current_mut(&mut self) -> PeekMut<Context> {
        self.last_mut().current_mut()
    }

    pub fn ip(&self) -> usize {
        self.last().ip()
    }

    pub fn valid(&self) -> bool {
        self.last().valid()
    }

    pub fn yeet_current(&mut self) {
        self.last_mut().yeet_current();
    }

    pub fn set_group_and_push(&mut self, group: Id) {
        self.last_mut().set_group_and_push(group)
    }

    pub fn pop_group(&mut self) -> Id {
        self.last_mut().pop_group()
    }

    pub fn pop_groups_until(&mut self, group: Id) {
        self.last_mut().pop_groups_until(group)
    }

    pub fn group(&self) -> Id {
        self.last().group()
    }
}

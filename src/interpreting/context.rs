use std::cmp::Ordering;
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;

use derive_more::{Deref, DerefMut};

use super::vm::{FuncCoord, ValueRef};
use crate::compiling::bytecode::OptRegister;
use crate::gd::ids::Id;
use crate::sources::CodeArea;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallInfo {
    pub func: FuncCoord,
    pub return_dest: OptRegister,
    pub call_area: Option<CodeArea>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub ip: usize,

    pub group: Id,
    pub registers: Vec<[ValueRef; 256]>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContextSplitMode {
    Allow,
    Disallow,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ip: 0,
            group: Id::Specific(0),
            registers: vec![],
        }
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
        self.contexts.peek().expect("BUG: no current context")
    }

    pub fn current_mut(&mut self) -> PeekMut<Context> {
        self.contexts.peek_mut().expect("BUG: no current context")
    }

    pub fn jump_current(&mut self, pos: usize) {
        self.current_mut().ip = pos
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

    pub fn set_group(&mut self, group: Id) {
        let mut current = self.current_mut();
        current.group = group;
    }

    // pub fn pop_group(&mut self) -> Id {
    //     let mut current = self.current_mut();
    //     current.group_stack.pop().unwrap()
    // }

    // pub fn pop_groups_until(&mut self, group: Id) {
    //     let mut current = self.current_mut();
    //     while current.group_stack.pop().unwrap() != group {}
    //     current.group_stack.push(group);
    // }

    pub fn group(&self) -> Id {
        self.current().group
    }
}

#[derive(Debug, Deref, DerefMut)]
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

    pub fn set_group(&mut self, group: Id) {
        self.last_mut().set_group(group)
    }

    // pub fn pop_group(&mut self) -> Id {
    //     self.last_mut().pop_group()
    // }

    // pub fn pop_groups_until(&mut self, group: Id) {
    //     self.last_mut().pop_groups_until(group)
    // }

    pub fn group(&self) -> Id {
        self.last().group()
    }
}

use std::cmp::Ordering;
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;
use std::mem::ManuallyDrop;

use ahash::AHashMap;
use derive_more::{Deref, DerefMut};

use super::value::{BuiltinFn, StoredValue};
use super::vm::{DeepClone, FuncCoord, ValueRef, Vm};
use crate::compiling::bytecode::OptRegister;
use crate::compiling::opcodes::OpcodePos;
use crate::gd::ids::Id;
use crate::sources::CodeArea;
use crate::util::ImmutVec;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackItem {
    pub registers: ImmutVec<ValueRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallInfo {
    pub func: FuncCoord,
    pub return_dest: Option<OptRegister>,
    pub call_area: Option<CodeArea>,
    pub is_builtin: Option<BuiltinFn>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TryCatch {
    pub jump_pos: OpcodePos,
    pub reg: OptRegister,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub ip: usize,

    pub try_catches: Vec<TryCatch>,

    pub group: Id,
    pub stack: Vec<StackItem>,

    pub returned: Option<ValueRef>,
}

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// pub enum ContextSplitMode {
//     Allow,
//     Disallow,
// }

#[allow(clippy::new_without_default)]
impl Context {
    pub fn new() -> Self {
        Self {
            ip: 0,
            group: Id::Specific(0),
            stack: vec![],
            try_catches: vec![],
            returned: None,
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

    pub fn current_ip(&self) -> usize {
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

    pub fn current_group(&self) -> Id {
        self.current().group
    }
}

impl Vm {
    pub fn split_current_context(&mut self) {
        let current = self.context_stack.current();
        let mut new = current.clone();

        // lord forgive me for what i am about to do

        let mut clone_map: AHashMap<*mut StoredValue, ValueRef> = AHashMap::new();

        for stack_item in &mut new.stack {
            for reg in stack_item.registers.iter_mut() {
                let k = match clone_map.get(&reg.as_ptr()) {
                    Some(k) => k.clone(),
                    None => {
                        let k = self.deep_clone_ref(&*reg);
                        clone_map.insert(reg.as_ptr(), k.clone());
                        k
                    },
                };

                *reg = k;
            }
        }
        self.context_stack.last_mut().contexts.push(new);
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

    pub fn current_mut(&mut self) -> PeekMut<Context> {
        self.last_mut().current_mut()
    }
}

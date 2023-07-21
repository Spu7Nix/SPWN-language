use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::binary_heap::PeekMut;
use std::collections::BinaryHeap;
use std::mem::ManuallyDrop;
use std::rc::Rc;

use ahash::AHashMap;
use derive_more::{Deref, DerefMut};

use super::value::{BuiltinFn, StoredValue, Value};
use super::vm::{DeepClone, FuncCoord, ValueRef, Vm};
use crate::compiling::bytecode::OptRegister;
use crate::compiling::opcodes::OpcodePos;
use crate::gd::ids::Id;
use crate::sources::{CodeArea, SpwnSource, ZEROSPAN};
use crate::util::ImmutVec;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackItem {
    pub registers: ImmutVec<ValueRef>,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ReturnDest {
    Reg(OptRegister),
    Extra,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallInfo {
    pub func: FuncCoord,
    pub return_dest: Option<ReturnDest>,
    pub call_area: Option<CodeArea>,
    pub is_builtin: Option<BuiltinFn>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TryCatch {
    pub jump_pos: OpcodePos,
    pub reg: OptRegister,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub unique_id: usize,

    pub ip: usize,

    pub try_catches: Vec<TryCatch>,

    pub group: Id,
    pub stack: Vec<StackItem>,

    pub extra_stack: Vec<StoredValue>,

    pub returned: Option<ValueRef>,
}

impl Eq for Context {
    fn assert_receiver_is_total_eq(&self) {}
}

// #[derive(Debug, PartialEq, Eq, Clone, Copy)]
// pub enum ContextSplitMode {
//     Allow,
//     Disallow,
// }

static mut CONTEXT_DBG_ID: usize = 0;

#[allow(clippy::new_without_default)]
impl Context {
    pub fn new(src: &Rc<SpwnSource>) -> Self {
        Self {
            unique_id: unsafe {
                CONTEXT_DBG_ID += 1;
                CONTEXT_DBG_ID - 1
            },
            ip: 0,
            group: Id::Specific(0),
            stack: vec![],
            try_catches: vec![],
            returned: None,
            extra_stack: vec![],
        }
    }

    pub fn extra_stack_top(&self, pos: usize) -> &StoredValue {
        &self.extra_stack[self.extra_stack.len() - pos - 1]
    }

    pub fn extra_stack_top_mut(&mut self, pos: usize) -> &mut StoredValue {
        let len = self.extra_stack.len();
        &mut self.extra_stack[len - pos - 1]
    }

    pub fn push_extra_stack(&mut self, v: StoredValue) {
        self.extra_stack.push(v)
    }

    pub fn pop_extra_stack(&mut self) -> Option<StoredValue> {
        self.extra_stack.pop()
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

#[derive(Debug)]
pub struct CloneMap(AHashMap<usize, ValueRef>);
// #[derive(Debug)]
// pub struct CloneInfo<'a> {
//     map: &'a mut CloneMap,
//     ptr: usize,
// }

impl CloneMap {
    pub fn new() -> Self {
        Self(AHashMap::new())
    }

    pub fn get(&self, key: &ValueRef) -> Option<&ValueRef> {
        self.0.get(&(key.as_ptr() as usize))
    }

    pub fn get_mut(&mut self, key: &ValueRef) -> Option<&mut ValueRef> {
        self.0.get_mut(&(key.as_ptr() as usize))
    }

    pub fn insert(&mut self, key: &ValueRef, v: ValueRef) -> Option<ValueRef> {
        self.0.insert(key.as_ptr() as usize, v)
    }
}

impl Vm {
    pub fn split_current_context(&mut self) {
        let current = self.context_stack.current();
        let mut new = current.clone();

        new.unique_id = unsafe {
            CONTEXT_DBG_ID += 1;
            CONTEXT_DBG_ID - 1
        };

        // lord forgive me for what i am about to do

        let mut clone_map = CloneMap::new();

        for stack_item in &mut new.stack {
            for reg in stack_item.registers.iter_mut() {
                *reg = reg.deep_clone_checked(self, &mut Some(&mut clone_map));
            }
        }

        for v in &mut new.extra_stack {
            *v = self.deep_clone_map(&*v, &mut Some(&mut clone_map))
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

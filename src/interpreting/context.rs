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
    pub store_extra: Option<OptRegister>,
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

#[allow(clippy::new_without_default)]
impl Context {
    pub fn new(src: &Rc<SpwnSource>) -> Self {
        Self {
            ip: 0,
            group: Id::Specific(0),
            stack: vec![],
            try_catches: vec![],
            returned: None,
            extra_stack: vec![],
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

#[derive(Debug)]
pub struct CloneMap(AHashMap<usize, ValueRef>);

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

        let mut clone_map = CloneMap::new();

        fn dfs_insert_into_map(ptr: &ValueRef, clone_map: &mut CloneMap) {
            if clone_map.get(ptr).is_some() {
                return;
            }
            println!("{:?}", ptr.borrow().value.get_type());
            clone_map.insert(ptr, ValueRef::new(ptr.borrow().clone()));
            ptr.borrow_mut().value.inner_references(|v| {
                dfs_insert_into_map(v, clone_map);
            });
        }

        fn dfs_replace_ptrs(ptr: &mut ValueRef, clone_map: &CloneMap) {
            ptr.borrow_mut().value.inner_references(|v| {
                dfs_replace_ptrs(v, clone_map);
            });
            match clone_map.get(ptr) {
                Some(replacement) => *ptr = replacement.clone(),
                None => {
                    println!("{:?}", ptr.borrow());
                },
            };
        }

        for stack_item in &new.stack {
            for reg in stack_item.registers.iter() {
                dfs_insert_into_map(reg, &mut clone_map);
            }
        }
        for stack_item in &mut new.stack {
            for reg in stack_item.registers.iter_mut() {
                dfs_replace_ptrs(reg, &clone_map);
            }
        }

        // lord forgive me for what i am about to do

        // let mut clone_map: AHashMap<usize, ValueRef> = AHashMap::new();

        // for stack_item in &mut new.stack {
        //     for reg in stack_item.registers.iter_mut() {
        //         let k = match clone_map.get(&(reg.as_ptr() as usize)) {
        //             Some(k) => k.clone(),
        //             None => {
        //                 let k = self.deep_clone_ref(&*reg);
        //                 clone_map.insert(reg.as_ptr() as usize, k.clone());
        //                 k
        //             },
        //         };

        //         *reg = k;
        //     }
        // }

        // for i in &mut new.extra_stack {
        //     *i = self.deep_clone(&*i)
        // }

        // self.context_stack.last_mut().contexts.push(new);
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

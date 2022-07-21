use ahash::AHashMap;

use crate::compiler::compiler::{Code, InstrNum};

use super::{
    interpreter::{CallId, Globals, StoredValue, ValueKey},
    value::{value_ops, Id, ValueIter},
};

#[derive(Debug, Clone)]
pub enum Block {
    For(ValueIter),
    While,
    Try(usize),
}
impl Block {
    pub fn get_iter(&mut self) -> &mut ValueIter {
        if let Block::For(iter) = self {
            iter
        } else {
            panic!("tried to get iterator of non-for block")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub position: (usize, usize),
    pub call_id: CallId,
    pub block_stack: Vec<Block>,
}
impl Frame {
    pub fn block(&mut self) -> &mut Block {
        self.block_stack.last_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub group: Id,
    pub id: String,
    // pub frames: Vec<Frame>,
    pub block_stack: Vec<Block>,
    pub vars: AHashMap<InstrNum, ValueKey>,
    pub stack: Vec<ValueKey>,

    pub returned: Option<ReturnType>,
    pub yeeted: bool,
    pub pos: usize,
}

#[derive(Debug, Clone)]
pub enum ReturnType {
    Explicit,
    Implicit,
}
impl Context {
    pub fn new() -> Self {
        Self {
            group: Id::Specific(0),
            id: "O".into(),
            vars: AHashMap::new(),
            // frames: vec![Frame {
            //     position: (0, 0),
            //     call_id: CallId(0),
            //     block_stack: vec![],
            // }],
            block_stack: vec![],
            stack: vec![],
            returned: None,
            yeeted: false,
            pos: 0,
        }
    }

    pub fn jump_to(&mut self, code: &Code, to: usize, func: usize) {
        self.pos = to;
        if self.pos >= code.bytecode_funcs[func].instructions.len() {
            // implicitly return
            self.returned = Some(ReturnType::Implicit);
        }
    }

    pub fn set_group(&mut self, group: Id) {
        self.group = group;
    }
    // pub fn return_out(&mut self, code: &Code, globals: &mut Globals) {
    //     let func = self.frame().position.0;
    //     self.frames.pop();
    //     self.pop_vars(&code.bytecode_funcs[func].scoped_var_ids);
    //     self.pop_vars(&code.bytecode_funcs[func].capture_ids);
    //     if !self.frames.is_empty() {
    //         self.advance_by(code, 1, globals);
    //     }
    // }
    pub fn yeet(&mut self) {
        self.yeeted = true
    }

    pub fn block(&mut self) -> &mut Block {
        self.block_stack.last_mut().unwrap()
    }

    pub fn modify_var(&mut self, id: InstrNum, var: StoredValue, globals: &mut Globals) {
        let key = self.get_var(id);
        globals.memory[key] = var;
    }
    pub fn set_var(&mut self, id: InstrNum, k: ValueKey) {
        *self.vars.get_mut(&id).unwrap() = k
    }
    pub fn get_var(&self, id: InstrNum) -> ValueKey {
        self.vars[&id]
    }
    pub fn clone_context(&self, globals: &mut Globals) -> Context {
        let vars: AHashMap<_, _> = self
            .vars
            .iter()
            .map(|(id, k)| {
                (*id, {
                    let value = globals.memory[*k].clone();
                    globals.memory.insert(value)
                })
            })
            .collect();
        let stack = self
            .stack
            .iter()
            .map(|p| {
                let value = globals.memory[*p].clone();
                globals.memory.insert(value)
            })
            .collect::<Vec<_>>();

        Context {
            vars,
            stack,
            ..self.clone()
        }
    }

    pub fn is_mergable_with(&self, other: &Self, globals: &Globals) -> bool {
        // check if stack is equal:
        if self.stack.len() != other.stack.len() {
            return false;
        }
        for i in 0..self.stack.len() {
            if !value_ops::equality(
                &globals.memory[self.stack[i]].value,
                &globals.memory[other.stack[i]].value,
                globals,
            ) {
                return false;
            }
        }
        // check if vars are equal
        if self.vars.len() != other.vars.len() {
            return false;
        }

        for (id, k1) in &self.vars {
            match other.vars.get(&id) {
                Some(k2) => {
                    if !value_ops::equality(
                        &globals.memory[*k1].value,
                        &globals.memory[*k2].value,
                        globals,
                    ) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub enum FullContext {
    Single(Context),
    Split(Box<FullContext>, Box<FullContext>),
}

impl FullContext {
    pub fn single() -> Self {
        Self::Single(Context::new())
    }
    pub fn is_split(&self) -> bool {
        matches!(self, FullContext::Split(..))
    }

    pub fn remove_finished(&mut self) -> bool {
        let stack = Self::stack(&mut self.iter(SkipMode::IncludeReturns).filter_map(|c| {
            if c.inner().yeeted {
                None
            } else {
                Some(c.clone())
            }
        }));
        if let Some(c) = stack {
            *self = c;
            true
        } else {
            false
        }
    }

    pub fn clean_yeeted(&mut self) -> bool {
        // true: whole tree should be yeeted
        match self {
            FullContext::Single(c) => c.yeeted,
            FullContext::Split(a, b) => {
                let a_yeeted = a.clean_yeeted();
                let b_yeeted = b.clean_yeeted();
                if a_yeeted && b_yeeted {
                    true
                } else if a_yeeted {
                    *self = *b.clone();
                    false
                } else if b_yeeted {
                    *self = *a.clone();
                    false
                } else {
                    false
                }
            }
        }
    }

    pub fn stack(list: &mut impl Iterator<Item = Self>) -> Option<Self> {
        let first = list.next()?;
        match Self::stack(list) {
            Some(second) => Some(FullContext::Split(first.into(), second.into())),
            None => Some(first),
        }
    }

    pub fn inner(&mut self) -> &mut Context {
        if let FullContext::Single(c) = self {
            c
        } else {
            panic!("Tried to take inner context of split context")
        }
    }
    // pub fn inner_mut(&mut self) -> &mut Context {
    //     if let FullContext::Single(c) = self {
    //         c
    //     } else {
    //         panic!("Tried to take inner context of split context")
    //     }
    // }

    pub fn split_context(&mut self, globals: &mut Globals) {
        let mut a = self.inner().clone();
        let mut b = a.clone_context(globals);
        a.id += "a";
        b.id += "b";
        let a = FullContext::Single(a);
        let b = FullContext::Single(b);
        *self = FullContext::Split(Box::new(a), Box::new(b));
    }

    pub fn iter(&mut self, skip_returns: SkipMode) -> ContextIter {
        ContextIter::new(self, skip_returns)
    }

    pub(crate) fn yeet_implicit(&mut self) {
        match self {
            FullContext::Single(a) => {
                if let Some(ReturnType::Implicit) = a.returned {
                    a.yeeted = true;
                }
            }
            FullContext::Split(a, b) => {
                a.yeet_implicit();
                b.yeet_implicit();
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SkipMode {
    SkipReturns,
    IncludeReturns,
}

pub struct ContextIter<'a> {
    current_node: Option<&'a mut FullContext>,
    right_nodes: Vec<&'a mut FullContext>,
    skip_returned: SkipMode,
}
impl<'a> ContextIter<'a> {
    fn new(node: &'a mut FullContext, skip_returned: SkipMode) -> Self {
        let mut iter = Self {
            current_node: None,
            right_nodes: vec![],
            skip_returned,
        };
        iter.add_left_subtree(node);
        iter
    }
    fn add_left_subtree(&mut self, mut node: &'a mut FullContext) {
        loop {
            match node {
                FullContext::Split(left, right) => {
                    self.right_nodes.push(right);
                    node = left;
                }
                single => {
                    self.current_node = Some(single);
                    break;
                }
            }
        }
    }
}

impl<'a> Iterator for ContextIter<'a> {
    type Item = &'a mut FullContext;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current_node.take();

        if let Some(r) = self.right_nodes.pop() {
            self.add_left_subtree(r);
        }

        match (result, self.skip_returned) {
            (Some(c), SkipMode::SkipReturns) => {
                if c.inner().returned.is_some() || c.inner().yeeted {
                    self.next()
                } else {
                    Some(c)
                }
            }

            (Some(c), SkipMode::IncludeReturns) => {
                if c.inner().yeeted {
                    self.next()
                } else {
                    Some(c)
                }
            }
            (None, _) => None,
        }
    }
}

use crate::compiler::compiler::{Code, InstrNum};

use super::{
    interpreter::{CallId, Globals, StoredValue, ValueKey},
    value::{value_ops, Id, ValueIter},
};

#[derive(Debug, Clone)]
pub enum Block {
    For(ValueIter),
    While,
    Try(InstrNum),
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
    pub frames: Vec<Frame>,
    pub vars: Vec<Vec<ValueKey>>,
    pub stack: Vec<ValueKey>,
}
impl Context {
    pub fn new(var_count: usize) -> Self {
        Self {
            group: Id::Specific(0),
            id: "O".into(),
            vars: vec![vec![]; var_count],
            frames: vec![Frame {
                position: (0, 0),
                call_id: CallId(0),
                block_stack: vec![],
            }],
            stack: vec![],
        }
    }
    pub fn frame(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
    pub fn advance_to(&mut self, code: &Code, i: usize, globals: &mut Globals) {
        let frame = self.frame();
        let pos = &mut frame.position;
        let call_id = frame.call_id;
        pos.1 = i;
        let (func, i) = pos;
        if *i >= code.bytecode_funcs[*func].instructions.len() {
            if !globals.calls.contains(&call_id) {
                self.yeet();
                return;
            }
            self.return_out(code, globals);
        }
    }
    pub fn advance_by(&mut self, code: &Code, n: usize, globals: &mut Globals) {
        let i = self.frame().position.1;
        self.advance_to(code, i + n, globals)
    }

    pub fn set_group(&mut self, group: Id) {
        self.group = group;
    }
    pub fn return_out(&mut self, code: &Code, globals: &mut Globals) {
        let func = self.frame().position.0;
        self.frames.pop();
        self.pop_vars(&code.bytecode_funcs[func].scoped_var_ids);
        self.pop_vars(&code.bytecode_funcs[func].capture_ids);
        if !self.frames.is_empty() {
            self.advance_by(code, 1, globals);
        }
    }
    pub fn yeet(&mut self) {
        self.frames = vec![];
    }

    pub fn push_vars(&mut self, vars: &[InstrNum], code: &Code, globals: &mut Globals) {
        for i in vars {
            self.vars[*i as usize].push(globals.memory.insert(code.dummy_value()));
        }
    }
    pub fn pop_vars(&mut self, vars: &[InstrNum]) {
        for i in vars {
            self.vars[*i as usize].pop();
        }
    }

    pub fn set_var(&mut self, id: InstrNum, var: StoredValue, globals: &mut Globals) {
        let key = self.get_var(id);
        globals.memory[key] = var;
        // *self.vars[id as usize].last_mut().unwrap() = k
    }
    pub fn replace_var(&mut self, id: InstrNum, k: ValueKey) {
        *self.vars[id as usize].last_mut().unwrap() = k
    }
    pub fn get_var(&self, id: InstrNum) -> ValueKey {
        *self.vars[id as usize].last().unwrap()
    }
    pub fn clone_context(&self, globals: &mut Globals) -> Context {
        let mut vars = vec![];
        for i in &self.vars {
            let var = i
                .iter()
                .map(|k| {
                    let value = globals.memory[*k].clone();
                    globals.memory.insert(value)
                })
                .collect::<Vec<_>>();
            vars.push(var);
        }
        let stack = self
            .stack
            .iter()
            .map(|p| {
                let value = globals.memory[*p].clone();
                globals.memory.insert(value)
            })
            .collect::<Vec<_>>();

        Context {
            group: self.group,
            id: self.id.clone(),
            frames: self.frames.clone(),
            vars,
            stack,
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

        for y in 0..self.vars.len() {
            if self.vars[y].len() != other.vars[y].len() {
                return false;
            }
            for x in 0..self.vars[0].len() {
                if !value_ops::equality(
                    &globals.memory[self.vars[y][x]].value,
                    &globals.memory[other.vars[y][x]].value,
                    globals,
                ) {
                    return false;
                }
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
    pub fn single(var_count: usize) -> Self {
        Self::Single(Context::new(var_count))
    }
    pub fn is_split(&self) -> bool {
        matches!(self, FullContext::Split(..))
    }

    pub fn remove_finished(&mut self) -> bool {
        let stack = Self::stack(&mut self.iter().filter_map(|c| {
            if c.inner().frames.is_empty() {
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

    pub fn iter(&mut self) -> ContextIter {
        ContextIter::new(self)
    }
}

pub struct ContextIter<'a> {
    current_node: Option<&'a mut FullContext>,
    right_nodes: Vec<&'a mut FullContext>,
}
impl<'a> ContextIter<'a> {
    fn new(node: &'a mut FullContext) -> Self {
        let mut iter = Self {
            current_node: None,
            right_nodes: vec![],
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

        match result {
            Some(c) => {
                // if c.inner().broken.is_some() {
                //     self.next()
                // } else {
                Some(c)
                // }
            }
            None => None,
        }
    }
}

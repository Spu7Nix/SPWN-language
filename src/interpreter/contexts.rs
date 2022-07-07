use crate::compiler::compiler::{Code, InstrNum};

use super::interpreter::{Globals, StoredValue, ValueKey};

#[derive(Default, Debug, Clone)]
pub struct Context {
    pub id: String,
    pub pos: (usize, usize),
    pub vars: Vec<Vec<Option<ValueKey>>>,
    pub stack: Vec<ValueKey>,
    pub finished: bool,
}
impl Context {
    pub fn new(var_count: usize) -> Self {
        Self {
            id: "O".into(),
            vars: vec![vec![None]; var_count],
            pos: (0, 0),
            stack: vec![],
            finished: false,
        }
    }
    pub fn advance_to(&mut self, code: &Code, i: usize) {
        self.pos.1 = i;
        if self.pos.0 == 0 && self.pos.1 >= code.instructions[0].0.len() {
            self.finished = true;
        }
    }

    pub fn set_var(&mut self, id: InstrNum, k: ValueKey) {
        *self.vars[id as usize].last_mut().unwrap() = Some(k)
    }
    pub fn get_var(&self, id: InstrNum) -> ValueKey {
        self.vars[id as usize].last().unwrap().unwrap()
    }
    pub fn clone_context(&self, globals: &mut Globals) -> Context {
        let mut vars = vec![];
        for i in &self.vars {
            let var = i
                .iter()
                .map(|k| {
                    if let Some(k) = k {
                        let value = globals.memory[*k].clone();
                        let new_k = globals.memory.insert(value);
                        Some(new_k)
                    } else {
                        None
                    }
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
            id: self.id.clone(),
            pos: self.pos,
            vars,
            stack,
            finished: self.finished,
        }
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

    pub fn inner(&mut self) -> &mut Context {
        if let FullContext::Single(c) = self {
            c
        } else {
            panic!("Tried to take inner context of split context")
        }
    }
    pub fn inner_mut(&mut self) -> &mut Context {
        if let FullContext::Single(c) = self {
            c
        } else {
            panic!("Tried to take inner context of split context")
        }
    }

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
        ContextIter::new(self, false)
    }
}

pub struct ContextIter<'a> {
    with_breaks: bool,
    current_node: Option<&'a mut FullContext>,
    right_nodes: Vec<&'a mut FullContext>,
}
impl<'a> ContextIter<'a> {
    fn new(node: &'a mut FullContext, with_breaks: bool) -> Self {
        let mut iter = Self {
            with_breaks,
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

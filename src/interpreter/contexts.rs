use crate::compiler::compiler::InstrNum;

use super::interpreter::ValueKey;

#[derive(Default, Debug, Clone)]
pub struct Context {
    pub vars: Vec<Vec<Option<ValueKey>>>,
}
impl Context {
    pub fn new(var_count: usize) -> Self {
        Self {
            vars: vec![vec![None]; var_count],
        }
    }
    pub fn get_var(&self, id: InstrNum) -> ValueKey {
        self.vars[id as usize].last().unwrap().unwrap()
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

    pub fn inner(&self) -> &Context {
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
    type Item = &'a mut Context;

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
                Some(c.inner_mut())
                // }
            }
            None => None,
        }
    }
}

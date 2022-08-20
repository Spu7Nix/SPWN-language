use std::collections::HashMap;

use super::interpreter::ValueKey;
use crate::{compilation::code::VarID, leveldata::gd_types::Id};

// variables for each call (kinda like a stack frame thing???)
#[derive(Debug, Clone)]
pub struct VarStack {
    pub vec: Vec<Option<ValueKey>>,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub group: Id,
    pub stack: Vec<ValueKey>,
    pub vars: Vec<VarStack>,

    pub yeeted: bool,
    pub returned: Option<ReturnType>,
    pub pos: isize,
}

impl Context {
    pub fn new(var_count: usize) -> Self {
        Self {
            group: Id::Specific(0),
            stack: Vec::new(),
            vars: vec![VarStack { vec: vec![None] }; var_count],
            yeeted: false,
            returned: None,
            pos: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReturnType {
    Explicit(ValueKey),
    Implicit,
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
        match self {
            FullContext::Single(c) => c,
            FullContext::Split(..) => panic!("cant call inner on split hee hee"),
        }
    }

    pub fn iter(&mut self, skip_returns: SkipMode) -> ContextIter {
        ContextIter::new(self, skip_returns)
    }

    pub fn yeet_implicit(&mut self) {
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

use crate::interpreter::ValueKey;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct VarData {
    key: ValueKey,
    layers: i16,
    mutable: bool,
}

#[derive(Default, Debug, Clone)]
pub struct Context {
    vars: HashMap<String, Vec<VarData>>,
}

#[derive(Debug, Clone)]
pub enum FullContext {
    Single(Context),
    Split(Box<FullContext>, Box<FullContext>),
}

impl FullContext {
    pub fn inner(&self) -> &Context {
        if let FullContext::Single(c) = self {
            c
        } else {
            panic!("Tried to take inner context of split context")
        }
    }

    pub fn foreach<F>(&self, mut f: F)
    where
        F: FnMut(&Context),
    {
        fn inner<F>(c: &FullContext, f: &mut F)
        where
            F: FnMut(&Context),
        {
            match c {
                FullContext::Single(c) => f(c),
                FullContext::Split(a, b) => {
                    inner(a, f);
                    inner(b, f);
                }
            }
        }

        inner(self, &mut f)
    }
}

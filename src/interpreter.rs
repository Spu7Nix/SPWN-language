use std::collections::HashMap;

use slotmap::{new_key_type, SlotMap};

use crate::{
    compiler::{Code, Instruction},
    contexts::FullContext,
    sources::CodeArea,
    value::Value,
};

new_key_type! {
    pub struct ValueKey;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredValue {
    value: Value,
    def_area: CodeArea,
}

pub struct Globals {
    memory: SlotMap<ValueKey, StoredValue>,

    contexts: FullContext,
}

pub fn execute(globals: &mut Globals, code: &Code, func: usize) {
    // let mut stack = vec![];

    for i in &code.instructions[func] {
        match i {
            _ => todo!(),
        }
    }
}

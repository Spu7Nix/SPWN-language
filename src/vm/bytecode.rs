use std::hash::Hash;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, Key, SlotMap};

use super::opcodes::Opcode;

new_key_type! {
    pub struct ConstKey;
}

#[derive(Serialize, Deserialize, PartialEq)]
enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // Array(Vec<Constant>),
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            Constant::Int(v) => v.hash(state),
            Constant::Float(v) => v.to_bits().hash(state),
            Constant::String(v) => v.hash(state),
            Constant::Bool(v) => v.hash(state),
        }
    }
}
impl Eq for Constant {}

// 256 registers (0 to 255 inclusive)
const TOTAL_REGISTERS: u8 = u8::MAX;

struct UniqueRegister<K: Key, T: Hash + Eq> {
    constants: SlotMap<K, T>,
    indexes: AHashMap<T, K>,
}

impl<K: Key, T: Hash + Eq> UniqueRegister<K, T> {
    pub fn new() -> Self {
        Self {
            constants: SlotMap::default(),
            indexes: AHashMap::new(),
        }
    }
    pub fn insert(&mut self, value: T) -> K {
        match self.indexes.get(&value) {
            Some(k) => *k,
            None => self.constants.insert(value),
        }
    }
}

struct BytecodeBuilder {
    constants: UniqueRegister<ConstKey, Constant>,
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self {
            constants: UniqueRegister::new(),
        }
    }

    pub fn load_int(&mut self, int: i64, reg: u8) {
        self.constants.insert(Constant::Int(int));
    }
}

/*

// */

// fn cock() {
//     let builder = BytecodeBuilder::new();
//     let func = builder.enter_func(|f| {
//         self.compile_expr(..., f)
//     });

//     func

//     func.load_int()

// }

struct Function {
    opcodes: Vec<Opcode>,
}

struct Bytecode {
    consts: Vec<Constant>,
    functions: Vec<Function>,
}

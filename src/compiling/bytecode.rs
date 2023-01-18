use std::hash::Hash;

use ahash::AHashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, Key, SecondaryMap, SlotMap};

use crate::vm::opcodes::{Opcode, Register};

new_key_type! {
    pub struct ConstKey;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
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

struct UniqueRegister<K: Key, T: Hash + Eq> {
    slotmap: SlotMap<K, T>,
    indexes: AHashMap<T, K>,
}

impl<K: Key, T: Hash + Eq> UniqueRegister<K, T> {
    pub fn new() -> Self {
        Self {
            slotmap: SlotMap::default(),
            indexes: AHashMap::new(),
        }
    }
    pub fn insert(&mut self, value: T) -> K {
        match self.indexes.get(&value) {
            Some(k) => *k,
            None => self.slotmap.insert(value),
        }
    }
}

enum ProtoOpcode {
    Raw(Opcode),

    Jump(usize),
    JumpIfFalse(Register, usize),
    LoadConst(Register, ConstKey),
}

struct ProtoFunc {
    labels: Vec<Vec<ProtoOpcode>>,
}
impl ProtoFunc {
    pub fn new() -> Self {
        Self {
            labels: vec![vec![]],
        }
    }
}

pub struct BytecodeBuilder {
    constants: UniqueRegister<ConstKey, Constant>,

    funcs: Vec<ProtoFunc>,
}

pub struct FuncBuilder<'a> {
    code_builder: &'a mut BytecodeBuilder,

    func: usize,
    label: usize,

    used_regs: u8,
}

impl BytecodeBuilder {
    pub fn new() -> Self {
        Self {
            constants: UniqueRegister::new(),
            funcs: vec![],
        }
    }

    pub fn new_func<F>(&mut self, f: F)
    where
        F: FnOnce(&mut FuncBuilder),
    {
        self.funcs.push(ProtoFunc::new());

        let mut func_builder = FuncBuilder {
            func: self.funcs.len() - 1,
            code_builder: self,
            label: 0,
            used_regs: 0,
        };

        f(&mut func_builder);
    }

    pub fn build(self) -> Bytecode {
        let mut const_index_map = SecondaryMap::default();

        let consts = self
            .constants
            .slotmap
            .into_iter()
            .enumerate()
            .map(|(i, (k, c))| {
                const_index_map.insert(k, i);
                c
            })
            .collect();

        let functions = self
            .funcs
            .into_iter()
            .map(|f| {
                let mut opcodes = vec![];

                for label in &f.labels {
                    for opcode in label {
                        opcodes.push(match opcode {
                            ProtoOpcode::Raw(o) => *o,
                            ProtoOpcode::Jump(l) => Opcode::Jump {
                                to: f.labels[0..*l].iter().map(|v| v.len()).sum::<usize>() as u16,
                            },
                            ProtoOpcode::JumpIfFalse(r, l) => Opcode::JumpIfFalse {
                                src: *r,
                                to: f.labels[0..*l].iter().map(|v| v.len()).sum::<usize>() as u16,
                            },
                            ProtoOpcode::LoadConst(r, k) => Opcode::LoadConst {
                                dest: *r,
                                id: const_index_map[*k] as u16,
                            },
                        })
                    }
                }

                Function { opcodes }
            })
            .collect();

        Bytecode { consts, functions }
    }
}

impl<'a> FuncBuilder<'a> {
    fn label_vec(&mut self) -> &mut Vec<ProtoOpcode> {
        &mut self.code_builder.funcs[self.func].labels[self.label]
    }

    pub fn next_reg(&mut self) -> Register {
        let old = self.used_regs;
        self.used_regs = self
            .used_regs
            .checked_add(1)
            .expect("sil;ly goober used too mnay regusters!!!! i🙌!");
        old
    }

    pub fn block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut FuncBuilder),
    {
        let (label, after_label) = {
            let func = &mut self.code_builder.funcs[self.func];
            func.labels.push(vec![]);
            func.labels.push(vec![]);
            (func.labels.len() - 2, func.labels.len() - 1)
        };

        let mut func_builder = FuncBuilder {
            code_builder: self.code_builder,
            func: self.func,
            used_regs: self.used_regs,
            label,
        };

        f(&mut func_builder);

        self.used_regs = func_builder.used_regs;
        self.label = after_label;
    }

    pub fn new_array<F>(&mut self, len: usize, dest: Register, f: F)
    where
        F: FnOnce(&mut FuncBuilder, &mut Vec<Register>),
    {
        self.label_vec().push(ProtoOpcode::Raw(Opcode::AllocArray {
            size: len as u16,
            dest,
        }));

        let mut items = vec![];
        f(self, &mut items);

        for i in items {
            self.label_vec()
                .push(ProtoOpcode::Raw(Opcode::Push { elem: i, dest }))
        }
    }

    pub fn load_int(&mut self, value: i64, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::Int(value));
        self.label_vec().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_float(&mut self, value: f64, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::Float(value));
        self.label_vec().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_string(&mut self, value: String, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::String(value));
        self.label_vec().push(ProtoOpcode::LoadConst(reg, k))
    }
    pub fn load_bool(&mut self, value: bool, reg: Register) {
        let k = self.code_builder.constants.insert(Constant::Bool(value));
        self.label_vec().push(ProtoOpcode::LoadConst(reg, k))
    }

    pub fn add(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Add { left, right, dest }))
    }
    pub fn sub(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Sub { left, right, dest }))
    }
    pub fn mult(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Mult { left, right, dest }))
    }
    pub fn div(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Div { left, right, dest }))
    }
    pub fn modulo(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Mod { left, right, dest }))
    }
    pub fn pow(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Pow { left, right, dest }))
    }
    pub fn shl(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::ShiftLeft { left, right, dest }))
    }
    pub fn shr(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::ShiftRight { left, right, dest }))
    }
    pub fn eq(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Eq { left, right, dest }))
    }
    pub fn neq(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Neq { left, right, dest }))
    }
    pub fn gt(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Gt { left, right, dest }))
    }
    pub fn gte(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Gte { left, right, dest }))
    }
    pub fn lt(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Lt { left, right, dest }))
    }
    pub fn lte(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Lte { left, right, dest }))
    }
    pub fn range(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Range { left, right, dest }))
    }
    pub fn in_op(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::In { left, right, dest }))
    }
    pub fn as_op(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::As { left, right, dest }))
    }
    pub fn is_op(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::As { left, right, dest }))
    }
    pub fn bin_or(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::BinOr { left, right, dest }))
    }
    pub fn bin_and(&mut self, left: Register, right: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::BinAnd { left, right, dest }))
    }

    pub fn add_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::AddEq { left, right }))
    }
    pub fn sub_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::SubEq { left, right }))
    }
    pub fn mult_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::MultEq { left, right }))
    }
    pub fn div_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::DivEq { left, right }))
    }
    pub fn modulo_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::ModEq { left, right }))
    }
    pub fn pow_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::PowEq { left, right }))
    }
    pub fn shl_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::ShiftLeftEq { left, right }))
    }
    pub fn shr_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::ShiftRightEq { left, right }))
    }
    pub fn bin_and_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::BinAndEq { left, right }))
    }
    pub fn bin_or_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::BinOrEq { left, right }))
    }
    pub fn bin_not_eq(&mut self, left: Register, right: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::BinNotEq { left, right }))
    }

    pub fn unary_not(&mut self, src: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Not { src, dest }))
    }
    pub fn unary_negate(&mut self, src: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Negate { src, dest }))
    }
    pub fn unary_bin_not(&mut self, src: Register, dest: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::BinNot { src, dest }))
    }

    pub fn copy(&mut self, from: Register, to: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Copy { from, to }))
    }

    pub fn print(&mut self, reg: Register) {
        self.label_vec()
            .push(ProtoOpcode::Raw(Opcode::Print { reg }))
    }

    pub fn repeat_block(&mut self) {
        let to = self.label;
        self.label_vec().push(ProtoOpcode::Jump(to))
    }

    pub fn exit_if_false(&mut self, reg: Register) {
        let to = self.label + 1;
        self.label_vec().push(ProtoOpcode::JumpIfFalse(reg, to))
    }
}

// pub fn cockshitball() {
//     let mut builder = BytecodeBuilder::new();

//     builder.new_func(|mut b| {
//         b.load_int(0, 0);
//         b.load_int(1, 1);

//         b.load_int(500, 3);

//         b.block(|b| {
//             b.lt(1, 3, 4);
//             b.exit_if_false(4);

//             b.add(0, 1, 2);
//             b.print(2);

//             b.copy(1, 0);
//             b.copy(2, 1);

//             b.repeat_block();
//         });

//         b.print(1);
//     });

//     let built = builder.build();
//     println!("{}", built);

//     let x = bincode::serialize(&built).unwrap();
//     println!("\n\nlen: {}, ser {x:?}", x.len());
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    opcodes: Vec<Opcode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bytecode {
    consts: Vec<Constant>,

    functions: Vec<Function>,
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Constants: {:?}\n", self.consts)?;

        for (i, func) in self.functions.iter().enumerate() {
            writeln!(f, "======== Function {} ========", i)?;
            for (i, opcode) in func.opcodes.iter().enumerate() {
                writeln!(f, "{}\t{:?}", i.to_string().bright_blue(), opcode)?;
            }
        }

        Ok(())
    }
}

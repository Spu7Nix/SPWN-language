use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ahash::AHashMap;
use slotmap::{new_key_type, SlotMap};

use crate::{
    compiling::bytecode::Bytecode,
    sources::{CodeArea, CodeSpan},
    util::Interner,
};

use super::{
    error::RuntimeError,
    opcodes::{Opcode, Register},
    value::{StoredValue, Value},
    value_ops,
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

new_key_type! {
    pub struct ValueKey;
}

pub struct Vm<'a> {
    registers: [ValueKey; 256],

    pub memory: SlotMap<ValueKey, StoredValue>,

    program: &'a Bytecode<Register>,

    pub interner: Rc<RefCell<Interner>>,
}

impl<'a> Vm<'a> {
    pub fn new(program: &'a Bytecode<Register>, interner: Rc<RefCell<Interner>>) -> Vm {
        Self {
            memory: SlotMap::default(),
            registers: [ValueKey::default(); 256],
            interner,
            program,
        }
    }

    pub fn deep_clone_value(&mut self, k: ValueKey) -> ValueKey {
        let v = self.memory[k].clone();

        let value = match v.value {
            Value::Array(arr) => {
                Value::Array(arr.into_iter().map(|v| self.deep_clone_value(v)).collect())
            }
            v => v,
        };
        self.memory.insert(StoredValue {
            value,
            area: v.area.clone(),
        })
    }
    pub fn deep_clone_reg(&mut self, reg: Register) -> ValueKey {
        self.deep_clone_value(self.registers[reg as usize])
    }

    pub fn get_reg(&self, reg: Register) -> &StoredValue {
        &self.memory[self.registers[reg as usize]]
    }
    pub fn get_reg_mut(&mut self, reg: Register) -> &mut StoredValue {
        &mut self.memory[self.registers[reg as usize]]
    }

    pub fn set_reg(&mut self, reg: Register, v: StoredValue) {
        self.registers[reg as usize] = self.memory.insert(v)
    }

    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: self.program.src.clone(),
        }
    }
    pub fn get_span(&self, func: usize, i: usize) -> CodeSpan {
        self.program.opcode_span_map[&(func, i)]
    }
    pub fn get_area(&self, func: usize, i: usize) -> CodeArea {
        self.make_area(self.get_span(func, i))
    }

    pub fn run_func(&mut self, func: usize) -> RuntimeResult<()> {
        let opcodes = &self.program.functions[func].opcodes;

        let mut ip = 0_usize;

        while ip < opcodes.len() {
            match &opcodes[ip] {
                Opcode::LoadConst { dest, id } => {
                    let value = Value::from_const(&self.program.consts[*id as usize]);

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value,
                            area: self.get_area(func, ip),
                        },
                    )
                    // self.registers[*dest as usize] = self.memory.insert(Value::from_const(
                    //     &self.program.consts[*id as usize],
                    //     &mut self.interner,
                    // ))
                }
                Opcode::Copy { from, to } => {
                    self.registers[*to as usize] = self.deep_clone_reg(*from)
                }
                Opcode::Print { reg } => {
                    println!("{}", self.get_reg(*reg).value.runtime_display(self))
                }
                Opcode::AllocArray { size, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Array(Vec::with_capacity(*size as usize)),
                        area: self.get_area(func, ip),
                    },
                ),
                Opcode::AllocDict { size, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Dict(AHashMap::with_capacity(*size as usize)),
                        area: self.get_area(func, ip),
                    },
                ),
                Opcode::PushArrayElem { elem, dest } => {
                    let push = self.deep_clone_reg(*elem);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Array(v) => v.push(push),
                        _ => unreachable!(),
                    }
                }
                Opcode::PushDictElem { elem, key, dest } => {
                    let push = self.deep_clone_reg(*elem);

                    let key = match &self.get_reg(*key).value {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };

                    let key = self.interner.borrow_mut().get_or_intern(key);

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Dict(v) => {
                            v.insert(key, push);
                        }
                        _ => unreachable!(),
                    }
                }
                Opcode::Add { left, right, dest } => {
                    self.bop(value_ops::add, func, ip, left, right, dest)?
                }
                Opcode::Sub { left, right, dest } => {
                    self.bop(value_ops::sub, func, ip, left, right, dest)?
                }
                Opcode::Mult { left, right, dest } => {
                    self.bop(value_ops::mult, func, ip, left, right, dest)?
                }
                Opcode::Div { left, right, dest } => {
                    self.bop(value_ops::div, func, ip, left, right, dest)?
                }
                Opcode::Mod { left, right, dest } => {
                    self.bop(value_ops::modulo, func, ip, left, right, dest)?
                }
                Opcode::Pow { left, right, dest } => {
                    self.bop(value_ops::pow, func, ip, left, right, dest)?
                }
                Opcode::ShiftLeft { left, right, dest } => todo!(),
                Opcode::ShiftRight { left, right, dest } => todo!(),
                Opcode::BinOr { left, right, dest } => todo!(),
                Opcode::BinAnd { left, right, dest } => todo!(),
                Opcode::AddEq { left, right } => todo!(),
                Opcode::SubEq { left, right } => todo!(),
                Opcode::MultEq { left, right } => todo!(),
                Opcode::DivEq { left, right } => todo!(),
                Opcode::ModEq { left, right } => todo!(),
                Opcode::PowEq { left, right } => todo!(),
                Opcode::ShiftLeftEq { left, right } => todo!(),
                Opcode::ShiftRightEq { left, right } => todo!(),
                Opcode::BinAndEq { left, right } => todo!(),
                Opcode::BinOrEq { left, right } => todo!(),
                Opcode::BinNotEq { left, right } => todo!(),
                Opcode::Not { src, dest } => {
                    let span = self.get_span(func, ip);
                    let value = value_ops::not(self.get_reg(*src), span, self)?;
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value,
                            area: self.make_area(span),
                        },
                    )
                }
                Opcode::Negate { src, dest } => {
                    let span = self.get_span(func, ip);
                    let value = value_ops::negate(self.get_reg(*src), span, self)?;
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value,
                            area: self.make_area(span),
                        },
                    )
                }
                Opcode::BinNot { src, dest } => todo!(),

                Opcode::Eq { left, right, dest } => todo!(),
                Opcode::Neq { left, right, dest } => todo!(),
                Opcode::Gt { left, right, dest } => {
                    self.bop(value_ops::gt, func, ip, left, right, dest)?
                }
                Opcode::Lt { left, right, dest } => {
                    self.bop(value_ops::lt, func, ip, left, right, dest)?
                }
                Opcode::Gte { left, right, dest } => {
                    self.bop(value_ops::gte, func, ip, left, right, dest)?
                }
                Opcode::Lte { left, right, dest } => {
                    self.bop(value_ops::lte, func, ip, left, right, dest)?
                }
                Opcode::Range { left, right, dest } => {
                    self.bop(value_ops::range, func, ip, left, right, dest)?
                }
                Opcode::In { left, right, dest } => todo!(),
                Opcode::As { left, right, dest } => todo!(),
                Opcode::Is { left, right, dest } => todo!(),
                Opcode::And { left, right, dest } => {
                    self.bop(value_ops::and, func, ip, left, right, dest)?
                }
                Opcode::Or { left, right, dest } => {
                    self.bop(value_ops::or, func, ip, left, right, dest)?
                }
                Opcode::Jump { to } => {
                    ip = *to as usize;
                    continue;
                }
                Opcode::JumpIfFalse { src, to } => {
                    let span = self.get_span(func, ip);
                    // println!("{:?}", self.get_reg(*src).value);
                    if !value_ops::to_bool(self.get_reg(*src), span, self)? {
                        ip = *to as usize;
                        continue;
                    }
                }
                Opcode::Ret { src } => todo!(),
                Opcode::WrapMaybe { src, dest } => todo!(),
                Opcode::LoadNone { dest } => todo!(),
                Opcode::LoadEmpty { dest } => todo!(),
                Opcode::Index { from, dest, index } => todo!(),
                Opcode::Member { from, dest, member } => todo!(),
                Opcode::Associated { from, dest, name } => todo!(),
                Opcode::YeetContext => todo!(),
                Opcode::EnterArrowStatement { skip_to } => todo!(),
                Opcode::LoadBuiltins { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Builtins,
                            area: self.make_area(span),
                        },
                    )
                }
                Opcode::Export { src } => todo!(),
            }

            ip += 1;
        }

        Ok(())
    }

    #[inline]
    fn bop<F>(
        &mut self,
        op: F,
        func: usize,
        ip: usize,
        left: &u8,
        right: &u8,
        dest: &u8,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&StoredValue, &StoredValue, CodeSpan, &Vm) -> RuntimeResult<Value>,
    {
        let span = self.get_span(func, ip);
        let value = op(self.get_reg(*left), self.get_reg(*right), span, self)?;
        self.set_reg(
            *dest,
            StoredValue {
                value,
                area: self.make_area(span),
            },
        );
        Ok(())
    }
}

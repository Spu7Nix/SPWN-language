use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use lasso::Spur;
use slotmap::{new_key_type, SlotMap};

use super::error::RuntimeError;
use super::opcodes::{Opcode, Register};
use super::value::{ArgData, StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::Bytecode;
use crate::gd::ids::IDClass;
use crate::sources::{CodeArea, CodeSpan};
use crate::util::Interner;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

new_key_type! {
    pub struct ValueKey;
}

pub struct Vm<'a> {
    // 256 registers per function
    registers: Vec<Vec<ValueKey>>,

    pub memory: SlotMap<ValueKey, StoredValue>,

    program: &'a Bytecode<Register>,

    pub interner: Rc<RefCell<Interner>>,

    pub id_counters: [usize; 4],
}

impl<'a> Vm<'a> {
    pub fn new(program: &'a Bytecode<Register>, interner: Rc<RefCell<Interner>>) -> Vm {
        Self {
            memory: SlotMap::default(),
            registers: vec![],
            interner,
            program,
            id_counters: [0; 4],
        }
    }

    pub fn resolve(&self, spur: &Spur) -> String {
        self.interner.borrow().resolve(spur).to_string()
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
        self.deep_clone_value(self.registers.last().unwrap()[reg as usize])
    }

    pub fn get_reg(&self, reg: Register) -> &StoredValue {
        &self.memory[self.registers.last().unwrap()[reg as usize]]
    }

    pub fn get_reg_key(&self, reg: Register) -> ValueKey {
        self.registers.last().unwrap()[reg as usize]
    }

    pub fn get_reg_mut(&mut self, reg: Register) -> &mut StoredValue {
        &mut self.memory[self.registers.last().unwrap()[reg as usize]]
    }

    pub fn set_reg(&mut self, reg: Register, v: StoredValue) {
        self.registers.last_mut().unwrap()[reg as usize] = self.memory.insert(v)
    }

    pub fn set_reg_key(&mut self, reg: Register, k: ValueKey) {
        self.registers.last_mut().unwrap()[reg as usize] = k
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

    pub fn push_func_regs(&mut self, func: usize) {
        let regs_used = self.program.functions[func].regs_used;
        self.registers.push(vec![ValueKey::default(); regs_used]);
    }

    pub fn pop_func_regs(&mut self) {
        self.registers.pop();
    }

    pub fn run_func(&mut self, func: usize) -> RuntimeResult<Option<ValueKey>> {
        let opcodes = &self.program.functions[func].opcodes;
        // let regs_used = self.program.functions[func].regs_used;

        // self.registers.push(vec![ValueKey::default(); regs_used]);

        let mut ip = 0_usize;

        while ip < opcodes.len() {
            match &opcodes[ip] {
                Opcode::LoadConst { dest, id } => {
                    let value = Value::from_const(&self.program.consts[*id as usize], self);

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value,
                            area: self.get_area(func, ip),
                        },
                    )
                }
                Opcode::Copy { from, to } => {
                    self.registers.last_mut().unwrap()[*to as usize] = self.deep_clone_reg(*from)
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
                Opcode::ShiftLeft { left, right, dest } => {
                    self.bop(value_ops::shift_left, func, ip, left, right, dest)?
                }
                Opcode::ShiftRight { left, right, dest } => {
                    self.bop(value_ops::shift_right, func, ip, left, right, dest)?
                }
                Opcode::BinOr { left, right, dest } => {
                    self.bop(value_ops::bin_or, func, ip, left, right, dest)?
                }
                Opcode::BinAnd { left, right, dest } => {
                    self.bop(value_ops::bin_and, func, ip, left, right, dest)?
                }

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

                Opcode::Eq { left, right, dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Bool(value_ops::equality(
                                &self.get_reg(*left).value,
                                &self.get_reg(*right).value,
                                self,
                            )),
                            area: self.make_area(span),
                        },
                    );
                }
                Opcode::Neq { left, right, dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Bool(!value_ops::equality(
                                &self.get_reg(*left).value,
                                &self.get_reg(*right).value,
                                self,
                            )),
                            area: self.make_area(span),
                        },
                    );
                }
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
                    if !value_ops::to_bool(self.get_reg(*src), span, self)? {
                        ip = *to as usize;
                        continue;
                    }
                }
                Opcode::Ret { src } => return Ok(Some(self.get_reg_key(*src))),
                Opcode::WrapMaybe { src, dest } => {
                    let v = self.deep_clone_reg(*src);
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Maybe(Some(v)),
                            area: self.make_area(span),
                        },
                    )
                }
                Opcode::LoadNone { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Maybe(None),
                            area: self.make_area(span),
                        },
                    )
                }
                Opcode::LoadEmpty { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Empty,
                            area: self.make_area(span),
                        },
                    )
                }
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
                Opcode::Call { args, base, dest } => {
                    let base = self.get_reg(*base);
                    let span = self.get_span(func, ip);
                    match base.value.clone() {
                        Value::Macro {
                            func_id,
                            args: arg_data,
                        } => {
                            let mut param_map = AHashMap::new();

                            for data in &arg_data {
                                param_map.insert(data.name, None);
                            }

                            match &self.get_reg(*args).value {
                                Value::Array(v) => {
                                    match &self.memory[v[0]].value {
                                        Value::Array(v) => {
                                            if v.len() > arg_data.len() {
                                                return Err(RuntimeError::TooManyArguments {
                                                    call_area: self.make_area(span),
                                                    macro_def_area: base.area.clone(),
                                                    macro_arg_amount: arg_data.len(),
                                                    call_arg_amount: v.len(),
                                                });
                                            }
                                            for (param, data) in v.iter().zip(&arg_data) {
                                                param_map.insert(data.name, Some(*param));
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                    match &self.memory[v[1]].value {
                                        Value::Dict(m) => {
                                            for (name, param) in m {
                                                if param_map.contains_key(name) {
                                                    param_map.insert(*name, Some(*param));
                                                } else {
                                                    return Err(
                                                        RuntimeError::NonexistentArgument {
                                                            call_area: self.make_area(span),
                                                            macro_def_area: base.area.clone(),
                                                            arg_name: self.resolve(name),
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }
                            let base_area = base.area.clone();
                            self.push_func_regs(func_id as usize);

                            for (i, data) in arg_data.iter().enumerate() {
                                let k = match param_map[&data.name] {
                                    Some(k) => self.deep_clone_value(k),
                                    None => match data.default {
                                        Some(k) => self.deep_clone_value(k),
                                        None => {
                                            return Err(RuntimeError::ArgumentNotSatisfied {
                                                call_area: self.make_area(span),
                                                macro_def_area: base_area,
                                                arg_name: self.resolve(&data.name),
                                            })
                                        }
                                    },
                                };
                                println!(
                                    "cocki {} {}",
                                    self.registers.len(),
                                    self.memory[k].value.runtime_display(self)
                                );
                                self.set_reg_key(i as Register, k)
                            }
                            let ret_key = match self.run_func(func_id as usize)? {
                                Some(k) => k,
                                None => self.memory.insert(StoredValue {
                                    value: Value::Empty,
                                    area: base_area,
                                }),
                            };
                            let ret_key = self.deep_clone_value(ret_key);
                            self.pop_func_regs();

                            self.set_reg_key(*dest, ret_key);
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (base.value.get_type(), base.area.clone()),
                                expected: ValueType::Macro,
                                area: self.make_area(span),
                            })
                        }
                    }
                }
                Opcode::CreateMacro { id, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Macro {
                            func_id: *id,
                            args: vec![],
                        },
                        area: self.get_area(func, ip),
                    },
                ),
                Opcode::PushMacroArg { name, dest } => {
                    let name = match &self.get_reg(*name).value {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };

                    let name = self.interner.borrow_mut().get_or_intern(name);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro { args, .. } => args.push(ArgData {
                            name,
                            default: None,
                            pattern: None,
                        }),
                        _ => unreachable!(),
                    }
                }
                Opcode::SetMacroArgDefault { src, dest } => {
                    let set = self.deep_clone_reg(*src);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro { args, .. } => args.last_mut().unwrap().default = Some(set),
                        _ => unreachable!(),
                    }
                }
                Opcode::SetMacroArgPattern { src, dest } => {
                    let set = self.deep_clone_reg(*src);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro { args, .. } => args.last_mut().unwrap().pattern = Some(set),
                        _ => unreachable!(),
                    }
                }
            }

            ip += 1;
        }

        // none is the same as empty (`()`)
        Ok(None)
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

    pub fn next_id(&mut self, c: IDClass) -> u16 {
        self.id_counters[c as usize] += 1;
        self.id_counters[c as usize] as u16
    }
}

use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use lasso::Spur;
use slotmap::{new_key_type, SlotMap};

use super::context::{CallStackItem, FullContext};
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
    pub struct ValueKey; pub struct BytecodeKey;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncCoord {
    func: usize,
    code: BytecodeKey,
}

impl FuncCoord {
    pub fn new(func: usize, code: BytecodeKey) -> Self {
        Self { func, code }
    }
}

pub struct Vm<'a> {
    // 256 registers per function
    pub memory: SlotMap<ValueKey, StoredValue>,

    pub programs: SlotMap<BytecodeKey, &'a Bytecode<Register>>,

    pub interner: Rc<RefCell<Interner>>,

    pub id_counters: [usize; 4],

    pub contexts: FullContext,
}

impl<'a> Vm<'a> {
    pub fn new(interner: Rc<RefCell<Interner>>) -> Vm<'a> {
        Self {
            memory: SlotMap::default(),
            interner,
            programs: SlotMap::default(),
            id_counters: [0; 4],
            contexts: FullContext::new(),
        }
    }

    pub fn resolve(&self, spur: &Spur) -> String {
        self.interner.borrow().resolve(spur).to_string()
    }

    pub fn deep_clone_key(&mut self, k: ValueKey) -> StoredValue {
        let v = self.memory[k].clone();

        let value = match v.value {
            Value::Array(arr) => Value::Array(
                arr.into_iter()
                    .map(|v| self.deep_clone_key_insert(v))
                    .collect(),
            ),
            v => v,
        };

        StoredValue {
            value,
            area: v.area.clone(),
        }
    }

    pub fn deep_clone_key_insert(&mut self, k: ValueKey) -> ValueKey {
        let v = self.deep_clone_key(k);
        self.memory.insert(v)
    }

    pub fn deep_clone_reg(&mut self, reg: Register) -> StoredValue {
        self.deep_clone_key(self.contexts.current().registers.last().unwrap()[reg as usize])
    }

    pub fn deep_clone_reg_insert(&mut self, reg: Register) -> ValueKey {
        let v = self.deep_clone_reg(reg);
        self.memory.insert(v)
    }

    pub fn get_reg(&self, reg: Register) -> &StoredValue {
        &self.memory[self.contexts.current().registers.last().unwrap()[reg as usize]]
    }

    pub fn get_reg_key(&self, reg: Register) -> ValueKey {
        self.contexts.current().registers.last().unwrap()[reg as usize]
    }

    pub fn get_reg_mut(&mut self, reg: Register) -> &mut StoredValue {
        &mut self.memory[self.contexts.current_mut().registers.last().unwrap()[reg as usize]]
    }

    pub fn set_reg(&mut self, reg: Register, v: StoredValue) {
        self.memory[self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize]] = v
    }

    pub fn change_reg_key(&mut self, reg: Register, k: ValueKey) {
        self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize] = k
    }

    // pub fn set_reg_key(&mut self, reg: Register, k: ValueKey) {
    //     self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize] = k
    // }

    pub fn make_area(&self, span: CodeSpan, code: BytecodeKey) -> CodeArea {
        CodeArea {
            span,
            src: self.programs[code].src.clone(),
        }
    }

    pub fn get_span(&self, func: FuncCoord, i: usize) -> CodeSpan {
        self.programs[func.code].opcode_span_map[&(func.func, i)]
    }

    pub fn get_area(&self, func: FuncCoord, i: usize) -> CodeArea {
        self.make_area(self.get_span(func, i), func.code)
    }

    pub fn push_call_stack(
        &mut self,
        func: FuncCoord,
        return_dest: Register,
        increment_last: bool,
    ) {
        let regs_used = self.programs[func.code].functions[func.func].regs_used;
        let mut regs = Vec::with_capacity(regs_used);
        for _ in 0..regs_used {
            regs.push(self.memory.insert(StoredValue {
                value: Value::Empty,
                area: self.make_area(CodeSpan::invalid(), func.code),
            }))
        }
        {
            let mut current = self.contexts.current_mut();
            current.registers.push(regs);
            if increment_last {
                current.pos_stack.last_mut().unwrap().ip += 1;
            }
            current.pos_stack.push(CallStackItem {
                func,
                ip: 0,
                return_dest,
            });
            current.recursion_depth += 1;
        }
        //dbg!(&self.contexts);
    }

    pub fn return_and_pop_current(&mut self, ret_val: Option<StoredValue>) {
        if self.contexts.current().pos_stack.len() == 1 {
            self.contexts.yeet_current();
            return;
        }

        let item = {
            let mut current = self.contexts.current_mut();
            current.recursion_depth -= 1;
            current.registers.pop();
            current.pos_stack.pop().unwrap()
        };

        if let Some(ret_val) = ret_val {
            self.set_reg(item.return_dest, ret_val);
        } else {
            self.set_reg(
                item.return_dest,
                StoredValue {
                    value: Value::Empty,
                    area: self.make_area(CodeSpan::internal(), item.func.code), // probably gonna have to store the areas in the stack
                },
            );
        }
    }

    pub fn run_program(&mut self) -> RuntimeResult<()> {
        //self.push_call_stack(start, 0);
        while self.contexts.valid() {
            let &CallStackItem { func, ip, .. } = self.contexts.current().pos_stack.last().unwrap();
            let opcodes = &self.programs[func.code].functions[func.func].opcodes;

            if ip >= opcodes.len() {
                self.return_and_pop_current(None);
                continue;
            }
            let opcode = &opcodes[ip];

            match opcode {
                Opcode::LoadConst { dest, id } => {
                    let value =
                        Value::from_const(&self.programs[func.code].consts[*id as usize], self);

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value,
                            area: self.get_area(func, ip),
                        },
                    )
                }
                Opcode::Copy { from, to } => {
                    let v = self.deep_clone_reg(*from);
                    self.set_reg(*to, v)
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
                    let push = self.deep_clone_reg_insert(*elem);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Array(v) => v.push(push),
                        _ => unreachable!(),
                    }
                }
                Opcode::PushDictElem { elem, key, dest } => {
                    let push = self.deep_clone_reg_insert(*elem);

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
                    self.bin_op(value_ops::add, func, ip, left, right, dest)?
                }
                Opcode::Sub { left, right, dest } => {
                    self.bin_op(value_ops::sub, func, ip, left, right, dest)?
                }
                Opcode::Mult { left, right, dest } => {
                    self.bin_op(value_ops::mult, func, ip, left, right, dest)?
                }
                Opcode::Div { left, right, dest } => {
                    self.bin_op(value_ops::div, func, ip, left, right, dest)?
                }
                Opcode::Mod { left, right, dest } => {
                    self.bin_op(value_ops::modulo, func, ip, left, right, dest)?
                }
                Opcode::Pow { left, right, dest } => {
                    self.bin_op(value_ops::pow, func, ip, left, right, dest)?
                }
                Opcode::ShiftLeft { left, right, dest } => {
                    self.bin_op(value_ops::shift_left, func, ip, left, right, dest)?
                }
                Opcode::ShiftRight { left, right, dest } => {
                    self.bin_op(value_ops::shift_right, func, ip, left, right, dest)?
                }
                Opcode::BinOr { left, right, dest } => {
                    self.bin_op(value_ops::bin_or, func, ip, left, right, dest)?
                }
                Opcode::BinAnd { left, right, dest } => {
                    self.bin_op(value_ops::bin_and, func, ip, left, right, dest)?
                }

                Opcode::AddEq { left: _, right: _ } => todo!(),
                Opcode::SubEq { left: _, right: _ } => todo!(),
                Opcode::MultEq { left: _, right: _ } => todo!(),
                Opcode::DivEq { left: _, right: _ } => todo!(),
                Opcode::ModEq { left: _, right: _ } => todo!(),
                Opcode::PowEq { left: _, right: _ } => todo!(),
                Opcode::ShiftLeftEq { left: _, right: _ } => todo!(),
                Opcode::ShiftRightEq { left: _, right: _ } => todo!(),
                Opcode::BinAndEq { left: _, right: _ } => todo!(),
                Opcode::BinOrEq { left: _, right: _ } => todo!(),
                Opcode::BinNotEq { left: _, right: _ } => todo!(),
                Opcode::Not { src, dest } => {
                    self.unary_op(value_ops::unary_not, func, ip, src, dest)?
                }
                Opcode::Negate { src, dest } => {
                    self.unary_op(value_ops::unary_negate, func, ip, src, dest)?
                }

                Opcode::BinNot { src: _, dest: _ } => todo!(),

                Opcode::Eq { left, right, dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Bool(value_ops::equality(
                                &self.get_reg(*left).value,
                                &self.get_reg(*right).value,
                                self,
                                func.code,
                            )),
                            area: self.make_area(span, func.code),
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
                                func.code,
                            )),
                            area: self.make_area(span, func.code),
                        },
                    );
                }
                Opcode::Gt { left, right, dest } => {
                    self.bin_op(value_ops::gt, func, ip, left, right, dest)?
                }
                Opcode::Lt { left, right, dest } => {
                    self.bin_op(value_ops::lt, func, ip, left, right, dest)?
                }
                Opcode::Gte { left, right, dest } => {
                    self.bin_op(value_ops::gte, func, ip, left, right, dest)?
                }
                Opcode::Lte { left, right, dest } => {
                    self.bin_op(value_ops::lte, func, ip, left, right, dest)?
                }
                Opcode::Range { left, right, dest } => {
                    self.bin_op(value_ops::range, func, ip, left, right, dest)?
                }
                Opcode::In {
                    left: _,
                    right: _,
                    dest: _,
                } => todo!(),
                Opcode::As {
                    left: _,
                    right: _,
                    dest: _,
                } => todo!(),
                Opcode::Is {
                    left: _,
                    right: _,
                    dest: _,
                } => todo!(),
                Opcode::And { left, right, dest } => {
                    self.bin_op(value_ops::and, func, ip, left, right, dest)?
                }
                Opcode::Or { left, right, dest } => {
                    self.bin_op(value_ops::or, func, ip, left, right, dest)?
                }
                Opcode::Jump { to } => {
                    self.contexts.jump_current(*to as usize);
                    continue;
                }
                Opcode::JumpIfFalse { src, to } => {
                    let span = self.get_span(func, ip);
                    if !value_ops::to_bool(self.get_reg(*src), span, self, func.code)? {
                        self.contexts.jump_current(*to as usize);
                        continue;
                    }
                }
                Opcode::Ret { src } => {
                    let ret_val = self.deep_clone_reg(*src);
                    self.return_and_pop_current(Some(ret_val));
                    continue;
                }
                Opcode::WrapMaybe { src, dest } => {
                    let v = self.deep_clone_reg_insert(*src);
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Maybe(Some(v)),
                            area: self.make_area(span, func.code),
                        },
                    )
                }
                Opcode::LoadNone { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Maybe(None),
                            area: self.make_area(span, func.code),
                        },
                    )
                }
                Opcode::LoadEmpty { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Empty,
                            area: self.make_area(span, func.code),
                        },
                    )
                }
                Opcode::Index {
                    from: _,
                    dest: _,
                    index: _,
                } => todo!(),
                Opcode::Member {
                    from: _,
                    dest: _,
                    member: _,
                } => todo!(),
                Opcode::Associated {
                    from: _,
                    dest: _,
                    name: _,
                } => todo!(),
                Opcode::YeetContext => {
                    self.contexts.yeet_current();
                    continue;
                }
                Opcode::EnterArrowStatement { skip_to } => {
                    self.contexts.split_current();
                    self.contexts.jump_current(*skip_to as usize);
                }
                Opcode::LoadBuiltins { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Builtins,
                            area: self.make_area(span, func.code),
                        },
                    )
                }
                Opcode::Export { src: _ } => todo!(),
                Opcode::Call { args, base, dest } => {
                    let base = self.get_reg(*base);
                    let span = self.get_span(func, ip);
                    match base.value.clone() {
                        Value::Macro {
                            func,
                            args: arg_data,
                            captured,
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
                                                    call_area: self.make_area(span, func.code),
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
                                                            call_area: self
                                                                .make_area(span, func.code),
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
                            self.push_call_stack(func, *dest, true);

                            for (i, data) in arg_data.iter().enumerate() {
                                let v = match param_map[&data.name] {
                                    Some(k) => self.deep_clone_key(k),
                                    None => match data.default {
                                        Some(k) => self.deep_clone_key(k),
                                        None => {
                                            return Err(RuntimeError::ArgumentNotSatisfied {
                                                call_area: self.make_area(span, func.code),
                                                macro_def_area: base_area,
                                                arg_name: self.resolve(&data.name),
                                            })
                                        }
                                    },
                                };
                                self.set_reg(i as Register, v)
                            }

                            for (k, (_, to)) in captured
                                .iter()
                                .zip(&self.programs[func.code].functions[func.func].capture_regs)
                            {
                                self.change_reg_key(*to, *k)
                            }

                            // let ret_val = match self.run_program(func)? {
                            //     Some(v) => v,
                            //     None => StoredValue {
                            //         value: Value::Empty,
                            //         area: base_area,
                            //     },
                            // };
                            continue;
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (base.value.get_type(), base.area.clone()),
                                expected: ValueType::Macro,
                                area: self.make_area(span, func.code),
                            })
                        }
                    }
                }
                Opcode::CreateMacro { id, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Macro {
                            func: FuncCoord {
                                func: *id as usize,
                                code: func.code,
                            },
                            args: vec![],
                            captured: self.programs[func.code].functions[*id as usize]
                                .capture_regs
                                .iter()
                                .map(|(from, _)| self.get_reg_key(*from))
                                .collect(),
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
                    let set = self.deep_clone_reg_insert(*src);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro { args, .. } => args.last_mut().unwrap().default = Some(set),
                        _ => unreachable!(),
                    }
                }
                Opcode::SetMacroArgPattern { src, dest } => {
                    let set = self.deep_clone_reg_insert(*src);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro { args, .. } => args.last_mut().unwrap().pattern = Some(set),
                        _ => unreachable!(),
                    }
                }
                _ => todo!(),
            }

            // increment ip
            // TODO: implicit return shit
            {
                let mut current = self.contexts.current_mut();
                let ip = &mut current.pos_stack.last_mut().unwrap().ip;
                *ip += 1;
            };
        }

        Ok(())
    }

    #[inline]
    fn bin_op<F>(
        &mut self,
        op: F,
        func: FuncCoord,
        ip: usize,
        left: &u8,
        right: &u8,
        dest: &u8,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&StoredValue, &StoredValue, CodeSpan, &Vm, BytecodeKey) -> RuntimeResult<Value>,
    {
        let span = self.get_span(func, ip);
        let value = op(
            self.get_reg(*left),
            self.get_reg(*right),
            span,
            self,
            func.code,
        )?;

        self.set_reg(
            *dest,
            StoredValue {
                value,
                area: self.make_area(span, func.code),
            },
        );
        Ok(())
    }

    #[inline]
    fn unary_op<F>(
        &mut self,
        op: F,
        func: FuncCoord,
        ip: usize,
        value: &u8,
        dest: &u8,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&StoredValue, CodeSpan, &Vm, BytecodeKey) -> RuntimeResult<Value>,
    {
        let span = self.get_span(func, ip);
        let value = op(self.get_reg(*value), span, self, func.code)?;

        self.set_reg(
            *dest,
            StoredValue {
                value,
                area: self.make_area(span, func.code),
            },
        );
        Ok(())
    }

    pub fn next_id(&mut self, c: IDClass) -> u16 {
        self.id_counters[c as usize] += 1;
        self.id_counters[c as usize] as u16
    }
}

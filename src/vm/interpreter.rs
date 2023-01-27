use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use ahash::AHashMap;
use lasso::Spur;
use slotmap::{new_key_type, SlotMap};

use super::context::{CallKey, CallStackItem, FullContext};
use super::error::RuntimeError;
use super::opcodes::{Opcode, Register};
use super::value::{ArgData, StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::Bytecode;
use crate::gd::ids::IDClass;
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, SpwnSource};
use crate::util::Interner;
use crate::vm::builtins::Builtin;
use crate::vm::value::MacroCode;
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

    pub programs: SlotMap<BytecodeKey, (SpwnSource, &'a Bytecode<Register>)>,
    pub src_map: AHashMap<SpwnSource, BytecodeKey>,

    pub interner: Rc<RefCell<Interner>>,

    pub id_counters: [usize; 4],

    pub contexts: FullContext,
}

impl<'a> Vm<'a> {
    pub fn new(bytecode_map: &'a BytecodeMap, interner: Rc<RefCell<Interner>>) -> Vm<'a> {
        let mut programs = SlotMap::default();
        let mut src_map = AHashMap::new();

        for (src, bytecode) in &bytecode_map.map {
            let k = programs.insert((src.clone(), bytecode));
            src_map.insert(src.clone(), k);
        }

        Self {
            memory: SlotMap::default(),
            interner,
            programs,
            id_counters: [0; 4],
            contexts: FullContext::new(),
            src_map,
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

    // please only use for "mutating" something, otherwise context fuckery
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
        // todo!()
        CodeArea {
            span,
            src: self.programs[code].0.clone(),
        }
    }

    pub fn get_span(&self, func: FuncCoord, i: usize) -> CodeSpan {
        self.programs[func.code].1.opcode_span_map[&(func.func, i)]
    }

    pub fn get_area(&self, func: FuncCoord, i: usize) -> CodeArea {
        self.make_area(self.get_span(func, i), func.code)
    }

    pub fn get_call_stack(&self) -> Vec<CallStackItem> {
        self.contexts.current().pos_stack.iter().cloned().collect()
    }

    pub fn push_call_stack(
        &mut self,
        func: FuncCoord,
        return_dest: Register,
        increment_last: bool,
        call_area: Option<CodeArea>,
    ) {
        let regs_used = self.programs[func.code].1.functions[func.func].regs_used;

        let mut regs = Vec::with_capacity(regs_used);

        for _ in 0..regs_used {
            regs.push(self.memory.insert(StoredValue {
                value: Value::Empty,
                area: self.make_area(CodeSpan::invalid(), func.code),
            }))
        }

        let call_key = self.contexts.have_not_returned.insert(());

        let mut current = self.contexts.current_mut();
        current.registers.push(regs);

        if increment_last {
            current.pos_stack.last_mut().unwrap().ip += 1;
        }

        current.pos_stack.push(CallStackItem {
            func,
            ip: 0,
            return_dest,
            call_key,
            call_area,
        });
        current.recursion_depth += 1;

        //dbg!(&self.contexts);
    }

    pub fn return_and_pop_current(&mut self, ret_val: Option<StoredValue>) -> Option<CallKey> {
        if self.contexts.current().pos_stack.len() == 1 {
            self.contexts.yeet_current();
            return None;
        }

        let mut current = self.contexts.current_mut();
        current.recursion_depth -= 1;
        current.registers.pop();
        let item = current.pos_stack.pop().unwrap();

        let ret_val = if let Some(ret_val) = ret_val {
            ret_val
        } else {
            StoredValue {
                value: Value::Empty,
                area: item.call_area.or(Some(CodeArea::internal())).unwrap(),
            }
        };

        self.memory[current.registers.last_mut().unwrap()[item.return_dest as usize]] = ret_val;

        Some(item.call_key)
    }

    pub fn run_program(&mut self) -> RuntimeResult<()> {
        //self.push_call_stack(start, 0);
        while self.contexts.valid() {
            let &CallStackItem {
                func, ip, call_key, ..
            } = self.contexts.current().pos_stack.last().unwrap();
            let opcodes = &self.programs[func.code].1.functions[func.func].opcodes;

            if ip >= opcodes.len() {
                if self.contexts.have_not_returned.contains_key(call_key) {
                    // implicit return
                    self.return_and_pop_current(None);
                } else {
                    // implicit yeet
                    self.contexts.yeet_current();
                }
                continue;
            }
            let opcode = &opcodes[ip];

            // println!(
            //     "{} - {opcode}",
            //     <&Opcode<Register> as Into<&'static str>>::into(opcode).green()
            // );

            match opcode {
                Opcode::LoadConst { dest, id } => {
                    let value =
                        Value::from_const(&self.programs[func.code].1.consts[*id as usize], self);

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
                    let Some(call_key) = self.return_and_pop_current(Some(ret_val)) else { continue };
                    self.contexts.have_not_returned.remove(call_key);
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
                Opcode::Index { base, dest, index } => {
                    let span = self.get_span(func, ip);

                    let base = self.get_reg(*base);
                    let index = self.get_reg(*index);

                    let index_wrap = |idx: i64, len: usize, typ: ValueType| {
                        let index_calc = if idx >= 0 { idx } else { len as i64 + idx };

                        if index_calc < 0 || index_calc >= len as i64 {
                            return Err(RuntimeError::IndexOutOfBounds {
                                len,
                                index: idx,
                                area: self.make_area(span, func.code),
                                typ,
                                call_stack: self.get_call_stack(),
                            });
                        }

                        Ok(index_calc as usize)
                    };

                    match (&base.value, &index.value) {
                        (Value::Array(v), Value::Int(index)) => {
                            let k = v[index_wrap(*index, v.len(), ValueType::Array)?];

                            self.change_reg_key(*dest, k);
                        }
                        (Value::String(s), Value::Int(index)) => {
                            let idx = index_wrap(*index, s.chars().count(), ValueType::String)?;
                            let c = s.chars().nth(idx).unwrap();

                            self.set_reg(
                                *dest,
                                StoredValue {
                                    value: Value::String(c.into()),
                                    area: self.make_area(span, func.code),
                                },
                            );
                        }
                        (Value::Dict(v), Value::String(s)) => {
                            let key_interned = self.interner.borrow_mut().get_or_intern(s);
                            match v.get(&key_interned) {
                                Some(k) => self.change_reg_key(*dest, *k),
                                None => {
                                    return Err(RuntimeError::NonexistentMember {
                                        area: self.make_area(span, func.code),
                                        member: s.clone(),
                                        base_type: base.value.get_type(),
                                        call_stack: self.get_call_stack(),
                                    })
                                }
                            }
                        }
                        _ => {
                            return Err(RuntimeError::InvalidIndex {
                                base: (base.value.get_type(), base.area.clone()),
                                index: (index.value.get_type(), index.area.clone()),
                                area: self.make_area(span, func.code),
                                call_stack: self.get_call_stack(),
                            })
                        }
                    };
                }
                Opcode::Member { from, dest, member } => {
                    let key = match &self.get_reg(*member).value {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    let span = self.get_span(func, ip);
                    let value = &self.get_reg(*from).value;

                    let special = match (value, &key[..]) {
                        (Value::String(s), "length") => Some(Value::Int(s.chars().count() as i64)),

                        (Value::Range(start, ..), "start") => Some(Value::Int(*start)),
                        (Value::Range(_, end, _), "end") => Some(Value::Int(*end)),
                        (Value::Range(_, _, step), "step") => Some(Value::Int(*step as i64)),

                        (Value::Array(v), "length") => Some(Value::Int(v.len() as i64)),
                        (Value::Dict(v), "length") => Some(Value::Int(v.len() as i64)),

                        (Value::Builtins, name) => Some(Value::Macro(MacroCode::Builtin(
                            Builtin::from_str(name).unwrap(),
                        ))),
                        _ => None,
                    };

                    macro_rules! error {
                        () => {
                            return Err(RuntimeError::NonexistentMember {
                                area: self.make_area(span, func.code),
                                member: key,
                                base_type: value.get_type(),
                                call_stack: self.get_call_stack(),
                            })
                        };
                    }

                    if let Some(v) = special {
                        self.set_reg(
                            *dest,
                            StoredValue {
                                value: v,
                                area: self.make_area(span, func.code),
                            },
                        );
                    } else {
                        let key_interned = self.interner.borrow_mut().get_or_intern(&key);
                        match value {
                            Value::Dict(v) => match v.get(&key_interned) {
                                Some(k) => self.change_reg_key(*dest, *k),
                                None => error!(),
                            },
                            _ => error!(),
                        }
                    }
                }
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
                    self.split_current_context();
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
                    let call_area = self.get_area(func, ip);
                    match base.value.clone() {
                        Value::Macro(MacroCode::Normal {
                            func,
                            args: arg_data,
                            captured,
                        }) => {
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
                                                    call_area,
                                                    macro_def_area: base.area.clone(),
                                                    macro_arg_amount: arg_data.len(),
                                                    call_arg_amount: v.len(),
                                                    call_stack: self.get_call_stack(),
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
                                                            call_area,
                                                            macro_def_area: base.area.clone(),
                                                            arg_name: self.resolve(name),
                                                            call_stack: self.get_call_stack(),
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
                            self.push_call_stack(func, *dest, true, Some(call_area.clone()));

                            for (i, data) in arg_data.iter().enumerate() {
                                let v = match param_map[&data.name] {
                                    Some(k) => self.deep_clone_key(k),
                                    None => match data.default {
                                        Some(k) => self.deep_clone_key(k),
                                        None => {
                                            return Err(RuntimeError::ArgumentNotSatisfied {
                                                call_area,
                                                macro_def_area: base_area,
                                                arg_name: self.resolve(&data.name),
                                                call_stack: self.get_call_stack(),
                                            })
                                        }
                                    },
                                };
                                self.set_reg(i as Register, v)
                            }

                            for (k, (_, to)) in captured
                                .iter()
                                .zip(&self.programs[func.code].1.functions[func.func].capture_regs)
                            {
                                self.change_reg_key(*to, *k)
                            }

                            continue;
                        }
                        Value::Macro(MacroCode::Builtin(b)) => {
                            let mut args = match &self.get_reg(*args).value {
                                Value::Array(v) => {
                                    match &self.memory[v[1]].value {
                                        Value::Dict(m) => {
                                            if !m.is_empty() {
                                                todo!()
                                            }
                                        }
                                        _ => unreachable!(),
                                    }
                                    match &self.memory[v[0]].value {
                                        Value::Array(v) => v.clone(),
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            };

                            //args.reverse();

                            let value = b.call(&mut args, self);
                            let span = self.get_span(func, ip);

                            self.set_reg(
                                *dest,
                                StoredValue {
                                    value,
                                    area: self.make_area(span, func.code),
                                },
                            )
                        }

                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (base.value.get_type(), base.area.clone()),
                                expected: ValueType::Macro,
                                area: call_area,
                                call_stack: self.get_call_stack(),
                            })
                        }
                    }
                }
                Opcode::CreateMacro { id, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Macro(MacroCode::Normal {
                            func: FuncCoord {
                                func: *id as usize,
                                code: func.code,
                            },
                            args: vec![],
                            captured: self.programs[func.code].1.functions[*id as usize]
                                .capture_regs
                                .iter()
                                .map(|(from, _)| self.get_reg_key(*from))
                                .collect(),
                        }),
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
                        Value::Macro(MacroCode::Normal { args, .. }) => args.push(ArgData {
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
                        Value::Macro(MacroCode::Normal { args, .. }) => {
                            args.last_mut().unwrap().default = Some(set)
                        }
                        _ => unreachable!(),
                    }
                }
                Opcode::SetMacroArgPattern { src, dest } => {
                    let set = self.deep_clone_reg_insert(*src);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro(MacroCode::Normal { args, .. }) => {
                            args.last_mut().unwrap().pattern = Some(set)
                        }
                        _ => unreachable!(),
                    }
                }
                Opcode::Import { src, dest } => {
                    let import = &self.programs[func.code].1.import_paths[*src as usize];

                    let rel_path = import.value.to_path_name().1;
                    let SpwnSource::File(current_path) = &self.programs[func.code].0;

                    let src = SpwnSource::File(current_path.parent().unwrap().join(rel_path));
                    let coord = FuncCoord {
                        func: 0,
                        code: self.src_map[&src],
                    };

                    self.push_call_stack(coord, *dest, true, None);
                    continue;
                }
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

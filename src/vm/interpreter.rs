use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use colored::Colorize;
use lasso::Spur;
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::context::{CallInfo, Context, ContextStack, FullContext};
use super::error::RuntimeError;
use super::opcodes::{Opcode, Register};
use super::value::{IteratorData, MacroTarget, StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::Bytecode;
use crate::compiling::compiler::{CustomTypeKey, TypeDef};
use crate::gd::gd_object::{GdObject, Trigger, TriggerOrder};
use crate::gd::ids::{IDClass, Id, SpecificId};
use crate::gd::object_keys::ObjectKeyValueType;
use crate::parsing::ast::{MacroArg, ObjectType, Spannable, Spanned};
use crate::parsing::utils::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, SpwnSource};
use crate::util::Interner;
use crate::vm::value::MacroData;
use crate::vm::value_ops as vo;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

new_key_type! {
    pub struct ValueKey;
    pub struct BytecodeKey;
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

    pub programs:
        SlotMap<BytecodeKey, (SpwnSource, &'a Bytecode<Register>, Vec<(CustomTypeKey, bool)>)>,
    pub src_map: AHashMap<SpwnSource, BytecodeKey>,

    pub interner: Rc<RefCell<Interner>>,

    pub id_counters: [usize; 4],

    pub contexts: ContextStack,
    pub objects: Vec<GdObject>,
    pub triggers: Vec<Trigger>,
    pub trigger_order_count: TriggerOrder,

    pub types: SecondaryMap<CustomTypeKey, Spanned<TypeDef>>,

    pub impls: AHashMap<ValueType, AHashMap<Spur, (ValueKey, Visibility)>>,
    pub overloads: AHashMap<Operator, Vec<ValueKey>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private(SpwnSource),
}

impl<'a> Vm<'a> {
    pub fn new(
        bytecode_map: &'a BytecodeMap,
        interner: Rc<RefCell<Interner>>,
        type_defs: AHashMap<TypeDef, Spanned<CustomTypeKey>>,
    ) -> Vm<'a> {
        let mut programs = SlotMap::default();
        let mut src_map = AHashMap::new();

        let mut type_src_map: AHashMap<_, Vec<(CustomTypeKey, bool)>> = AHashMap::new();

        for (
            TypeDef {
                def_src, private, ..
            },
            k,
        ) in &type_defs
        {
            type_src_map
                .entry(def_src)
                .and_modify(|v| v.push((k.value, *private)))
                .or_insert_with(|| vec![(k.value, *private)]);
        }

        for (src, bytecode) in &bytecode_map.map {
            let k = programs.insert((
                src.clone(),
                bytecode,
                type_src_map.remove(src).unwrap_or_default(),
            ));
            src_map.insert(src.clone(), k);
        }

        let mut types = SecondaryMap::new();

        for (info, k) in type_defs {
            types.insert(k.value, info.clone().spanned(k.span));
        }

        Self {
            memory: SlotMap::default(),
            interner,
            programs,
            id_counters: [0; 4],
            contexts: ContextStack(vec![]),
            src_map,
            objects: Vec::new(),
            triggers: Vec::new(),
            types,
            impls: AHashMap::new(),
            overloads: AHashMap::new(),
            trigger_order_count: TriggerOrder::new(),
        }
    }

    pub fn resolve(&self, spur: &Spur) -> String {
        self.interner.borrow().resolve(spur).to_string()
    }

    pub fn intern(&self, s: &str) -> Spur {
        self.interner.borrow_mut().get_or_intern(s)
    }

    pub fn intern_vec(&self, s: &[char]) -> Spur {
        let s: String = s.iter().collect();
        self.intern(&s)
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
        // println!(
        //     "alulu {} {:?} ",
        //     reg,
        //     self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize]
        // );
        self.memory[self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize]] = v
    }

    pub fn change_reg_key(&mut self, reg: Register, k: ValueKey) {
        self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize] = k
    }

    pub fn reset_reg(&mut self, reg: Register, func: FuncCoord) {
        let k = self.memory.insert(StoredValue {
            value: Value::Empty,
            area: self.make_area(CodeSpan::invalid(), func.code),
        });
        self.change_reg_key(reg, k);
    }

    // pub fn set_reg_key(&mut self, reg: Register, k: ValueKey) {
    //     self.contexts.current_mut().registers.last_mut().unwrap()[reg as usize] = k
    // }

    pub fn make_area(&self, span: CodeSpan, code: BytecodeKey) -> CodeArea {
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

    pub fn get_call_stack(&self) -> Vec<CallInfo> {
        self.contexts
            .0
            .iter()
            .map(|f| &f.call_info)
            .cloned()
            .collect()
    }

    // pub fn push_call_stack(
    //     &mut self,
    //     func: FuncCoord,
    //     return_dest: Option<Register>,
    //     increment_last: bool,
    //     call_area: Option<CodeArea>,
    // ) {
    //     let regs_used = self.programs[func.code].1.functions[func.func].regs_used;

    //     let mut regs = Vec::with_capacity(regs_used);

    //     for _ in 0..regs_used {
    //         regs.push(self.memory.insert(StoredValue {
    //             value: Value::Empty,
    //             area: self.make_area(CodeSpan::invalid(), func.code),
    //         }))
    //     }

    //     let call_key = self.contexts.have_not_returned.insert(());

    //     let mut current = self.contexts.current_mut();
    //     current.registers.push(regs);

    //     if increment_last {
    //         current.pos_stack.last_mut().unwrap().ip += 1;
    //     }

    //     current.pos_stack.push(CallInfo {
    //         func,
    //         ip: 0,
    //         return_dest,
    //         call_key,
    //         call_area,
    //     });
    //     current.recursion_depth += 1;
    // }

    // pub fn return_and_pop_current(&mut self, ret_val: Option<StoredValue>) -> Option<CallKey> {
    //     if self.contexts.current().pos_stack.len() == 1 {
    //         self.contexts.yeet_current();
    //         return None;
    //     }

    //     let mut current = self.contexts.current_mut();
    //     current.recursion_depth -= 1;
    //     current.registers.pop();
    //     let item = current.pos_stack.pop().unwrap();

    //     let ret_val = if let Some(ret_val) = ret_val {
    //         ret_val
    //     } else {
    //         StoredValue {
    //             value: Value::Empty,
    //             area: item.call_area.unwrap_or_else(CodeArea::internal),
    //         }
    //     };

    //     if let Some(r) = item.return_dest {
    //         self.memory[current.registers.last_mut().unwrap()[r as usize]] = ret_val;
    //     }

    //     Some(item.call_key)
    // }

    pub fn run_function(
        &mut self,
        mut context: Context,
        call_info: CallInfo,
        cb: Box<dyn FnOnce(&mut Vm) -> RuntimeResult<()>>,
    ) -> RuntimeResult<()> {
        let &CallInfo {
            func, return_dest, ..
        } = &call_info;

        let original_ip = context.ip;
        context.ip = 0;

        {
            let regs_used = self.programs[func.code].1.functions[func.func].regs_used;

            let mut regs = Vec::with_capacity(regs_used);

            for _ in 0..regs_used {
                regs.push(self.memory.insert(StoredValue {
                    value: Value::Empty,
                    area: self.make_area(CodeSpan::invalid(), func.code),
                }))
            }

            context.registers.push(regs);
        }

        self.contexts.0.push(FullContext::new(context, call_info));
        // dbg!(self.contexts.0.len(), return_dest);

        cb(self)?;

        let opcodes = &self.programs[func.code].1.functions[func.func].opcodes;

        while self.contexts.valid() {
            let ip = self.contexts.ip();
            println!("gagaba {}: {}", func.func, ip);

            if ip >= opcodes.len() {
                if !self.contexts.last().have_returned {
                    return Ok(());
                } else {
                    self.contexts.yeet_current();
                }
                continue;
            }
            let opcode = &opcodes[ip];

            match opcode {
                Opcode::LoadConst { dest, id } => {
                    let area = self.get_area(func, ip);
                    let value = Value::from_const(
                        &self.programs[func.code].1.consts[*id as usize],
                        self,
                        &area,
                    );

                    self.set_reg(*dest, StoredValue { value, area })
                }
                Opcode::Copy { from, to } => {
                    println!("galha {} {} {}", self.contexts.0.len(), ip, func.func);
                    let v = self.deep_clone_reg(*from);
                    self.set_reg(*to, v)
                }
                Opcode::Dbg { reg } => {
                    println!(
                        "{}, {} | {:?} | {:?}",
                        self.get_reg(*reg).value.runtime_display(self),
                        self.contexts.group().fmt("g").green(),
                        self.get_reg(*reg).value,
                        self.get_reg_key(*reg),
                    )
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

                    let key = self.intern_vec(&key);

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Dict(v) => {
                            v.insert(key, (push, Visibility::Public));
                        }
                        _ => unreachable!(),
                    }
                }
                Opcode::PushArrayElemByKey { elem, dest } => {
                    let push = self.get_reg_key(*elem);
                    println!("bavi {:?}", self.get_reg_mut(*dest).value);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Array(v) => v.push(push),
                        _ => unreachable!(),
                    }
                }
                Opcode::PushDictElemByKey { elem, key, dest } => {
                    let push = self.get_reg_key(*elem);

                    let key = match &self.get_reg(*key).value {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };

                    let key = self.intern_vec(&key);

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Dict(v) => {
                            v.insert(key, (push, Visibility::Public));
                        }
                        _ => unreachable!(),
                    }
                }

                Opcode::MakeDictElemPrivate { dest, key } => {
                    let key = match &self.get_reg(*key).value {
                        Value::String(s) => s.clone(),
                        _ => unreachable!(),
                    };

                    let key = self.intern_vec(&key);

                    let new_vis = Visibility::Private(self.programs[func.code].0.clone());

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Dict(v) => {
                            v.entry(key).and_modify(|(_, p)| *p = new_vis);
                        }
                        _ => unreachable!(),
                    }
                }

                Opcode::AllocObject { size, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Object(
                            AHashMap::with_capacity(*size as usize),
                            ObjectType::Object,
                        ),
                        area: self.get_area(func, ip),
                    },
                ),
                Opcode::AllocTrigger { size, dest } => self.set_reg(
                    *dest,
                    StoredValue {
                        value: Value::Object(
                            AHashMap::with_capacity(*size as usize),
                            ObjectType::Trigger,
                        ),
                        area: self.get_area(func, ip),
                    },
                ),

                Opcode::MakeByteArray { reg } => {
                    let area = self.get_area(func, ip);

                    let s = match &self.get_reg(*reg).value {
                        Value::String(s) => s.iter().collect::<String>(),
                        _ => unreachable!(),
                    };

                    let val = Value::Array(
                        s.bytes()
                            .map(|c| {
                                self.memory.insert(StoredValue {
                                    value: Value::Int(c as i64),
                                    area: area.clone(),
                                })
                            })
                            .collect(),
                    );

                    self.set_reg(*reg, StoredValue { value: val, area })
                }

                Opcode::PushObjectElemKey {
                    elem,
                    obj_key,
                    dest,
                } => {
                    // Objec
                    let push = self.deep_clone_reg_insert(*elem);

                    let param = {
                        let types = obj_key.types();

                        let mut valid = false;

                        for t in types {
                            match (t, &self.memory[push].value) {
                                (ObjectKeyValueType::Int, Value::Int(_))
                                | (ObjectKeyValueType::Float, Value::Float(_) | Value::Int(_))
                                | (ObjectKeyValueType::Bool, Value::Bool(_))
                                | (
                                    ObjectKeyValueType::Group,
                                    Value::Group(_) | Value::TriggerFunction { .. },
                                )
                                | (ObjectKeyValueType::Channel, Value::Channel(_))
                                | (ObjectKeyValueType::Block, Value::Block(_))
                                | (ObjectKeyValueType::Item, Value::Item(_))
                                | (ObjectKeyValueType::String, Value::String(_))
                                | (ObjectKeyValueType::Epsilon, Value::Epsilon) => {
                                    valid = true;
                                    break;
                                }

                                (ObjectKeyValueType::GroupArray, Value::Array(v))
                                    if v.iter().all(|k| {
                                        matches!(&self.memory[*k].value, Value::Group(_))
                                    }) =>
                                {
                                    valid = true;
                                    break;
                                }

                                _ => (),
                            }
                        }

                        if !valid {
                            println!("{:?} {:?}", types, &self.memory[push].value);
                            todo!()
                            //panic!("\n\nOk   heres the deal!!! I not this yet XDXDC😭😭🤣🤣 \nLOl")
                        }

                        vo::to_obj_param(
                            &self.memory[push],
                            self.get_span(func, ip),
                            self,
                            func.code,
                        )?
                    };

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Object(v, _) => {
                            v.insert(obj_key.id(), param);
                        }
                        _ => unreachable!(),
                    }
                }
                Opcode::PushObjectElemUnchecked {
                    elem,
                    obj_key,
                    dest,
                } => {
                    // Objec
                    let push = self.deep_clone_reg_insert(*elem);

                    let param = vo::to_obj_param(
                        &self.memory[push],
                        self.get_span(func, ip),
                        self,
                        func.code,
                    )?;

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Object(v, _) => {
                            v.insert(*obj_key, param);
                        }
                        _ => unreachable!(),
                    }
                }

                Opcode::Add { left, right, dest } => {
                    self.bin_op(vo::add, func, ip, left, right, dest, BinOp::Plus)?
                }
                Opcode::Sub { left, right, dest } => {
                    self.bin_op(vo::sub, func, ip, left, right, dest, BinOp::Minus)?
                }
                Opcode::Mult { left, right, dest } => {
                    self.bin_op(vo::mult, func, ip, left, right, dest, BinOp::Mult)?
                }
                Opcode::Div { left, right, dest } => {
                    self.bin_op(vo::div, func, ip, left, right, dest, BinOp::Div)?
                }
                Opcode::Mod { left, right, dest } => {
                    self.bin_op(vo::modulo, func, ip, left, right, dest, BinOp::Mod)?
                }
                Opcode::Pow { left, right, dest } => {
                    self.bin_op(vo::pow, func, ip, left, right, dest, BinOp::Pow)?
                }
                Opcode::ShiftLeft { left, right, dest } => {
                    self.bin_op(vo::shift_left, func, ip, left, right, dest, BinOp::ShiftLeft)?
                }
                Opcode::ShiftRight { left, right, dest } => {
                    self.bin_op(vo::shift_right, func, ip, left, right, dest, BinOp::ShiftRight)?
                }
                Opcode::BinOr { left, right, dest } => {
                    self.bin_op(vo::bin_or, func, ip, left, right, dest, BinOp::BinOr)?
                }
                Opcode::BinAnd { left, right, dest } => {
                    self.bin_op(vo::bin_and, func, ip, left, right, dest, BinOp::BinAnd)?
                }

                Opcode::AddEq { left, right } => {
                    self.assign_op(vo::add, func, ip, left, right, AssignOp::PlusEq)?
                }
                Opcode::SubEq { left, right } => {
                    self.assign_op(vo::sub, func, ip, left, right, AssignOp::MinusEq)?
                }
                Opcode::MultEq { left, right } => {
                    self.assign_op(vo::mult, func, ip, left, right, AssignOp::MultEq)?
                }
                Opcode::DivEq { left, right } => {
                    self.assign_op(vo::div, func, ip, left, right, AssignOp::DivEq)?
                }
                Opcode::ModEq { left, right } => {
                    self.assign_op(vo::modulo, func, ip, left, right, AssignOp::ModEq)?
                }
                Opcode::PowEq { left, right } => {
                    self.assign_op(vo::pow, func, ip, left, right, AssignOp::PowEq)?
                }
                Opcode::ShiftLeftEq { left, right } => {
                    self.assign_op(vo::shift_left, func, ip, left, right, AssignOp::ShiftLeftEq)?
                }
                Opcode::ShiftRightEq { left, right } => {
                    self.assign_op(vo::shift_right, func, ip, left, right, AssignOp::ShiftRightEq)?
                }
                Opcode::BinAndEq { left, right } => {
                    self.assign_op(vo::bin_and, func, ip, left, right, AssignOp::BinAndEq)?
                }
                Opcode::BinOrEq { left, right } => {
                    self.assign_op(vo::bin_or, func, ip, left, right, AssignOp::BinOrEq)?
                }

                Opcode::Not { src, dest } => {
                    self.unary_op(vo::unary_not, func, ip, src, dest, UnaryOp::ExclMark)?
                }
                Opcode::Negate { src, dest } => {
                    self.unary_op(vo::unary_negate, func, ip, src, dest, UnaryOp::Minus)?
                }

                Opcode::BinNot { src: _, dest: _ } => todo!(),

                Opcode::Eq { left, right, dest } => {
                    self.bin_op(vo::eq_op, func, ip, left, right, dest, BinOp::Mod)?
                }
                Opcode::Neq { left, right, dest } => {
                    self.bin_op(vo::neq_op, func, ip, left, right, dest, BinOp::Mod)?
                }
                Opcode::Gt { left, right, dest } => {
                    self.bin_op(vo::gt, func, ip, left, right, dest, BinOp::Gt)?
                }
                Opcode::Lt { left, right, dest } => {
                    self.bin_op(vo::lt, func, ip, left, right, dest, BinOp::Lt)?
                }
                Opcode::Gte { left, right, dest } => {
                    self.bin_op(vo::gte, func, ip, left, right, dest, BinOp::Gte)?
                }
                Opcode::Lte { left, right, dest } => {
                    self.bin_op(vo::lte, func, ip, left, right, dest, BinOp::Lte)?
                }
                Opcode::Range { left, right, dest } => {
                    self.bin_op(vo::range, func, ip, left, right, dest, BinOp::Range)?
                }
                Opcode::In {
                    left: _,
                    right: _,
                    dest: _,
                } => todo!(),
                Opcode::As { left, right, dest } => {
                    let span = self.get_span(func, ip);

                    if !self.run_overload(
                        Operator::Bin(BinOp::As),
                        [*left, *right],
                        func,
                        span,
                        Some(*dest),
                    )? {
                        let value = vo::as_op(
                            &self.get_reg(*left).clone(),
                            &self.get_reg(*right).clone(),
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
                    };
                }
                Opcode::And { left, right, dest } => {
                    self.bin_op(vo::and, func, ip, left, right, dest, BinOp::And)?
                }
                Opcode::Or { left, right, dest } => {
                    self.bin_op(vo::or, func, ip, left, right, dest, BinOp::Or)?
                }
                Opcode::Jump { to } => {
                    self.contexts.jump_current(*to as usize);
                    continue;
                }
                Opcode::JumpIfFalse { src, to } => {
                    let span = self.get_span(func, ip);
                    if !vo::to_bool(self.get_reg(*src), span, self, func.code)? {
                        self.contexts.jump_current(*to as usize);
                        continue;
                    }
                }
                Opcode::Ret { src, module_ret } => {
                    let mut ret_val = self.deep_clone_reg(*src);

                    if *module_ret {
                        match ret_val.value {
                            Value::Dict(d) => {
                                // let module_name = self.programs[func.code].0.name()

                                ret_val.value = Value::Module {
                                    exports: d.into_iter().map(|(s, (k, _))| (s, k)).collect(),
                                    types: self.programs[func.code].2.to_vec(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    }

                    self.contexts.last_mut().have_returned = true;

                    let return_dest = self.contexts.last().call_info.return_dest;
                    {
                        let mut current = self.contexts.current_mut();
                        current.registers.pop();

                        if let Some(r) = return_dest {
                            self.memory[current.registers.last_mut().unwrap()[r as usize]] =
                                ret_val;
                        }
                    }

                    let mut top = self.contexts.last_mut().yeet_current().unwrap();
                    top.ip = original_ip + 1;

                    if return_dest.is_some() {
                        // dbg!(&self.contexts);
                        let idx = self.contexts.0.len() - 2;
                        self.contexts.0[idx].contexts.push(top);
                    }

                    continue;

                    // let Some(call_key) = self.return_and_pop_current(Some(ret_val)) else { continue };
                    // self.contexts.have_not_returned.remove(call_key);
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
                Opcode::LoadEmptyDict { dest } => {
                    let span = self.get_span(func, ip);
                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Dict(AHashMap::new()),
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
                            let idx = index_wrap(*index, s.len(), ValueType::String)?;
                            let c = s[idx];

                            self.set_reg(
                                *dest,
                                StoredValue {
                                    value: Value::String(vec![c]),
                                    area: self.make_area(span, func.code),
                                },
                            );
                        }
                        (Value::Dict(v), Value::String(s)) => {
                            let key_interned = self.intern_vec(s);
                            match v.get(&key_interned) {
                                Some((k, _)) => self.change_reg_key(*dest, *k),
                                None => {
                                    return Err(RuntimeError::NonexistentMember {
                                        area: self.make_area(span, func.code),
                                        member: s.iter().collect(),
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
                    let key: String = match &self.get_reg(*member).value {
                        Value::String(s) => s.iter().collect(),
                        _ => unreachable!(),
                    };
                    let span = self.get_span(func, ip);

                    let value = &self.get_reg(*from).value;

                    let special = match (value, &key[..]) {
                        (Value::String(s), "length") => Some(Value::Int(s.len() as i64)),

                        (Value::Range(start, ..), "start") => Some(Value::Int(*start)),
                        (Value::Range(_, end, _), "end") => Some(Value::Int(*end)),
                        (Value::Range(_, _, step), "step") => Some(Value::Int(*step as i64)),

                        (Value::Array(v), "length") => Some(Value::Int(v.len() as i64)),
                        (Value::Dict(v), "length") => Some(Value::Int(v.len() as i64)),

                        _ => None,
                    };

                    macro_rules! error {
                        ($type:ident) => {
                            return Err(RuntimeError::NonexistentMember {
                                area: self.make_area(span, func.code),
                                member: key.into(),
                                base_type: $type,
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
                        let key_interned = self.intern(&key);
                        let base_type = value.get_type();

                        let mut found = false;

                        match value {
                            Value::Dict(v) => {
                                if let Some((k, _)) = v.get(&key_interned) {
                                    self.change_reg_key(*dest, *k);
                                    found = true;
                                }
                            }
                            Value::Instance { items, .. } => {
                                if let Some((k, vis)) = items.get(&key_interned) {
                                    if let Visibility::Private(src) = vis {
                                        if src != &self.programs[func.code].0 {
                                            return Err(RuntimeError::PrivateMemberAccess {
                                                area: self.make_area(span, func.code),
                                                member: key,
                                                call_stack: self.get_call_stack(),
                                            });
                                        }
                                    }

                                    self.change_reg_key(*dest, *k);
                                    found = true;
                                }
                            }
                            Value::Module { exports, .. } => {
                                if let Some(k) = exports.get(&key_interned) {
                                    self.change_reg_key(*dest, *k);
                                    found = true;
                                }
                            }
                            _ => (),
                        }

                        if !found {
                            let Some(members) = self.impls.get(&base_type) else { error!(base_type) };
                            let Some((k, _)) = members.get(&self.intern(&key)) else { error!(base_type) };

                            let mut v = self.deep_clone_key(*k);

                            if let Value::Macro(MacroData { self_arg, args, .. }) = &mut v.value {
                                match args.get(0) {
                                    Some(arg) if arg.name().value == self.intern("self") => {
                                        *self_arg = Some(self.get_reg_key(*from))
                                    }
                                    _ => {
                                        return Err(RuntimeError::AssociatedNotAMethod {
                                            area: self.make_area(span, func.code),
                                            def_area: v.area.clone(),
                                            func_name: key,
                                            base_type,
                                            call_stack: self.get_call_stack(),
                                        });
                                    }
                                }
                            } else {
                                return Err(RuntimeError::NotAMethod {
                                    area: self.make_area(span, func.code),
                                    def_area: v.area.clone(),
                                    member_name: key,
                                    member_type: v.value.get_type(),
                                    base_type,
                                    call_stack: self.get_call_stack(),
                                });
                            }

                            self.set_reg(*dest, v);
                        }
                    }
                }
                Opcode::TypeMember { from, dest, member } => {
                    let stored_value = self.get_reg(*from);
                    let value = &stored_value.value;
                    let span = self.get_span(func, ip);

                    match &self.get_reg(*from).value {
                        Value::Module { types, .. } => {
                            let key = match &self.get_reg(*member).value {
                                Value::String(s) => self.intern_vec(s),
                                _ => unreachable!(),
                            };

                            let typ = types
                                .iter()
                                .find(|k| self.types[k.0].value.name == key)
                                .ok_or(RuntimeError::NonexistentTypeMember {
                                    area: self.make_area(span, func.code),
                                    type_name: self.resolve(&key),
                                    call_stack: self.get_call_stack(),
                                })?;

                            if typ.1 {
                                return Err(RuntimeError::PrivateType {
                                    area: self.make_area(span, func.code),
                                    type_name: self.resolve(&key),
                                    call_stack: self.get_call_stack(),
                                });
                            }

                            self.set_reg(
                                *dest,
                                StoredValue {
                                    value: Value::Type(ValueType::Custom(typ.0)),
                                    area: self.make_area(span, func.code),
                                },
                            );
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (value.get_type(), stored_value.area.clone()),
                                area: self.make_area(span, func.code),
                                expected: ValueType::Module,
                                call_stack: self.get_call_stack(),
                            })
                        }
                    }
                }
                Opcode::Associated { from, dest, name } => {
                    let key = self.intern_vec(match &self.get_reg(*name).value {
                        Value::String(s) => s,
                        _ => unreachable!(),
                    });
                    let span = self.get_span(func, ip);

                    let value = self.get_reg(*from);

                    match &value.value {
                        Value::Type(t) => {
                            macro_rules! error {
                                () => {
                                    return Err(RuntimeError::NonexistentAssociatedMember {
                                        area: self.make_area(span, func.code),
                                        member: self.resolve(&key).into(),
                                        base_type: *t,
                                        call_stack: self.get_call_stack(),
                                    })
                                };
                            }
                            match self.impls.get(t) {
                                Some(members) => match members.get(&key) {
                                    Some((k, _)) => {
                                        let v = self.deep_clone_key(*k);

                                        self.set_reg(*dest, v);
                                    }
                                    None => error!(),
                                },
                                None => error!(),
                            }
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (value.value.get_type(), value.area.clone()),
                                area: self.make_area(span, func.code),
                                expected: ValueType::Type,
                                call_stack: self.get_call_stack(),
                            })
                        }
                    }
                }
                Opcode::CreateInstance { base, dict, dest } => {
                    let span = self.get_span(func, ip);

                    let value = self.get_reg(*base);

                    let typ = match &value.value {
                        Value::Type(ValueType::Custom(k)) => *k,
                        Value::Type(t) => {
                            return Err(RuntimeError::CannotInstanceBuiltinType {
                                area: self.make_area(span, func.code),
                                typ: *t,
                                call_stack: self.get_call_stack(),
                            })
                        }
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (value.value.get_type(), value.area.clone()),
                                area: self.make_area(span, func.code),
                                expected: ValueType::Type,
                                call_stack: self.get_call_stack(),
                            })
                        }
                    };

                    let items = match &self.get_reg(*dict).value {
                        Value::Dict(items) => items.clone(),
                        _ => unreachable!(),
                    };

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Instance { typ, items },
                            area: self.make_area(span, func.code),
                        },
                    );
                }
                Opcode::Impl { base, dict } => {
                    let span = self.get_span(func, ip);

                    let value = self.get_reg(*base);

                    let typ = match &value.value {
                        Value::Type(t) => *t,
                        _ => {
                            return Err(RuntimeError::TypeMismatch {
                                v: (value.value.get_type(), value.area.clone()),
                                area: self.make_area(span, func.code),
                                expected: ValueType::Type,
                                call_stack: self.get_call_stack(),
                            })
                        }
                    };

                    let items = match &self.get_reg(*dict).value {
                        Value::Dict(items) => items.clone(),
                        _ => unreachable!(),
                    };

                    for (name, (k, vis)) in &items {
                        let name = self.resolve(name);

                        if let Value::Macro(MacroData { target, .. }) = &mut self.memory[*k].value {
                            if let Some(f) = typ.get_override(&name) {
                                *target = MacroTarget::Builtin(f)
                            }
                        }
                    }

                    self.impls
                        .entry(typ)
                        .and_modify(|d| d.extend(items.clone().into_iter()))
                        .or_insert(items);
                }
                Opcode::Overload { array, op } => {
                    let span = self.get_span(func, ip);

                    let items = match &self.get_reg(*array).value {
                        Value::Array(items) => items.clone(),
                        _ => unreachable!(),
                    };

                    self.overloads
                        .entry(*op)
                        .and_modify(|d| d.extend(items.clone()))
                        .or_insert(items);
                }
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
                    let base = self.get_reg(*base).clone();
                    let call_area = self.get_area(func, ip);
                    match base.value {
                        Value::Macro(data) => {
                            let pos_args;
                            let named_args;

                            match std::mem::take({
                                let k = self.get_reg_key(*args);
                                &mut self.memory[k].value
                            }) {
                                Value::Array(v) => {
                                    match std::mem::take(&mut self.memory[v[0]].value) {
                                        Value::Array(v) => {
                                            pos_args = v;
                                        }
                                        _ => unreachable!(),
                                    }
                                    match std::mem::take(&mut self.memory[v[1]].value) {
                                        Value::Dict(m) => {
                                            named_args =
                                                m.into_iter().map(|(s, (k, _))| (s, k)).collect();
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                                _ => unreachable!(),
                            }

                            self.run_macro(
                                data,
                                pos_args,
                                named_args,
                                func,
                                call_area,
                                Some(*dest),
                                base.area,
                            )?
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
                        value: Value::Macro(MacroData {
                            target: MacroTarget::Macro {
                                func: FuncCoord {
                                    func: *id as usize,
                                    code: func.code,
                                },
                                captured: self.programs[func.code].1.functions[*id as usize]
                                    .capture_regs
                                    .iter()
                                    .map(|(from, _)| self.get_reg_key(*from))
                                    .collect(),
                            },
                            args: vec![],
                            self_arg: None,
                        }),
                        area: self.get_area(func, ip),
                    },
                ),
                Opcode::PushMacroArg { name, dest, is_ref } => {
                    let name = match &self.get_reg(*name).value {
                        Value::String(s) => self.intern_vec(s),
                        _ => unreachable!(),
                    };
                    let span = self.get_span(func, ip);

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro(MacroData { args, .. }) => args.push(MacroArg::Single {
                            name: name.spanned(span),
                            default: None,
                            pattern: None,
                            is_ref: *is_ref,
                        }),
                        _ => unreachable!(),
                    }
                }
                Opcode::PushMacroSpreadArg { name, dest } => {
                    let name = match &self.get_reg(*name).value {
                        Value::String(s) => self.intern_vec(s),
                        _ => unreachable!(),
                    };
                    let span = self.get_span(func, ip);

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro(MacroData { args, .. }) => args.push(MacroArg::Spread {
                            name: name.spanned(span),
                            pattern: None,
                        }),
                        _ => unreachable!(),
                    }
                }
                Opcode::SetMacroArgDefault { src, dest } => {
                    let set = self.deep_clone_reg_insert(*src);
                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro(MacroData { args, .. }) => {
                            *args.last_mut().unwrap().default_mut() = Some(set)
                        }
                        _ => unreachable!(),
                    }
                }
                Opcode::SetMacroArgPattern { id, dest } => {
                    // let span = self.get_span(func, ip);

                    let pat = &self.programs[func.code].1.const_patterns[*id as usize];

                    match &mut self.get_reg_mut(*dest).value {
                        Value::Macro(MacroData { args, .. }) => {
                            *args.last_mut().unwrap().pattern_mut() = Some(pat.clone())
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

                    let current_context = self.contexts.last_mut().yeet_current().unwrap();

                    self.run_function(
                        current_context,
                        CallInfo {
                            func: coord,
                            return_dest: Some(*dest),
                            call_area: None,
                        },
                        Box::new(|_| Ok(())),
                    )?;

                    //self.contexts.merge_down(full_context, original_ip + 1);
                }
                Opcode::LoadArbitraryId { class, dest } => {
                    let id = Id::Arbitrary(self.next_id(*class));
                    let v = match class {
                        IDClass::Group => Value::Group(id),
                        IDClass::Color => Value::Channel(id),
                        IDClass::Block => Value::Block(id),
                        IDClass::Item => Value::Item(id),
                    };

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: v,
                            area: self.get_area(func, ip),
                        },
                    )
                }
                Opcode::PushContextGroup { src } => {
                    let group = match &self.get_reg(*src).value {
                        Value::Group(g) => *g,
                        _ => unreachable!(),
                    };
                    self.contexts.set_group_and_push(group);
                }
                Opcode::PopGroupStack { fn_reg } => {
                    let prev_group = match &self.get_reg(*fn_reg).value {
                        Value::TriggerFunction { prev_context, .. } => *prev_context,
                        _ => unreachable!(),
                    };

                    self.contexts.pop_groups_until(prev_group);
                }
                Opcode::MakeTriggerFunc { src, dest } => {
                    let group = match &self.get_reg(*src).value {
                        Value::Group(g) => *g,
                        _ => unreachable!(),
                    };

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::TriggerFunction {
                                group,
                                prev_context: self.contexts.group(),
                            },
                            area: self.get_area(func, ip),
                        },
                    )
                }
                Opcode::UnwrapOrJump { src, to } => {
                    // let span = self.get_span(func, ip);
                    match self.get_reg(*src).value {
                        Value::Maybe(v) => match v {
                            Some(k) => {
                                let v = self.deep_clone_key(k);
                                self.set_reg(*src, v)
                            }
                            None => {
                                self.contexts.jump_current(*to as usize);
                                continue;
                            }
                        },
                        _ => unreachable!(),
                    }
                }
                Opcode::WrapIterator { src, dest } => {
                    let span = self.get_span(func, ip);
                    let val = self.get_reg_key(*src);
                    let stored_val = &self.memory[val];
                    let iterator = match &stored_val.value {
                        Value::Array(_) => Value::Iterator(IteratorData::Array {
                            array: val,
                            index: 0,
                        }),
                        Value::String(_) => Value::Iterator(IteratorData::String {
                            string: val,
                            index: 0,
                        }),
                        Value::Range(a, b, c) => Value::Iterator(IteratorData::Range {
                            range: (*a, *b, *c),
                            index: 0,
                        }),
                        _ => {
                            return Err(RuntimeError::CannotIterate {
                                v: (stored_val.value.get_type(), stored_val.area.clone()),
                                area: self.make_area(span, func.code),
                                call_stack: self.get_call_stack(),
                            })
                        }
                    };

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: iterator,
                            area: self.get_area(func, ip),
                        },
                    )
                }
                Opcode::IterNext { src, dest } => {
                    let area = self.get_area(func, ip);

                    let val = match &self.get_reg(*src).value {
                        Value::Iterator(data) => data.next(self, area),
                        _ => unreachable!(),
                    }
                    .map(|v| {
                        let k = self.memory.insert(v);
                        self.deep_clone_key_insert(k)
                    });

                    match &mut self.get_reg_mut(*src).value {
                        Value::Iterator(data) => data.increment(),
                        _ => unreachable!(),
                    }

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Maybe(val),
                            area: self.get_area(func, ip),
                        },
                    )
                }
                // Opcode::PatEq { src, dest } => todo!(),
                // Opcode::PatNeq { src, dest } => todo!(),
                // Opcode::PatGt { src, dest } => todo!(),
                // Opcode::PatGte { src, dest } => todo!(),
                // Opcode::PatLt { src, dest } => todo!(),
                // Opcode::PatLte { src, dest } => todo!(),
                Opcode::Throw { err } => {
                    let area = self.get_area(func, ip);
                    let err = self.get_reg(*err);

                    return Err(RuntimeError::ThrownError {
                        area,
                        message: err.value.runtime_display(self),
                        call_stack: self.get_call_stack(),
                    });
                }
                Opcode::TypeOf { src, dest } => {
                    let area = self.get_area(func, ip);

                    self.set_reg(
                        *dest,
                        StoredValue {
                            value: Value::Type(self.get_reg(*src).value.get_type()),
                            area,
                        },
                    )
                }
            }

            {
                let mut current = self.contexts.current_mut();
                current.ip += 1;
            };
        }

        self.contexts.0.pop();
        if return_dest.is_some() {
            let mut current = self.contexts.current_mut();
            current.ip -= 1;
        };

        Ok(())
    }

    // #[inline]
    pub fn run_macro(
        &mut self,
        data: MacroData,
        pos_args: Vec<ValueKey>,
        named_args: AHashMap<Spur, ValueKey>,
        func: FuncCoord,
        call_area: CodeArea,

        dest: Option<Register>,
        base_area: CodeArea,
    ) -> RuntimeResult<()> {
        let mut param_map: AHashMap<Spur, ValueKey> = AHashMap::new();

        if let Some(s) = data.self_arg {
            param_map.insert(self.intern("self"), s);
        }

        for arg in &data.args {
            if let MacroArg::Spread { name, .. } = arg {
                param_map.insert(
                    name.value,
                    self.memory.insert(StoredValue {
                        value: Value::Array(vec![]),
                        area: self.make_area(name.span, func.code),
                    }),
                );
            }
        }

        {
            let mut exp_idx = data.self_arg.is_some() as usize;
            let mut passed_idx = 0;

            while passed_idx < pos_args.len() {
                if exp_idx >= data.args.len() {
                    return Err(RuntimeError::TooManyArguments {
                        call_area,
                        macro_def_area: base_area,
                        macro_arg_amount: data.args.len() - data.self_arg.is_some() as usize,
                        call_arg_amount: pos_args.len(),
                        call_stack: self.get_call_stack(),
                    });
                }

                let param = pos_args[passed_idx];
                let data = &data.args[exp_idx];
                match data {
                    MacroArg::Single { name, .. } => {
                        param_map.insert(name.value, param);
                        exp_idx += 1;
                        passed_idx += 1;
                    }
                    MacroArg::Spread { name, .. } => {
                        match &mut self.memory[param_map[&name.value]].value {
                            Value::Array(v) => v.push(param),
                            _ => unreachable!(),
                        }

                        passed_idx += 1;
                    }
                }
            }
        }

        {
            for (name, param) in named_args {
                if data.args.iter().any(|m| {
                    matches!(
                        m,
                        MacroArg::Single { name: arg_name, .. } if arg_name.value == name
                    )
                }) {
                    param_map.insert(name, param);
                } else {
                    return Err(RuntimeError::InvalidKeywordArgument {
                        call_area,
                        macro_def_area: base_area,
                        arg_name: self.resolve(&name),
                        call_stack: self.get_call_stack(),
                    });
                }
            }
        }

        macro_rules! per_arg {
            ($vm:ident,($i:ident, $v:ident) $b:block) => {
                for ($i, data) in data.args.iter().enumerate() {
                    let $v = match param_map.get(&data.name().value) {
                        Some(k) => {
                            if let MacroArg::Single { is_ref: true, .. } = data {
                                *k
                            } else {
                                $vm.deep_clone_key_insert(*k)
                            }
                        }
                        None => match data.default() {
                            Some(k) => $vm.deep_clone_key_insert(*k),
                            None => {
                                return Err(RuntimeError::ArgumentNotSatisfied {
                                    call_area,
                                    macro_def_area: base_area,
                                    arg_name: $vm.resolve(&data.name().value),
                                    call_stack: $vm.get_call_stack(),
                                })
                            }
                        },
                    };

                    if let Some(pattern) = data.pattern() {
                        if !pattern.value_matches(&self.memory[$v].value, $vm) {
                            todo!()
                        }
                    }

                    $b
                }
            };
        }

        match data.target {
            MacroTarget::Macro { func, captured } => {
                let current_context = self.contexts.last_mut().yeet_current().unwrap();

                let mut regs_keys = vec![];

                per_arg! {
                    self,
                    (i, k) {
                        regs_keys.push((i, k));
                    }
                }

                self.run_function(
                    current_context,
                    CallInfo {
                        func,
                        return_dest: dest,
                        call_area: Some(call_area),
                    },
                    Box::new(move |vm| {
                        // for (i, k) in regs_keys {
                        //     vm.change_reg_key(i as Register, k)
                        // }

                        // for (k, (_, to)) in captured
                        //     .iter()
                        //     .zip(&vm.programs[func.code].1.functions[func.func].capture_regs)
                        // {
                        //     vm.change_reg_key(*to, *k);
                        // }

                        Ok(())
                    }),
                )?;

                Ok(())
            }

            MacroTarget::Builtin(f) => {
                let mut args = vec![];
                per_arg! {
                    self,
                    (_i, k) {
                        args.push(k);
                    }
                }
                let ret = f.0(args, self, call_area.clone())?;
                if let Some(dest) = dest {
                    self.set_reg(
                        dest,
                        StoredValue {
                            value: ret,
                            area: call_area,
                        },
                    );
                }
                Ok(())
            }
        }
    }

    fn find_overload<const N: usize>(
        &mut self,
        op: Operator,
        values: [Register; N],
    ) -> Option<(MacroData, CodeArea)> {
        // if let Some(overloads) = self.overloads.get(&op).cloned() {
        //     'overloads: for overload in overloads.iter().rev() {
        //         let data = match &self.memory[*overload].value {
        //             Value::Macro(data) => {
        //                 data.clone()
        //                 // if values
        //                 //     .into_iter()
        //                 //     .zip(&data.args)
        //                 //     .all(|(v, arg)| arg.pattern().as_ref().unwrap().value_matches(v, self))
        //                 // {
        //                 //     return Some((data.clone(), self.memory[*overload].area.clone()));
        //                 // }
        //             }
        //             _ => unreachable!(),
        //         };
        //         for (v, arg) in values.into_iter().zip(&data.args) {
        //             if !arg
        //                 .pattern()
        //                 .as_ref()
        //                 .unwrap()
        //                 .value_matches(&self.get_reg(v).value.clone(), self)
        //                 .unwrap()
        //             // lol
        //             {
        //                 continue 'overloads;
        //             }
        //         }
        //         return Some((data.clone(), self.memory[*overload].area.clone()));
        //     }
        // }
        None
    }

    fn run_overload<const N: usize>(
        &mut self,
        operator: Operator,
        regs: [u8; N],
        func: FuncCoord,
        span: CodeSpan,
        dest: Option<u8>,
    ) -> Result<bool, RuntimeError> {
        if let Some((data, base_area)) = // .map(|a| &self.get_reg(a).value)
            self.find_overload(operator, regs)
        {
            self.run_macro(
                data,
                regs.map(|a| self.get_reg_key(a)).to_vec(),
                AHashMap::new(),
                func,
                self.make_area(span, func.code),
                dest,
                base_area,
            )?;
            return Ok(true);
        }
        Ok(false)
    }

    #[inline]
    pub fn bin_op<F>(
        &mut self,
        op: F,
        func: FuncCoord,
        ip: usize,
        left: &u8,
        right: &u8,
        dest: &u8,
        operator: BinOp,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&StoredValue, &StoredValue, CodeSpan, &Vm, BytecodeKey) -> RuntimeResult<Value>,
    {
        let span = self.get_span(func, ip);

        if self.run_overload(Operator::Bin(operator), [*left, *right], func, span, Some(*dest))? {
            return Ok(());
        };

        let value = op(self.get_reg(*left), self.get_reg(*right), span, self, func.code)?;

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
    fn assign_op<F>(
        &mut self,
        op: F,
        func: FuncCoord,
        ip: usize,
        left: &u8,
        right: &u8,
        operator: AssignOp,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&StoredValue, &StoredValue, CodeSpan, &Vm, BytecodeKey) -> RuntimeResult<Value>,
    {
        let span = self.get_span(func, ip);

        if self.run_overload(Operator::Assign(operator), [*left, *right], func, span, None)? {
            return Ok(());
        };

        let value = op(self.get_reg(*left), self.get_reg(*right), span, self, func.code)?;
        let k = self.get_reg_key(*left);
        self.memory[k].value = value;
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
        operator: UnaryOp,
    ) -> Result<(), RuntimeError>
    where
        F: Fn(&StoredValue, CodeSpan, &Vm, BytecodeKey) -> RuntimeResult<Value>,
    {
        let span = self.get_span(func, ip);

        if self.run_overload(Operator::Unary(operator), [*value], func, span, Some(*dest))? {
            return Ok(());
        };

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

    pub fn convert_type(
        &mut self,
        v: &StoredValue,
        b: ValueType,
        span: CodeSpan, // ✍️
        code: BytecodeKey,
    ) -> RuntimeResult<Value> {
        if v.value.get_type() == b {
            return Ok(v.value.clone());
        }

        Ok(match (&v.value, b) {
            (Value::Int(i), ValueType::Group) => Value::Group(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Channel) => Value::Channel(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Block) => Value::Block(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Item) => Value::Item(Id::Specific(*i as u16)),

            (Value::Int(i), ValueType::Float) => Value::Float(*i as f64),
            (Value::Float(i), ValueType::Int) => Value::Int(*i as i64),

            (v, ValueType::String) => Value::String(v.runtime_display(self).chars().collect()),
            (v, ValueType::Type) => Value::Type(v.get_type()),
            (_, ValueType::Bool) => Value::Bool(value_ops::to_bool(v, span, self, code)?),
            // (_, ValueType::Pattern) => Value::Pattern(value_ops::to_pattern(v, span, self, code)?),
            (Value::Bool(b), ValueType::Int) => Value::Int(*b as i64),
            (Value::Bool(b), ValueType::Float) => Value::Float(*b as i64 as f64),

            (Value::Array(i), ValueType::Dict) => todo!(),
            (Value::Range(a, b, c), ValueType::Array) => Value::Array(todo!()),

            (Value::TriggerFunction { group, .. }, ValueType::Group) => Value::Group(*group),

            (Value::String(s), ValueType::Float) => {
                Value::Float(s.iter().collect::<String>().parse().unwrap())
            }

            (Value::String(s), ValueType::Array) => Value::Array(
                s.iter()
                    .map(|c| {
                        self.memory.insert(StoredValue {
                            value: Value::String(vec![*c]),
                            area: v.area.clone(),
                        })
                    })
                    .collect(),
            ),
            // oop
            _ => todo!("error"),
        })
    }
}

use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::mem::{self};
use std::rc::Rc;

use ahash::AHashMap;
use base64::Engine;
use colored::Colorize;
use derive_more::{Deref, DerefMut};
use itertools::{Either, Itertools};

use super::builtins::RustFnInstr;
use super::context::{CallInfo, CloneMap, Context, ContextStack, FullContext, StackItem};
use super::value::{MacroTarget, StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::{Bytecode, CallExpr, Constant, Function, OptRegister, Register};
use crate::compiling::opcodes::{CallExprID, ConstID, Opcode, RuntimeStringFlag};
use crate::gd::gd_object::{make_spawn_trigger, TriggerObject, TriggerOrder};
use crate::gd::ids::{IDClass, Id};
use crate::gd::object_keys::OBJECT_KEYS;
use crate::interpreting::context::{ReturnDest, TryCatch};
use crate::interpreting::error::RuntimeError;
use crate::interpreting::value::MacroData;
use crate::parsing::ast::{MacroArg, Vis, VisSource, VisTrait};
use crate::sources::{
    BytecodeMap, CodeArea, CodeSpan, Spannable, Spanned, SpwnSource, TypeDefMap, ZEROSPAN,
};
use crate::util::{ImmutCloneVec, ImmutStr};

const RECURSION_LIMIT: usize = 256;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub trait DeepClone<I> {
    fn deep_clone_map(&self, input: I, map: &mut Option<&mut CloneMap>) -> StoredValue;
    fn deep_clone(&self, input: I) -> StoredValue {
        self.deep_clone_map(input, &mut None)
    }
    fn deep_clone_ref(&self, input: I) -> ValueRef {
        let v: StoredValue = self.deep_clone(input);
        ValueRef::new(v)
    }
    fn deep_clone_ref_map(&self, input: I, map: &mut Option<&mut CloneMap>) -> ValueRef {
        let v: StoredValue = self.deep_clone_map(input, map);
        ValueRef::new(v)
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct ValueRef(Rc<RefCell<StoredValue>>);

impl PartialEq for ValueRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ValueRef {}

impl ValueRef {
    pub fn new(v: StoredValue) -> Self {
        Self(Rc::new(RefCell::new(v)))
    }

    pub fn deep_clone_checked(&self, vm: &Vm, map: &mut Option<&mut CloneMap>) -> Self {
        if let Some(m) = map {
            if let Some(v) = m.get(self) {
                return v.clone();
            }
            let new = vm.deep_clone_ref_map(self, map);
            // goofy thing to avoid borrow checker
            if let Some(m) = map {
                m.insert(self, new.clone());
            }
            return new;
        }
        vm.deep_clone_ref_map(self, map)
    }
}

#[derive(Debug)]
pub struct Program {
    pub src: Rc<SpwnSource>,
    pub bytecode: Rc<Bytecode>,
}

impl Program {
    pub fn get_constant(&self, id: ConstID) -> &Constant {
        &self.bytecode.constants[*id as usize]
    }

    pub fn get_call_expr(&self, id: CallExprID) -> &CallExpr<OptRegister, OptRegister, ImmutStr> {
        &self.bytecode.call_exprs[*id as usize]
    }

    pub fn get_function(&self, id: usize) -> &Function {
        &self.bytecode.functions[id]
    }
}

#[derive(Debug, Clone)]
pub struct FuncCoord {
    pub program: Rc<Program>,
    pub func: usize,
}

impl PartialEq for FuncCoord {
    fn eq(&self, other: &Self) -> bool {
        self.func == other.func && Rc::ptr_eq(&self.program, &other.program)
    }
}
impl Eq for FuncCoord {}

impl Hash for FuncCoord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.program.as_ref() as *const _ as usize).hash(state);
        self.func.hash(state);
    }
}

impl FuncCoord {
    pub fn get_func(&self) -> &Function {
        self.program.get_function(self.func)
    }
}

pub struct Vm {
    // readonly
    pub bytecode_map: BytecodeMap,
    pub type_def_map: TypeDefMap,
    is_doc_gen: bool,

    //penis
    pub context_stack: ContextStack,

    pub triggers: Vec<TriggerObject>,
    pub trigger_order_count: TriggerOrder,

    pub id_counters: [u16; 4],

    pub impls: AHashMap<ValueType, AHashMap<ImmutCloneVec<char>, VisSource<ValueRef>>>,
}

impl Vm {
    pub fn new(is_doc_gen: bool, type_def_map: TypeDefMap, bytecode_map: BytecodeMap) -> Self {
        Self {
            context_stack: ContextStack(vec![]),
            is_doc_gen,
            triggers: vec![],
            trigger_order_count: TriggerOrder::new(),
            type_def_map,
            bytecode_map,
            id_counters: Default::default(),
            impls: AHashMap::new(),
        }
    }

    pub fn make_area(&self, span: CodeSpan, program: &Rc<Program>) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&program.src),
        }
    }

    pub fn set_reg(&mut self, reg: OptRegister, v: StoredValue) {
        let mut binding = self.context_stack.current_mut();
        let mut g = binding.stack.last_mut().unwrap().registers[reg.0 as usize].borrow_mut();
        *g = v;
    }

    pub fn borrow_reg<F, R>(&self, reg: OptRegister, f: F) -> RuntimeResult<R>
    where
        F: FnOnce(Ref<'_, StoredValue>) -> RuntimeResult<R>,
    {
        f(self.context_stack.current().stack.last().unwrap().registers[*reg as usize].borrow())
    }

    pub fn borrow_reg_mut<F, R>(&self, reg: OptRegister, f: F) -> RuntimeResult<R>
    where
        F: FnOnce(RefMut<'_, StoredValue>) -> RuntimeResult<R>,
    {
        f(
            self.context_stack.current().stack.last().unwrap().registers[*reg as usize]
                .borrow_mut(),
        )
    }

    pub fn get_reg_ref(&self, reg: OptRegister) -> &ValueRef {
        &self.context_stack.current().stack.last().unwrap().registers[*reg as usize]
    }

    pub fn change_reg_ref(&mut self, reg: OptRegister, k: ValueRef) {
        self.context_stack
            .current_mut()
            .stack
            .last_mut()
            .unwrap()
            .registers[*reg as usize] = k
    }

    pub fn next_id(&mut self, c: IDClass) -> u16 {
        self.id_counters[c as usize] += 1;
        self.id_counters[c as usize]
    }
}

impl DeepClone<&StoredValue> for Vm {
    fn deep_clone_map(&self, input: &StoredValue, map: &mut Option<&mut CloneMap>) -> StoredValue {
        let area = input.area.clone();

        let mut deep_clone_dict_items = |v: &AHashMap<ImmutCloneVec<char>, VisSource<ValueRef>>| {
            v.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.clone().map(|v| v.deep_clone_checked(self, map)),
                    )
                })
                .collect()
        };

        let value = match &input.value {
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| v.deep_clone_checked(self, map))
                    .collect(),
            ),
            Value::Dict(map) => Value::Dict(deep_clone_dict_items(map)),
            Value::Maybe(v) => Value::Maybe(v.as_ref().map(|v| v.deep_clone_checked(self, map))),
            Value::Instance { typ, items } => Value::Instance {
                typ: *typ,
                items: deep_clone_dict_items(items),
            },
            Value::Module { exports, types } => Value::Module {
                exports: exports
                    .iter()
                    .map(|(k, v)| (Rc::clone(k), v.deep_clone_checked(self, map)))
                    .collect(),
                types: types.clone(),
            },
            v @ (Value::Macro(data) | Value::Iterator(data)) => {
                let mut new_data = data.clone();
                for i in new_data.defaults.iter_mut() {
                    if let Some(r) = i {
                        *r = r.deep_clone_checked(self, map)
                    }
                }
                if let Some(r) = &mut new_data.self_arg {
                    *r = r.deep_clone_checked(self, map)
                }

                match &mut new_data.target {
                    MacroTarget::Spwn { captured, .. } => {
                        for r in captured.iter_mut() {
                            *r = r.deep_clone_checked(self, map)
                        }
                    },
                    MacroTarget::FullyRust { .. } => (),
                }
                if matches!(v, Value::Macro(_)) {
                    Value::Macro(new_data)
                } else {
                    Value::Iterator(new_data)
                }
            },
            // todo: iterator, object
            v => v.clone(),
        };

        value.into_stored(area)
    }
}

impl DeepClone<&ValueRef> for Vm {
    fn deep_clone_map(&self, input: &ValueRef, map: &mut Option<&mut CloneMap>) -> StoredValue {
        let v = input.borrow();

        self.deep_clone_map(&*v, map)
    }
}

impl DeepClone<OptRegister> for Vm {
    fn deep_clone_map(&self, input: OptRegister, map: &mut Option<&mut CloneMap>) -> StoredValue {
        let v = &self.context_stack.current().stack.last().unwrap().registers[*input as usize];
        self.deep_clone_map(v, map)
    }
}

pub struct SplitFnRet;

#[derive(Debug, Clone, Copy)]
pub enum LoopFlow {
    ContinueLoop,
    Normal,
}

impl Vm {
    pub fn get_call_stack(&self) -> Vec<CallInfo> {
        self.context_stack
            .0
            .iter()
            .take(5)
            .map(|f| &f.call_info)
            .cloned()
            .collect()
    }

    pub fn run_rust_instrs(
        &mut self,
        // mut context: Context,
        call_info: CallInfo,
        instrs: &[RustFnInstr<'_>],
    ) -> RuntimeResult<SplitFnRet> {
        let mut context = self.context_stack.last_mut().yeet_current().unwrap();

        let original_ip = context.ip;
        context.ip = 0;

        self.context_stack
            .push(FullContext::new(context, call_info));

        let mut finished_contexts = vec![];

        let mut finish_context = |mut c: Context| {
            c.ip = original_ip + 1;
            // println!("sigla: {}", c.ip);
            finished_contexts.push(c)
        };

        while self.context_stack.last().valid() {
            let ip = self.context_stack.last().current_ip();

            if ip >= instrs.len() {
                if !self.context_stack.last().have_returned {
                    finish_context(self.context_stack.last_mut().yeet_current().unwrap());
                } else {
                    self.context_stack.last_mut().yeet_current();
                }
                continue;
            }

            match instrs[ip](self)? {
                LoopFlow::ContinueLoop => continue,
                LoopFlow::Normal => {},
            };

            {
                let mut current = self.context_stack.current_mut();
                current.ip += 1;
            };
            // self.try_merge_contexts();
        }
        self.context_stack.pop().unwrap();
        self.context_stack
            .last_mut()
            .contexts
            .extend(finished_contexts.into_iter());

        Ok(SplitFnRet)
    }

    /// <img src="https://cdna.artstation.com/p/assets/images/images/056/833/046/original/lara-hughes-blahaj-spin-compressed.gif?1670214805" width=60 height=60>
    /// <img src="https://cdna.artstation.com/p/assets/images/images/056/833/046/original/lara-hughes-blahaj-spin-compressed.gif?1670214805" width=60 height=60>
    /// <img src="https://cdna.artstation.com/p/assets/images/images/056/833/046/original/lara-hughes-blahaj-spin-compressed.gif?1670214805" width=60 height=60>
    pub fn run_function(
        &mut self,
        mut context: Context,
        call_info: CallInfo,
        cb: Box<dyn FnOnce(&mut Vm) -> RuntimeResult<()>>,
    ) -> RuntimeResult<SplitFnRet> {
        let CallInfo {
            func: FuncCoord { program, func },
            return_dest,
            is_builtin,
            ..
        } = call_info.clone();

        let original_ip = context.ip;
        context.ip = 0;

        {
            let used = program.get_function(func).regs_used;
            let mut regs = Vec::with_capacity(used as usize);

            for _ in 0..program.get_function(func).regs_used {
                let v = ValueRef::new(StoredValue {
                    value: Value::Empty,
                    area: CodeArea {
                        src: Rc::clone(&program.src),
                        span: CodeSpan::internal(),
                    },
                });
                regs.push(v)
            }

            context.stack.push(StackItem {
                registers: regs.into_boxed_slice(),
                store_extra: None,
            });
        }

        self.context_stack
            .push(FullContext::new(context, call_info));
        cb(self)?;

        let opcodes = &program.get_function(func).opcodes;

        let mut finished_contexts = vec![];

        let mut finish_context = |mut c: Context| {
            c.ip = original_ip + 1;
            // println!("sigla: {}", c.ip);
            finished_contexts.push(c)
        };

        while self.context_stack.last().valid() {
            {
                if let Some(r) = self
                    .context_stack
                    .current()
                    .stack
                    .last()
                    .unwrap()
                    .store_extra
                {
                    let v = self.context_stack.current_mut().extra_stack.pop().unwrap();
                    // println!("jabba {:?}", v.value);
                    self.set_reg(r, v);
                    self.context_stack
                        .current_mut()
                        .stack
                        .last_mut()
                        .unwrap()
                        .store_extra = None;
                }
            }

            let ip = self.context_stack.last().current_ip();

            if ip >= opcodes.len() {
                if !self.context_stack.last().have_returned {
                    // let return_dest = self.context_stack.last().call_info.return_dest;
                    {
                        let mut current = self.context_stack.current_mut();

                        current.stack.pop();
                    }

                    finish_context(self.context_stack.last_mut().yeet_current().unwrap());
                } else {
                    self.context_stack.last_mut().yeet_current();
                }
                continue;
            }

            let Spanned {
                value: opcode,
                span: opcode_span,
            } = opcodes[ip];

            if self.context_stack.len() > RECURSION_LIMIT {
                return Err(RuntimeError::RecursionLimit {
                    area: self.make_area(opcode_span, &program),
                    call_stack: self.get_call_stack(),
                });
            }

            macro_rules! load_val {
                ($val:expr, $to:expr) => {
                    self.set_reg($to, $val.into_stored(self.make_area(opcode_span, &program)))
                };
            }

            // loop {
            //     print!(
            //         "{} {} ",
            //         format!(
            //             "CTX {:?}, F{}, next {}",
            //             self.context_stack.current().unique_id,
            //             func,
            //             ip
            //         )
            //         .green()
            //         .dimmed(),
            //         "Debug:".bright_green().bold()
            //     );
            //     // self.context_stack.current();
            //     io::stdout().flush().unwrap();

            //     let mut user_input = String::new();
            //     io::stdin().read_line(&mut user_input).unwrap();
            //     let s = user_input.trim();

            //     if s.is_empty() {
            //         // println!("{}", "END DEBUG".bright_red().bold());
            //         break;
            //     } else {
            //         fn is_debug_type<'a>(s: &'a str, prefix: &'a str) -> Option<&'a str> {
            //             if s.len() > prefix.len() && &s[..prefix.len()] == prefix {
            //                 Some(&s[prefix.len()..])
            //             } else {
            //                 None
            //             }
            //         }

            //         if let Some(s) = is_debug_type(s, "0x") {
            //             let ptr = unsafe {
            //                 &*(usize::from_str_radix(s, 16).unwrap() as *mut StoredValue)
            //             };

            //             println!("{} {:?}", "value:".dimmed(), ptr.value);
            //         } else if let Some(s) = is_debug_type(s, "R") {
            //             let reg = Register(s.parse().unwrap());

            //             let value_ref = self.get_reg_ref(reg);

            //             println!("{}", self.runtime_display(value_ref, true),)
            //         } else {
            //             println!("{}", "stupid bitch".dimmed())
            //         }
            //     }
            // }

            // MaybeUninit
            let mut run_opcode = |opcode| -> RuntimeResult<LoopFlow> {
                match opcode {
                    Opcode::LoadConst { id, to } => {
                        let value = Value::from_const(self, program.get_constant(id));
                        self.set_reg(to, value.into_stored(self.make_area(opcode_span, &program)));
                    },
                    Opcode::CopyDeep { from, to } => self.set_reg(to, self.deep_clone(from)),
                    Opcode::CopyShallow { from, to } => {
                        let new = self.get_reg_ref(from).borrow().clone();
                        self.set_reg(to, new)
                    },
                    Opcode::CopyMem { from, to } => {
                        let v = self.get_reg_ref(from).clone();
                        self.change_reg_ref(to, v);
                    },

                    Opcode::Plus { a, b, to } => {
                        self.bin_op(value_ops::plus, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Minus { a, b, to } => {
                        self.bin_op(value_ops::minus, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Mult { a, b, to } => {
                        self.bin_op(value_ops::mult, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Div { a, b, to } => {
                        self.bin_op(value_ops::div, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Mod { a, b, to } => {
                        self.bin_op(value_ops::modulo, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Pow { a, b, to } => {
                        self.bin_op(value_ops::pow, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Eq { a, b, to } => {
                        self.bin_op(value_ops::eq_op, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Neq { a, b, to } => {
                        self.bin_op(value_ops::neq_op, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Gt { a, b, to } => {
                        self.bin_op(value_ops::gt, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Gte { a, b, to } => {
                        self.bin_op(value_ops::gte, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Lt { a, b, to } => {
                        self.bin_op(value_ops::lt, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Lte { a, b, to } => {
                        self.bin_op(value_ops::lte, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::BinOr { a, b, to } => {
                        self.bin_op(value_ops::bin_or, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::BinAnd { a, b, to } => {
                        self.bin_op(value_ops::bin_and, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::Range { a, b, to } => {
                        self.bin_op(value_ops::range, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::In { a, b, to } => {
                        self.bin_op(value_ops::in_op, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::ShiftLeft { a, b, to } => {
                        self.bin_op(value_ops::shift_left, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::ShiftRight { a, b, to } => {
                        self.bin_op(value_ops::shift_right, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::As { a, b, to } => {
                        self.bin_op(value_ops::as_op, &program, a, b, to, opcode_span)?;
                    },
                    Opcode::PlusEq { a, b } => {
                        // println!(
                        //     "albebe {:?} {:?} {:?} {:?}",
                        //     self.get_reg_ref(a).borrow().value,
                        //     self.get_reg_ref(a).as_ptr(),
                        //     self.get_reg_ref(b).borrow().value,
                        //     self.get_reg_ref(b).as_ptr()
                        // );
                        // println!(
                        //     "tzuma {:?} {:?}",
                        //     self.get_reg_ref(Register(6)).as_ptr(),
                        //     self.get_reg_ref(Register(9)).as_ptr(),
                        // );
                        self.assign_op(value_ops::plus, &program, a, b, opcode_span)?;
                    },
                    Opcode::MinusEq { a, b } => {
                        self.assign_op(value_ops::minus, &program, a, b, opcode_span)?;
                    },
                    Opcode::MultEq { a, b } => {
                        self.assign_op(value_ops::plus, &program, a, b, opcode_span)?;
                    },
                    Opcode::DivEq { a, b } => {
                        self.assign_op(value_ops::mult, &program, a, b, opcode_span)?;
                    },
                    Opcode::PowEq { a, b } => {
                        self.assign_op(value_ops::pow, &program, a, b, opcode_span)?;
                    },
                    Opcode::ModEq { a, b } => {
                        self.assign_op(value_ops::modulo, &program, a, b, opcode_span)?;
                    },
                    Opcode::BinAndEq { a, b } => {
                        self.assign_op(value_ops::bin_and, &program, a, b, opcode_span)?;
                    },
                    Opcode::BinOrEq { a, b } => {
                        self.assign_op(value_ops::bin_or, &program, a, b, opcode_span)?;
                    },
                    Opcode::ShiftLeftEq { a, b } => {
                        self.assign_op(value_ops::shift_left, &program, a, b, opcode_span)?;
                    },
                    Opcode::ShiftRightEq { a, b } => {
                        self.assign_op(value_ops::shift_right, &program, a, b, opcode_span)?;
                    },
                    Opcode::Not { v, to } => {
                        self.unary_op(value_ops::unary_not, &program, v, to, opcode_span)?;
                    },
                    Opcode::Negate { v, to } => {
                        self.unary_op(value_ops::unary_negate, &program, v, to, opcode_span)?;
                    },
                    Opcode::Jump { to } => {
                        self.context_stack.last_mut().jump_current(*to as usize);
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::JumpIfFalse { check, to } => {
                        let b = self.borrow_reg(check, |check| {
                            value_ops::to_bool(&check, opcode_span, self, &program)
                        })?;

                        if !b {
                            self.context_stack.last_mut().jump_current(*to as usize);
                            return Ok(LoopFlow::ContinueLoop);
                        }
                    },
                    Opcode::JumpIfTrue { check, to } => {
                        let b = self.borrow_reg(check, |check| {
                            value_ops::to_bool(&check, opcode_span, self, &program)
                        })?;

                        if b {
                            self.context_stack.last_mut().jump_current(*to as usize);
                            return Ok(LoopFlow::ContinueLoop);
                        }
                    },
                    Opcode::UnwrapOrJump { check, to } => {
                        match &self.get_reg_ref(check).clone().borrow().value {
                            Value::Maybe(v) => match v {
                                Some(k) => {
                                    let val = self.deep_clone(k);

                                    self.set_reg(check, val);
                                },
                                None => {
                                    self.context_stack.last_mut().jump_current(*to as usize);
                                    return Ok(LoopFlow::ContinueLoop);
                                },
                            },
                            _ => unreachable!(),
                        };
                    },
                    Opcode::AllocArray { dest, len } => self.set_reg(
                        dest,
                        StoredValue {
                            value: Value::Array(Vec::with_capacity(len as usize)),
                            area: self.make_area(opcode_span, &program),
                        },
                    ),
                    Opcode::PushArrayElem { elem, dest } => {
                        let push = self.deep_clone_ref(elem);
                        match &mut self.get_reg_ref(dest).borrow_mut().value {
                            Value::Array(v) => v.push(push),
                            _ => unreachable!(),
                        }
                    },
                    Opcode::AllocDict { dest, capacity } => self.set_reg(
                        dest,
                        StoredValue {
                            value: Value::Dict(AHashMap::with_capacity(capacity as usize)),
                            area: self.make_area(opcode_span, &program),
                        },
                    ),
                    Opcode::InsertDictElem { elem, dest, key } => {
                        let push = self.deep_clone_ref(elem);

                        let key = self.borrow_reg(key, |key| match &key.value {
                            Value::String(s) => Ok(Rc::clone(s)),
                            _ => unreachable!(),
                        })?;

                        match &mut self.get_reg_ref(dest).borrow_mut().value {
                            Value::Dict(v) => v.insert(key, VisSource::Public(push)),
                            _ => unreachable!(),
                        };
                    },
                    Opcode::InsertPrivDictElem { elem, dest, key } => {
                        let push = self.deep_clone_ref(elem);

                        let key = self.borrow_reg(key, |key| match &key.value {
                            Value::String(s) => Ok(Rc::clone(s)),
                            _ => unreachable!(),
                        })?;

                        match &mut self.get_reg_ref(dest).borrow_mut().value {
                            Value::Dict(v) => {
                                v.insert(key, VisSource::Private(push, Rc::clone(&program.src)))
                            },
                            _ => unreachable!(),
                        };
                    },
                    Opcode::WrapIterator { src, dest } => todo!(),
                    Opcode::IterNext { src, dest } => todo!(),
                    Opcode::Return { src, module_ret } => {
                        let mut ret_val = self.deep_clone(src);

                        if module_ret {
                            match ret_val.value {
                                Value::Dict(d) => {
                                    ret_val.value = Value::Module {
                                        exports: d
                                            .into_iter()
                                            .map(|(s, v)| (s, v.value().clone()))
                                            .collect(),
                                        types: program
                                            .bytecode
                                            .custom_types
                                            .iter()
                                            .map(|(id, v)| match v {
                                                Vis::Public(_) => Vis::Public(*id),
                                                Vis::Private(_) => Vis::Private(*id),
                                            })
                                            .collect(),
                                    };
                                    // println!("{:?}", ret_val.value);
                                },
                                _ => unreachable!(),
                            }
                        }

                        self.context_stack.last_mut().have_returned = true;

                        let return_dest = self.context_stack.last().call_info.return_dest;
                        {
                            let mut current = self.context_stack.current_mut();
                            current.stack.pop();

                            match return_dest {
                                None => (),
                                Some(ReturnDest::Reg(r)) => {
                                    current.stack.last_mut().unwrap().registers[*r as usize] =
                                        ValueRef::new(ret_val)
                                },
                                Some(ReturnDest::Extra) => current.extra_stack.push(ret_val),
                            }

                            // if let Some(r) = return_dest {
                            //     current.stack.last_mut().unwrap().registers[*r as usize] =
                            //         ValueRef::new(ret_val)
                            // }
                        }

                        finish_context(self.context_stack.last_mut().yeet_current().unwrap());

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Import { id, dest } => {
                        let import = &program.bytecode.import_paths[*id as usize];

                        let coord = FuncCoord {
                            func: 0,
                            program: Rc::new(Program {
                                src: Rc::new(import.clone()),
                                bytecode: self.bytecode_map[import].clone(),
                            }),
                        };

                        let current_context = self.context_stack.last_mut().yeet_current().unwrap();
                        // also i need to do mergig
                        self.run_function(
                            current_context,
                            CallInfo {
                                func: coord,
                                return_dest: Some(ReturnDest::Reg(dest)),
                                call_area: None,
                                is_builtin: None,
                            },
                            Box::new(|_| Ok(())),
                            // ContextSplitMode::Disallow,
                        )?;
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::EnterArrowStatement { skip } => {
                        // println!("futa");
                        self.split_current_context();
                        // println!("nari");
                        self.context_stack.last_mut().jump_current(*skip as usize);
                    },
                    Opcode::YeetContext => {
                        self.context_stack.last_mut().yeet_current();
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Dbg { reg, show_ptr } => {
                        let value_ref = self.get_reg_ref(reg);

                        println!(
                            "{} {} {}, {:?}, {}",
                            self.runtime_display(value_ref, show_ptr),
                            "::".dimmed(),
                            self.context_stack.last().current_group().fmt("g").green(),
                            value_ref.as_ptr(),
                            self.context_stack.last().current().stack.len(),
                        )
                    },
                    Opcode::Throw { reg } => {
                        let value = self.deep_clone(reg);

                        return Err(RuntimeError::ThrownError {
                            area: self.make_area(opcode_span, &program),
                            value,
                            call_stack: self.get_call_stack(),
                        });
                    },
                    Opcode::Index { base, dest, index }
                    | Opcode::IndexMem { base, dest, index } => {
                        let is_mem = matches!(opcode, Opcode::IndexMem { .. });

                        let base_ref = self.get_reg_ref(base).borrow();
                        let index_ref = self.get_reg_ref(index).borrow();

                        let index_wrap = |idx: i64, len: usize, typ: ValueType| {
                            let index_calc = if idx >= 0 { idx } else { len as i64 + idx };

                            if index_calc < 0 || index_calc >= len as i64 {
                                return Err(RuntimeError::IndexOutOfBounds {
                                    len,
                                    index: idx,
                                    area: self.make_area(opcode_span, &program),
                                    typ,
                                    call_stack: self.get_call_stack(),
                                });
                            }

                            Ok(index_calc as usize)
                        };

                        macro_rules! drop {
                            () => {
                                mem::drop(base_ref);
                                mem::drop(index_ref);
                            };
                        }

                        match (&base_ref.value, &index_ref.value) {
                            (Value::Array(v), Value::Int(index)) => {
                                let v = &v[index_wrap(*index, v.len(), ValueType::Array)?];

                                if !is_mem {
                                    let v = self.deep_clone(v);
                                    drop!();
                                    self.set_reg(dest, v);
                                } else {
                                    let v = v.clone();
                                    drop!();
                                    self.change_reg_ref(dest, v);
                                }
                            },
                            (Value::String(s), Value::Int(index)) => {
                                let idx = index_wrap(*index, s.len(), ValueType::String)?;
                                let c = s[idx];

                                drop!();

                                let v = Value::String(Rc::new([c]))
                                    .into_stored(self.make_area(opcode_span, &program));
                                self.set_reg(dest, v);
                            },
                            (Value::Dict(v), Value::String(s)) => match v.get(s) {
                                Some(v) => {
                                    let v = v.value();

                                    if !is_mem {
                                        let v = self.deep_clone(v);
                                        drop!();
                                        self.set_reg(dest, v);
                                    } else {
                                        let v = v.clone();
                                        drop!();
                                        self.change_reg_ref(dest, v);
                                    }
                                },
                                None => {
                                    return Err(RuntimeError::NonexistentMember {
                                        area: self.make_area(opcode_span, &program),
                                        member: s.iter().collect(),
                                        base_type: base_ref.value.get_type(),
                                        call_stack: self.get_call_stack(),
                                    })
                                },
                            },
                            _ => {
                                return Err(RuntimeError::InvalidIndex {
                                    base: (base_ref.value.get_type(), base_ref.area.clone()),
                                    index: (index_ref.value.get_type(), index_ref.area.clone()),
                                    area: self.make_area(opcode_span, &program),
                                    call_stack: self.get_call_stack(),
                                });
                            },
                        };
                    },
                    Opcode::Member { from, dest, member }
                    | Opcode::MemberMem { from, dest, member } => {
                        let is_mem = matches!(opcode, Opcode::MemberMem { .. });

                        let key = match &self.get_reg_ref(member).borrow().value {
                            Value::String(s) => Rc::clone(s),
                            _ => unreachable!(),
                        };

                        let value = self.get_reg_ref(from).borrow();

                        let special = match (&value.value, &key[..]) {
                            (Value::String(s), ['l', 'e', 'n', 'g', 't', 'h']) => {
                                Some(Value::Int(s.len() as i64))
                            },

                            (Value::Range { start, .. }, ['s', 't', 'a', 'r', 't']) => {
                                Some(Value::Int(*start))
                            },
                            (Value::Range { end, .. }, ['e', 'n', 'd']) => Some(Value::Int(*end)),
                            (Value::Range { step, .. }, ['s', 't', 'e', 'p']) => {
                                Some(Value::Int(*step as i64))
                            },

                            (Value::Array(v), ['l', 'e', 'n', 'g', 't', 'h']) => {
                                Some(Value::Int(v.len() as i64))
                            },
                            (Value::Dict(v), ['l', 'e', 'n', 'g', 't', 'h']) => {
                                Some(Value::Int(v.len() as i64))
                            },

                            (Value::Builtins, ['o', 'b', 'j', '_', 'p', 'r', 'o', 'p', 's']) => {
                                Some(Value::Dict({
                                    let mut map = AHashMap::new();
                                    for (n, k) in OBJECT_KEYS.iter() {
                                        map.insert(
                                            n.chars().collect_vec().into(),
                                            VisSource::Public(ValueRef::new(
                                                Value::ObjectKey(*k).into_stored(
                                                    self.make_area(opcode_span, &program),
                                                ),
                                            )),
                                        );
                                    }
                                    map
                                }))
                            },

                            _ => None,
                        };

                        macro_rules! error {
                            ($type:ident) => {
                                return Err(RuntimeError::NonexistentMember {
                                    area: self.make_area(opcode_span, &program),
                                    member: key.iter().collect(),
                                    base_type: $type,
                                    call_stack: self.get_call_stack(),
                                })
                            };
                        }

                        if let Some(v) = special {
                            mem::drop(value);

                            self.set_reg(
                                dest,
                                v.into_stored(self.make_area(opcode_span, &program)),
                            );
                        } else {
                            let base_type = value.value.get_type();

                            'out: {
                                match &value.value {
                                    Value::Dict(v) => {
                                        if let Some(v) = v.get(&key) {
                                            let v = v.value();

                                            if !is_mem {
                                                let v = self.deep_clone(v);

                                                mem::drop(value);

                                                self.set_reg(dest, v);
                                            } else {
                                                let v = v.clone();

                                                mem::drop(value);

                                                self.change_reg_ref(dest, v);
                                            }
                                            break 'out;
                                        }
                                    },
                                    Value::Instance { items, .. } => {
                                        if let Some(v) = items.get(&key) {
                                            if let VisSource::Private(v, src) = v {
                                                if src != &program.src {
                                                    return Err(
                                                        RuntimeError::PrivateMemberAccess {
                                                            area: self
                                                                .make_area(opcode_span, &program),
                                                            member: key.iter().collect(),
                                                            call_stack: self.get_call_stack(),
                                                        },
                                                    );
                                                }
                                            }

                                            let v = v.value();

                                            if !is_mem {
                                                let v = self.deep_clone(v);

                                                mem::drop(value);

                                                self.set_reg(dest, v);
                                            } else {
                                                let v = v.clone();

                                                mem::drop(value);

                                                self.change_reg_ref(dest, v);
                                            }
                                            break 'out;
                                        }
                                    },
                                    Value::Module { exports, .. } => {
                                        if let Some(v) = exports.get(&key) {
                                            if !is_mem {
                                                let v = self.deep_clone(v);

                                                mem::drop(value);

                                                self.set_reg(dest, v);
                                            } else {
                                                let v = v.clone();

                                                mem::drop(value);

                                                self.change_reg_ref(dest, v);
                                            }
                                            break 'out;
                                        }
                                    },
                                    _ => (),
                                }

                                let Some(members) = self.impls.get(&base_type) else {
                                    error!(base_type)
                                };
                                let Some(r) = members.get(&key) else {
                                    error!(base_type)
                                };

                                let mut v = self.deep_clone(r.value());

                                if let Value::Macro(MacroData {
                                    self_arg,

                                    is_method,
                                    ..
                                }) = &mut v.value
                                {
                                    if *is_method {
                                        *self_arg = Some(self.get_reg_ref(from).clone())
                                    } else {
                                        return Err(RuntimeError::AssociatedMemberNotAMethod {
                                            area: self.make_area(opcode_span, &program),
                                            def_area: v.area.clone(),
                                            member_name: key.iter().collect(),
                                            member_type: v.value.get_type(),
                                            base_type,
                                            call_stack: self.get_call_stack(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::AssociatedMemberNotAMethod {
                                        area: self.make_area(opcode_span, &program),
                                        def_area: v.area.clone(),
                                        member_name: key.iter().collect(),
                                        member_type: v.value.get_type(),
                                        base_type,
                                        call_stack: self.get_call_stack(),
                                    });
                                }

                                mem::drop(value);

                                self.set_reg(dest, v);
                            }
                        }
                    },
                    Opcode::Associated { from, dest, member }
                    | Opcode::AssociatedMem { from, dest, member } => {
                        let is_mem = matches!(opcode, Opcode::AssociatedMem { .. });

                        let key = match &self.get_reg_ref(member).borrow().value {
                            Value::String(s) => Rc::clone(s),
                            _ => unreachable!(),
                        };

                        let value = self.get_reg_ref(from).borrow();

                        match &value.value {
                            Value::Type(t) => {
                                macro_rules! error {
                                    () => {
                                        return Err(RuntimeError::NonexistentAssociatedMember {
                                            area: self.make_area(opcode_span, &program),
                                            member: key.iter().collect(),
                                            base_type: *t,
                                            call_stack: self.get_call_stack(),
                                        })
                                    };
                                }
                                match self.impls.get(t) {
                                    Some(members) => match members.get(&key) {
                                        Some(elem) => {
                                            if !is_mem {
                                                let v = self.deep_clone(elem.value());

                                                mem::drop(value);

                                                self.set_reg(dest, v);
                                            } else {
                                                let v = elem.value().clone();

                                                mem::drop(value);

                                                self.change_reg_ref(dest, v);
                                            }
                                        },
                                        None => error!(),
                                    },
                                    None => error!(),
                                }
                            },
                            _ => {
                                return Err(RuntimeError::TypeMismatch {
                                    v: (value.value.get_type(), value.area.clone()),
                                    area: self.make_area(opcode_span, &program),
                                    expected: ValueType::Type,
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        }
                    },
                    Opcode::TypeOf { src, dest } => {
                        let t = self.get_reg_ref(src).borrow().value.get_type();
                        self.set_reg(
                            dest,
                            Value::Type(t).into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::Len { src, dest } => {
                        let len = match &self.get_reg_ref(src).borrow().value {
                            Value::Array(v) => v.len(),
                            Value::Dict(v) => v.len(),
                            Value::String(v) => v.len(),
                            v => {
                                // println!("{}", ip);
                                unreachable!()
                            },
                        };

                        self.set_reg(
                            dest,
                            Value::Int(len as i64)
                                .into_stored(self.make_area(opcode_span, &program)),
                        )
                    },

                    Opcode::MismatchThrowIfFalse {
                        check_reg,
                        value_reg,
                    } => {
                        let check = self.get_reg_ref(check_reg).borrow();
                        let matches = match &check.value {
                            Value::Bool(b) => *b,
                            _ => unreachable!(),
                        };
                        if !matches {
                            let pattern_area = check.area.clone();

                            let v = self.get_reg_ref(value_reg).borrow();
                            let v = (v.value.get_type(), v.area.clone());
                            return Err(RuntimeError::PatternMismatch {
                                v,
                                pattern_area,
                                call_stack: self.get_call_stack(),
                            });
                        }
                    },
                    Opcode::WrapMaybe { from, to } => {
                        let v = self.deep_clone_ref(from);
                        self.set_reg(
                            to,
                            Value::Maybe(Some(v))
                                .into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::TypeMember { from, dest, member } => {
                        let from = self.get_reg_ref(from).borrow();

                        let key = match &self.get_reg_ref(member).borrow().value {
                            Value::String(s) => s.clone(),
                            _ => unreachable!(),
                        };

                        match &from.value {
                            Value::Module { types, .. } => {
                                let typ = types
                                    .iter()
                                    .find(|k| *self.type_def_map[k.value()].name == *key)
                                    .ok_or(RuntimeError::NonexistentTypeMember {
                                        area: self.make_area(opcode_span, &program),
                                        type_name: key.iter().collect(),
                                        call_stack: self.get_call_stack(),
                                    })?;

                                if typ.is_priv() {
                                    return Err(RuntimeError::PrivateType {
                                        area: self.make_area(opcode_span, &program),
                                        type_name: key.iter().collect(),
                                        call_stack: self.get_call_stack(),
                                    });
                                }

                                let typ = *typ.value();

                                mem::drop(from);

                                self.set_reg(
                                    dest,
                                    StoredValue {
                                        value: Value::Type(ValueType::Custom(typ)),
                                        area: self.make_area(opcode_span, &program),
                                    },
                                );
                            },
                            v => {
                                return Err(RuntimeError::TypeMismatch {
                                    v: (v.get_type(), from.area.clone()),
                                    area: self.make_area(opcode_span, &program),
                                    expected: ValueType::Module,
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        }
                    },

                    Opcode::LoadEmpty { to } => load_val!(Value::Empty, to),
                    Opcode::LoadNone { to } => load_val!(Value::Maybe(None), to),
                    Opcode::LoadBuiltins { to } => load_val!(Value::Builtins, to),
                    Opcode::LoadEpsilon { to } => load_val!(Value::Epsilon, to),

                    Opcode::LoadArbitraryID { class, dest } => {
                        let id = Id::Arbitrary(self.next_id(class));
                        let v = match class {
                            IDClass::Group => Value::Group(id),
                            IDClass::Channel => Value::Channel(id),
                            IDClass::Block => Value::Block(id),
                            IDClass::Item => Value::Item(id),
                        };

                        self.set_reg(
                            dest,
                            StoredValue {
                                value: v,
                                area: self.make_area(opcode_span, &program),
                            },
                        )
                    },

                    Opcode::ApplyStringFlag { flag, reg } => {
                        let area = self.make_area(opcode_span, &program);

                        let val = match &self.get_reg_ref(reg).borrow().value {
                            Value::String(s) => {
                                let mut v = vec![];

                                match flag {
                                    RuntimeStringFlag::ByteString => {
                                        for c in s.iter() {
                                            let mut buf = [0u8; 4];

                                            for b in c.encode_utf8(&mut buf).bytes() {
                                                v.push(ValueRef::new(
                                                    Value::Int(b as i64).into_stored(area.clone()),
                                                ))
                                            }
                                        }

                                        Value::Array(v)
                                    },
                                    RuntimeStringFlag::Unindent => {
                                        let s = unindent::unindent(&s.iter().collect::<String>());

                                        Value::String(s.chars().collect_vec().into())
                                    },
                                    RuntimeStringFlag::Base64 => {
                                        let s = base64::engine::general_purpose::URL_SAFE
                                            .encode(s.iter().collect::<String>());

                                        Value::String(s.chars().collect_vec().into())
                                    },
                                }
                            },
                            _ => unreachable!(),
                        };

                        self.set_reg(reg, val.into_stored(area))
                    },
                    Opcode::ToString { from, dest } => {
                        let s = self.runtime_display(self.get_reg_ref(from), false);
                        self.set_reg(
                            dest,
                            Value::String(s.chars().collect())
                                .into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::MakeInstance { base, items, dest } => {
                        let base = self.get_reg_ref(base).borrow();

                        let t = match &base.value {
                            Value::Type(ValueType::Custom(t)) => *t,
                            Value::Type(t) => {
                                return Err(RuntimeError::CannotInstanceBuiltinType {
                                    area: self.make_area(opcode_span, &program),
                                    typ: *t,
                                    call_stack: self.get_call_stack(),
                                })
                            },
                            v => {
                                return Err(RuntimeError::TypeMismatch {
                                    v: (v.get_type(), base.area.clone()),
                                    expected: ValueType::Type,
                                    area: self.make_area(opcode_span, &program),
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        };
                        let items = match &self.get_reg_ref(items).borrow().value {
                            Value::Dict(t) => t.clone(),
                            _ => unreachable!(),
                        };
                        mem::drop(base);
                        self.set_reg(
                            dest,
                            Value::Instance { typ: t, items }
                                .into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::PushTryCatch { reg, to } => {
                        self.context_stack
                            .current_mut()
                            .try_catches
                            .push(TryCatch { jump_pos: to, reg });
                    },
                    Opcode::PopTryCatch => {
                        self.context_stack.current_mut().try_catches.pop();
                    },
                    Opcode::CreateMacro { func, dest } => {
                        let func: usize = func.into();
                        let func_coord = FuncCoord {
                            program: program.clone(),
                            func,
                        };
                        let f = &program.bytecode.functions[func];

                        let mut defaults = vec![None; f.args.len()];

                        let captured = f
                            .captured_regs
                            .iter()
                            .map(|(from, _)| self.get_reg_ref(*from).clone())
                            .collect_vec();

                        let value = Value::Macro(MacroData {
                            target: MacroTarget::Spwn {
                                func: func_coord,
                                is_builtin: None,
                                captured: captured.into(),
                            },

                            defaults: defaults.into(),
                            self_arg: None,
                            is_method: false,
                        });

                        self.set_reg(
                            dest,
                            value.into_stored(self.make_area(opcode_span, &program)),
                        );
                    },
                    Opcode::PushMacroDefault { to, from, arg } => {
                        match &mut self.get_reg_ref(to).borrow_mut().value {
                            Value::Macro(data) => {
                                data.defaults[arg as usize] = Some(self.get_reg_ref(from).clone());
                            },
                            _ => unreachable!(),
                        }
                    },
                    Opcode::MarkMacroMethod { reg } => {
                        match &mut self.get_reg_ref(reg).borrow_mut().value {
                            Value::Macro(data) => data.is_method = true,
                            _ => unreachable!(),
                        }
                    },
                    Opcode::Call { base, call } => {
                        // println!(
                        //     "tzuma {:?} {:?}",
                        //     self.get_reg_ref(Register(6)).as_ptr(),
                        //     self.get_reg_ref(Register(9)).as_ptr(),
                        // );
                        let call = program.get_call_expr(call);
                        let call = CallExpr {
                            dest: call.dest,
                            positional: call
                                .positional
                                .iter()
                                .map(|r| self.get_reg_ref(*r).clone())
                                .collect_vec()
                                .into(),
                            named: call
                                .named
                                .iter()
                                .map(|(s, r)| (s.clone(), self.get_reg_ref(*r).clone()))
                                .collect_vec()
                                .into(),
                        };

                        self.call_macro(
                            self.get_reg_ref(base).clone(),
                            &call,
                            &program,
                            self.make_area(opcode_span, &program),
                        )?;
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Impl { base, dict } => {
                        let t = match &self.get_reg_ref(base).borrow().value {
                            Value::Type(t) => *t,
                            _ => unreachable!(),
                        };

                        match t {
                            ValueType::Custom(..) => (),
                            _ => {
                                if !matches!(
                                    &*program.src,
                                    SpwnSource::Core(_) | SpwnSource::Std(_)
                                ) {
                                    // return Err(RuntimeError::ImplOnBuiltin {
                                    //     area: self.make_area(opcode_span, &program),
                                    //     call_stack: self.get_call_stack(),
                                    // });
                                }
                            },
                        }

                        let map = match &self.get_reg_ref(dict).borrow().value {
                            Value::Dict(t) => t.clone(),
                            _ => unreachable!(),
                        };

                        for (k, v) in &map {
                            if let Value::Macro(MacroData {
                                target: MacroTarget::Spwn { is_builtin, .. },
                                ..
                            }) = &mut v.value().borrow_mut().value
                            {
                                if let Some(f) = t.get_override_fn(&k.iter().collect::<String>()) {
                                    *is_builtin = Some(f)
                                }
                            }
                        }

                        self.impls.insert(t, map);
                    },
                    Opcode::RunBuiltin { args, dest } => {
                        if let Some(f) = is_builtin {
                            let area = self.make_area(opcode_span, &program);
                            self.context_stack
                                .current_mut()
                                .stack
                                .last_mut()
                                .unwrap()
                                .store_extra = Some(dest);
                            f.0(
                                (0..args)
                                    .map(|r| self.get_reg_ref(Register(r)).clone())
                                    .collect_vec(),
                                self,
                                &program,
                                area,
                            )?;
                            return Ok(LoopFlow::ContinueLoop);
                        } else {
                            panic!("mcock")
                        }
                    },
                    Opcode::MakeTriggerFunc { src, dest } => {
                        let group = match &self.get_reg_ref(src).borrow().value {
                            Value::Group(g) => *g,
                            _ => unreachable!(),
                        };

                        self.set_reg(
                            dest,
                            Value::TriggerFunction {
                                group,
                                prev_context: self.context_stack.last().current_group(),
                            }
                            .into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::CallTriggerFunc { func } => {
                        let v = self.get_reg_ref(func).borrow();
                        let target = match &v.value {
                            Value::TriggerFunction { group, .. } => *group,
                            Value::Group(g) => *g,
                            _ => {
                                return Err(RuntimeError::TypeMismatch {
                                    v: (v.value.get_type(), v.area.clone()),
                                    area: self.make_area(opcode_span, &program),
                                    expected: ValueType::TriggerFunction, // OR group idk how to make this error do that
                                    call_stack: self.get_call_stack(),
                                });
                            },
                        };
                        mem::drop(v);
                        let trigger = make_spawn_trigger(
                            self.context_stack.last().current_group(),
                            target,
                            self,
                        );
                        self.triggers.push(trigger);
                    },
                    Opcode::SetContextGroup { reg } => {
                        let group = match &self.get_reg_ref(reg).borrow().value {
                            Value::Group(g) => *g,
                            Value::TriggerFunction { prev_context, .. } => *prev_context,
                            _ => unreachable!(),
                        };
                        self.context_stack.last_mut().set_group(group);
                    },
                }
                Ok(LoopFlow::Normal)
            };

            match run_opcode(opcode) {
                Ok(flow) => match flow {
                    LoopFlow::ContinueLoop => {
                        continue;
                    },
                    LoopFlow::Normal => {},
                },
                Err(err) => {
                    let t = self.context_stack.current_mut().try_catches.pop();
                    if let Some(try_catch) = t {
                        assert_eq!(
                            std::mem::size_of::<std::mem::Discriminant<RuntimeError>>(),
                            std::mem::size_of::<u64>()
                        );

                        let val = match err {
                            RuntimeError::ThrownError { value, .. } => value,
                            _ => Value::Error(unsafe {
                                std::mem::transmute::<_, u64>(std::mem::discriminant(&err)) as usize
                            })
                            .into_stored(self.make_area(ZEROSPAN, &program)),
                        };

                        self.set_reg(try_catch.reg, val);

                        self.context_stack
                            .last_mut()
                            .jump_current(*try_catch.jump_pos as usize);
                        continue;
                    } else {
                        Err(err)?;
                    }
                },
            }

            {
                let mut current = self.context_stack.current_mut();
                current.ip += 1;
            };

            // self.try_merge_contexts();
        }

        //let f = self.context_stack.pop().unwrap();
        // if return_dest.is_some() {
        //     let mut current = self.context_stack.current_mut();
        //     current.ip -= 1;
        // };

        // println!("g: {} {}", finished_contexts.len(), func);
        self.context_stack.pop().unwrap();
        if return_dest.is_some() {
            self.context_stack
                .last_mut()
                .contexts
                .extend(finished_contexts.into_iter());
        }

        Ok(SplitFnRet)
    }

    fn try_merge_contexts(&mut self) {
        let mut top = vec![];
        {
            let full_ctx = self.context_stack.last_mut();

            if full_ctx.contexts.len() <= 1 {
                return;
            }

            // group by pos

            let top_ip = full_ctx.current().ip;
            while !full_ctx.contexts.is_empty() {
                if full_ctx.current().ip == top_ip {
                    top.push(full_ctx.contexts.pop().unwrap());
                } else {
                    break;
                }
            }
        }

        if top.len() > 1 {
            // hash the contexts
            let mut hashes = AHashMap::new();
            for ctx in top {
                let mut state = DefaultHasher::default();
                for val in ctx.stack.last().unwrap().registers.iter() {
                    self.hash_value(val, &mut state);
                }
                // for val in ctx.extra_stack.iter() {
                //     self.hash_value(val, &mut state);
                // }
                let hash = state.finish();
                hashes.entry(hash).or_insert_with(Vec::new).push(ctx);
            }

            // merge the contexts
            for (_, ctxs) in hashes {
                if ctxs.len() > 1 {
                    let mut iter = ctxs.into_iter();
                    let ctx = iter.next().unwrap();
                    let target = ctx.group;
                    for ctx2 in iter {
                        // add a spawn trigger in this context to the merged context
                        let trigger = make_spawn_trigger(ctx2.group, target, self);
                        self.triggers.push(trigger);
                    }
                    self.context_stack.last_mut().contexts.push(ctx);
                } else {
                    self.context_stack.last_mut().contexts.extend(ctxs);
                }
            }
        } else {
            self.context_stack.last_mut().contexts.extend(top);
        }
    }

    pub fn call_macro(
        &mut self,
        base: ValueRef,
        call_expr: &CallExpr<ValueRef, OptRegister, Box<str>>,
        program: &Rc<Program>,
        area: CodeArea,
    ) -> Result<(), RuntimeError>
// where
    //     F1: FnOnce(&Self) -> ValueRef,
    //     F2: FnOnce(&Self) -> CallExpr<ValueRef, OptRegister, Box<str>>,
    {
        let base = base.borrow();

        let macro_area = base.area.clone();

        let data = match &base.value {
            Value::Macro(data) => data,
            v => {
                return Err(RuntimeError::TypeMismatch {
                    v: (v.get_type(), macro_area),
                    expected: ValueType::Macro,
                    area: area.clone(),
                    call_stack: self.get_call_stack(),
                })
            },
        };

        let (args, spread_arg): (Box<dyn Iterator<Item = Option<&ImmutStr>>>, Option<u8>) =
            match &data.target {
                MacroTarget::Spwn {
                    func,
                    is_builtin,
                    captured,
                } => {
                    let f = func.get_func();
                    (
                        Box::new(f.args.iter().map(|v| v.value.as_ref())),
                        f.spread_arg,
                    )
                },
                MacroTarget::FullyRust {
                    fn_ptr,
                    args,
                    spread_arg,
                } => (Box::new(args.iter().map(|f| Some(f))), *spread_arg),
            };

        // let func = data.func.get_func();

        enum ArgFill {
            Single(Option<ValueRef>, Option<ImmutStr>),
            Spread(Vec<ValueRef>),
        }

        let mut arg_name_map = AHashMap::new();

        let mut fill = args
            .enumerate()
            .map(|(i, arg)| {
                if let Some(name) = &arg {
                    arg_name_map.insert(&**name, i);
                }

                if spread_arg == Some(i as u8) {
                    ArgFill::Spread(vec![])
                } else {
                    ArgFill::Single(None, arg.cloned())
                }
            })
            .collect_vec();

        let mut next_arg_idx = 0;

        if data.is_method {
            match fill.get_mut(next_arg_idx) {
                Some(ArgFill::Single(v, _)) => {
                    *v = data.self_arg.clone();
                    next_arg_idx += 1;
                },
                _ => unreachable!(),
            }
        }

        for arg in call_expr.positional.iter() {
            match fill.get_mut(next_arg_idx) {
                Some(ArgFill::Single(opt, ..)) => {
                    *opt = Some(arg.clone());
                    next_arg_idx += 1;
                },
                Some(ArgFill::Spread(s)) => s.push(arg.clone()),
                None => {
                    return Err(RuntimeError::TooManyArguments {
                        call_area: area,
                        macro_def_area: macro_area,
                        call_arg_amount: call_expr.positional.len(),
                        macro_arg_amount: fill.len(),
                        call_stack: self.get_call_stack(),
                    })
                },
            }
        }

        for (name, arg) in call_expr.named.iter() {
            let Some(idx) = arg_name_map.get(name) else {
                return Err(RuntimeError::UnknownKeywordArgument {
                    name: name.to_string(),
                    macro_def_area: macro_area,
                    call_area: area,
                    call_stack: self.get_call_stack(),
                });
            };
            match &mut fill[*idx] {
                ArgFill::Single(opt, ..) => {
                    *opt = Some(arg.clone());
                },
                ArgFill::Spread(_) => {
                    return Err(RuntimeError::UnknownKeywordArgument {
                        name: name.to_string(),
                        macro_def_area: macro_area,
                        call_area: area,
                        call_stack: self.get_call_stack(),
                    })
                },
            }
        }

        match &data.target {
            MacroTarget::Spwn {
                func,
                is_builtin,
                captured,
            } => {
                let func = func.clone();
                let is_builtin = *is_builtin;
                let captured = captured.clone();

                mem::drop(base);

                let current_context = self.context_stack.last_mut().yeet_current().unwrap();

                self.run_function(
                    current_context,
                    CallInfo {
                        func,
                        return_dest: Some(
                            call_expr
                                .dest
                                .map(ReturnDest::Reg)
                                .unwrap_or(ReturnDest::Extra),
                        ),
                        call_area: Some(area.clone()),
                        is_builtin,
                    },
                    Box::new(move |vm| {
                        let arg_amount = fill.len();
                        for (i, arg) in fill.into_iter().enumerate() {
                            match arg {
                                ArgFill::Single(Some(r), ..) => {
                                    vm.change_reg_ref(Register(i as u8), r);
                                },
                                ArgFill::Spread(v) => {
                                    vm.set_reg(
                                        Register(i as u8),
                                        StoredValue {
                                            value: Value::Array(v),
                                            area: area.clone(),
                                        },
                                    );
                                },
                                ArgFill::Single(None, name) => {
                                    return Err(RuntimeError::ArgumentNotSatisfied {
                                        call_area: area.clone(),
                                        macro_def_area: macro_area,
                                        arg: if let Some(name) = name {
                                            Either::Left(name.to_string())
                                        } else {
                                            Either::Right(i)
                                        },
                                        call_stack: vm.get_call_stack(),
                                    })
                                },
                            }
                        }

                        for (i, v) in captured.iter().enumerate() {
                            vm.change_reg_ref(Register((arg_amount + i) as u8), v.clone());
                        }

                        Ok(())
                    }),
                    // ContextSplitMode::Allow,
                )?;
            },
            MacroTarget::FullyRust { fn_ptr, .. } => {
                let fn_ptr = fn_ptr.clone();
                mem::drop(base);

                fn_ptr.0.borrow_mut()(
                    {
                        let mut out = Vec::with_capacity(fill.len());
                        for (i, arg) in fill.into_iter().enumerate() {
                            out.push(match arg {
                                ArgFill::Single(Some(r), ..) => r,
                                ArgFill::Spread(v) => ValueRef::new(StoredValue {
                                    value: Value::Array(v),
                                    area: area.clone(),
                                }),
                                ArgFill::Single(None, name) => {
                                    return Err(RuntimeError::ArgumentNotSatisfied {
                                        call_area: area.clone(),
                                        macro_def_area: macro_area,
                                        arg: if let Some(name) = name {
                                            Either::Left(name.to_string())
                                        } else {
                                            Either::Right(i)
                                        },
                                        call_stack: self.get_call_stack(),
                                    })
                                },
                            })
                        }
                        out
                    },
                    self,
                    program,
                    area,
                )?;
            },
        }

        Ok(())
    }

    #[inline]
    pub fn bin_op(
        &mut self,
        op: fn(&StoredValue, &StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>,
        program: &Rc<Program>,
        left: OptRegister,
        right: OptRegister,
        dest: OptRegister,
        span: CodeSpan,
    ) -> Result<(), RuntimeError> {
        // TODO: overloads

        #[rustfmt::skip]
        let value = self.borrow_reg(left, |left| {
            self.borrow_reg(right, |right| {
                op(
                    &left,
                    &right,
                    span,
                    self,
                    program,
                )
            })
        })?;

        self.set_reg(
            dest,
            StoredValue {
                value,
                area: self.make_area(span, program),
            },
        );
        Ok(())
    }

    #[inline]
    fn assign_op(
        &mut self,
        op: fn(&StoredValue, &StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>,
        program: &Rc<Program>,
        left: OptRegister,
        right: OptRegister,
        span: CodeSpan,
    ) -> Result<(), RuntimeError> {
        #[rustfmt::skip]
        let value = self.borrow_reg(left, |left| {
            self.borrow_reg(right, |right| {
                op(
                    &left,
                    &right,
                    span,
                    self,
                    program,
                )
            })
        })?;

        self.borrow_reg_mut(left, |mut v| {
            v.value = value;
            Ok(())
        })?;
        Ok(())
    }

    #[inline]
    fn unary_op(
        &mut self,
        op: fn(&StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>,
        program: &Rc<Program>,
        value: OptRegister,
        dest: OptRegister,
        span: CodeSpan,
    ) -> Result<(), RuntimeError> {
        #[rustfmt::skip]
        let value = self.borrow_reg(value, |value| {
            op(
                &value,
                span,
                self,
                program,
            )
        })?;

        self.set_reg(
            dest,
            StoredValue {
                value,
                area: self.make_area(span, program),
            },
        );
        Ok(())
    }

    pub fn convert_type(
        &self,
        v: &StoredValue,
        b: ValueType,
        span: CodeSpan, // ✍️
        program: &Rc<Program>,
    ) -> RuntimeResult<Value> {
        if v.value.get_type() == b {
            return Ok(v.value.clone());
        }

        Ok(match (&v.value, b) {
            _ => todo!("error"),
        })
    }
}

impl Vm {
    pub fn hash_value(&self, val: &ValueRef, state: &mut DefaultHasher) {
        // todo: rest
        std::mem::discriminant(&val.borrow().value).hash(state);

        match &val.borrow().value {
            Value::Int(n) => n.hash(state),
            Value::Float(a) => a.to_bits().hash(state),
            Value::Bool(a) => a.hash(state),
            Value::String(a) => a.hash(state),
            Value::Array(a) => {
                for v in a {
                    self.hash_value(v, state)
                }
            },
            Value::Dict(a) => {
                let a: BTreeMap<_, _> = a.iter().collect();
                for (k, v) in a {
                    self.hash_value(v.value(), state);
                    v.source().hash(state);
                    k.hash(state);
                }
            },
            Value::Group(a) => a.hash(state),
            Value::Channel(a) => a.hash(state),
            Value::Block(a) => a.hash(state),
            Value::Item(a) => a.hash(state),
            Value::Builtins => (),
            Value::Range { start, end, step } => (start, end, step).hash(state),
            Value::Maybe(a) => {
                if let Some(v) = a {
                    self.hash_value(v, state)
                }
            },
            Value::Empty => (),
            Value::Type(a) => a.hash(state),
            Value::Module { exports, types } => {
                let a: BTreeMap<_, _> = exports.iter().collect();
                for (k, v) in a {
                    self.hash_value(v, state);
                    k.hash(state);
                }
                types.hash(state)
            },
            Value::TriggerFunction {
                group: g,
                prev_context: p,
            } => (g, p).hash(state),
            Value::Error(a) => a.hash(state),
            Value::Macro(data) | Value::Iterator(data) => {
                let MacroData {
                    target,
                    defaults,
                    self_arg,
                    is_method,
                } = data;

                match target {
                    MacroTarget::Spwn {
                        func,
                        is_builtin,
                        captured,
                    } => {
                        func.hash(state);
                        is_builtin.hash(state);
                        for i in captured.iter() {
                            self.hash_value(i, state);
                        }
                    },
                    MacroTarget::FullyRust {
                        fn_ptr,
                        args,
                        spread_arg,
                    } => {
                        fn_ptr.hash(state);
                        args.hash(state);
                        spread_arg.hash(state);
                    },
                }

                is_method.hash(state);

                for r in defaults.iter().flatten() {
                    self.hash_value(r, state)
                }

                if let Some(v) = self_arg {
                    self.hash_value(v, state)
                }
                // data.func.h
                // data.hash(state)
            },
            Value::Epsilon => (),
            Value::Instance { typ, items } => {
                let items: BTreeMap<_, _> = items.iter().collect();
                typ.hash(state);
                for (k, v) in items {
                    self.hash_value(v.value(), state);
                    v.source().hash(state);
                    k.hash(state);
                }
            },
            Value::Chroma { r, g, b, a } => (r, g, b, a).hash(state),
            Value::ObjectKey(a) => a.hash(state),
        }
    }

    pub fn runtime_display(&self, value: &ValueRef, with_ptr: bool) -> String {
        let mut out = if with_ptr {
            format!("{:?}: ", value.as_ptr()).dimmed().to_string()
        } else {
            "".to_string()
        };

        out += &match &value.borrow().value {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("{:?}", s.iter().collect::<String>()),
            Value::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|k| self.runtime_display(k, with_ptr))
                    .join(", ")
            ),
            Value::Dict(d) => format!(
                "{{ {} }}",
                d.iter()
                    .map(|(s, v)| format!(
                        "{}: {}",
                        s.iter().collect::<String>(),
                        self.runtime_display(v.value(), with_ptr)
                    ))
                    .join(", ")
            ),
            Value::Group(id) => id.fmt("g"),
            Value::Channel(id) => id.fmt("c"),
            Value::Block(id) => id.fmt("b"),
            Value::Item(id) => id.fmt("i"),
            Value::Builtins => "$".to_string(),
            Value::Chroma { r, g, b, a } => format!("@chroma::rgb8({r}, {g}, {b}, {a})"),
            Value::Range { start, end, step } => {
                if *step == 1 {
                    format!("{start}..{end}")
                } else {
                    format!("{start}..{step}..{end}")
                }
            },
            Value::Maybe(o) => match o {
                Some(k) => format!("({})?", self.runtime_display(k, with_ptr)),
                None => "?".into(),
            },
            Value::Empty => "()".into(),

            Value::Macro(MacroData { defaults, .. }) => {
                format!("<{}-arg macro at {:?}>", defaults.len(), value.as_ptr())
            },
            Value::Iterator(_) => {
                format!("<iterator at {:?}>", value.as_ptr())
            },
            Value::TriggerFunction { .. } => "!{...}".to_string(),
            Value::Type(t) => t.runtime_display(self),
            // Value::Object(map, typ) => format!(
            //     "{} {{ {} }}",
            //     match typ {
            //         ObjectType::Object => "obj",
            //         ObjectType::Trigger => "trigger",
            //     },
            //     map.iter()
            //         .map(|(s, k)| format!("{s}: {k:?}"))
            //         .collect::<Vec<_>>()
            //         .join(", ")
            // ),
            Value::Epsilon => "$.epsilon()".to_string(),
            Value::Module { exports, types } => format!(
                "module {{ {}{} }}",
                exports
                    .iter()
                    .map(|(s, k)| format!(
                        "{}: {}",
                        s.iter().collect::<String>(),
                        self.runtime_display(k, with_ptr)
                    ))
                    .join(", "),
                if types.iter().any(|p| p.is_pub()) {
                    format!(
                        "; {}",
                        types
                            .iter()
                            .filter(|p| p.is_pub())
                            .map(|p| ValueType::Custom(*p.value()).runtime_display(self))
                            .join(", ")
                    )
                } else {
                    "".into()
                }
            ),

            // Value::Iterator(_) => "<iterator>".into(),
            // Value::ObjectKey(k) => format!("$.obj_props.{}", <ObjectKey as Into<&str>>::into(*k)),
            Value::Error(id) => {
                use delve::VariantNames;
                format!(
                    "{} {{...}}",
                    crate::interpreting::error::ErrorDiscriminants::VARIANT_NAMES[*id]
                )
            },

            Value::Instance { typ, items } => format!(
                "@{}::{{ {} }}",
                self.type_def_map[&typ].name.iter().collect::<String>(),
                items
                    .iter()
                    .map(|(s, v)| format!(
                        "{}: {}",
                        s.iter().collect::<String>(),
                        self.runtime_display(v.value(), with_ptr)
                    ))
                    .join(", ")
            ),
            Value::ObjectKey(_) => todo!(),
            // todo: iterator, object
        };
        out
    }
}

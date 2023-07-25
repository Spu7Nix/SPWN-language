use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::{self};
use std::rc::Rc;

use ahash::AHashMap;
use base64::Engine;
use colored::Colorize;
use delve::VariantNames;
use derive_more::{Deref, DerefMut};
use itertools::{Either, Itertools};

use super::context::{CallInfo, CloneMap, Context, ContextStack, FullContext, StackItem};
use super::multi::Multi;
use super::value::{MacroTarget, StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::{Bytecode, CallExpr, Constant, Function, OptRegister, Register};
use crate::compiling::opcodes::{CallExprID, ConstID, Opcode, RuntimeStringFlag};
use crate::gd::gd_object::{make_spawn_trigger, TriggerObject, TriggerOrder};
use crate::gd::ids::{IDClass, Id};
use crate::gd::object_keys::OBJECT_KEYS;
use crate::interpreting::context::TryCatch;
use crate::interpreting::error::RuntimeError;
use crate::interpreting::value::MacroData;
use crate::parsing::ast::{Vis, VisSource, VisTrait};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spanned, SpwnSource, TypeDefMap, ZEROSPAN};
use crate::util::{ImmutCloneVec, ImmutStr, ImmutVec};

const RECURSION_LIMIT: usize = 256;

pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type RuntimeCtxResult<T> = Result<T, (Context, RuntimeError)>;

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

impl Hash for ValueRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state);
    }
}

impl From<StoredValue> for ValueRef {
    fn from(value: StoredValue) -> Self {
        Self::new(value)
    }
}

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

    pub fn make_area(span: CodeSpan, program: &Rc<Program>) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&program.src),
        }
    }

    pub fn insert_multi<T, F>(&mut self, v: Multi<T>, mut f: F)
    where
        F: FnMut(&mut Context, T),
    {
        for (mut c, v) in v {
            f(&mut c, v);
            self.context_stack.last_mut().contexts.push(c)
        }
    }

    pub fn change_reg<T: Into<ValueRef>>(&mut self, reg: OptRegister, v: T) {
        self.context_stack
            .current_mut()
            .stack
            .last_mut()
            .unwrap()
            .registers[*reg as usize] = v.into()
    }

    pub fn change_reg_multi<T: Into<ValueRef>>(&mut self, reg: OptRegister, v: Multi<T>) {
        // self.contexts.last_mut().unwrap().yeet_current().unwrap();

        self.insert_multi(v, |ctx, v| {
            ctx.stack.last_mut().unwrap().registers[*reg as usize] = v.into();
        });
    }

    pub fn write_pointee(&mut self, reg: OptRegister, v: StoredValue) {
        let mut binding = self.context_stack.current_mut();
        let mut g = binding.stack.last_mut().unwrap().registers[reg.0 as usize].borrow_mut();
        *g = v;
    }

    pub fn write_pointee_multi(&mut self, reg: OptRegister, v: Multi<StoredValue>) {
        self.insert_multi(v, |ctx, v| {
            let mut g = ctx.stack.last_mut().unwrap().registers[reg.0 as usize].borrow_mut();
            *g = v;
        });
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

    pub fn next_id(&mut self, c: IDClass) -> u16 {
        self.id_counters[c as usize] += 1;
        self.id_counters[c as usize]
    }

    pub fn get_impl(&self, typ: ValueType, name: &str) -> Option<&VisSource<ValueRef>> {
        self.impls
            .get(&typ)
            .and_then(|v| v.get::<Rc<[char]>>(&name.chars().collect_vec().into()))
    }

    pub fn set_error(&mut self, v: RuntimeError) {
        self.context_stack.current_mut().set_error(v);
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
                for r in new_data.defaults.iter_mut().flatten() {
                    // if let Some(r) = i {
                    *r = r.deep_clone_checked(self, map)
                    // }
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

    /// <img src="https://cdna.artstation.com/p/assets/images/images/056/833/046/original/lara-hughes-blahaj-spin-compressed.gif?1670214805" width=60 height=60>
    /// <img src="https://cdna.artstation.com/p/assets/images/images/056/833/046/original/lara-hughes-blahaj-spin-compressed.gif?1670214805" width=60 height=60>
    /// <img src="https://cdna.artstation.com/p/assets/images/images/056/833/046/original/lara-hughes-blahaj-spin-compressed.gif?1670214805" width=60 height=60>
    pub fn run_function(
        &mut self,
        mut context: Context,
        call_info: CallInfo,
        cb: Box<dyn FnOnce(&mut Vm)>,
    ) -> Multi<ValueRef> {
        let program = call_info.func.program.clone();
        let func = call_info.func.func;

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
                try_catches: vec![],
            });
        }

        let original_ip = context.ip;
        context.ip = 0;

        self.context_stack
            .push(FullContext::new(context, call_info));

        cb(self);

        let opcodes = &program.get_function(func).opcodes;

        let mut out_contexts = vec![];

        if self.context_stack.len() > RECURSION_LIMIT {
            self.set_error(RuntimeError::RecursionLimit {
                area: Vm::make_area(ZEROSPAN, &program),
                call_stack: self.get_call_stack(),
            });
        }

        while self.context_stack.last().valid() {
            let ip = self.context_stack.current().ip;

            if ip >= opcodes.len() {
                if !self.context_stack.last().have_returned {
                    let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                    top.stack.pop();

                    out_contexts.push(top);
                } else {
                    self.context_stack.last_mut().yeet_current();
                }
                continue;
            }

            let Spanned {
                value: opcode,
                span: opcode_span,
            } = opcodes[ip];

            // println!("{}", ip);

            {
                let mut ctx = self.context_stack.current_mut();

                if let Some(err) = ctx.errored.take() {
                    if let Some(s) = ctx.stack.last_mut().unwrap().try_catches.pop() {
                        let val = match err {
                            RuntimeError::ThrownError { value, .. } => value,
                            _ => Value::Error(err)
                                .into_stored(Vm::make_area(ZEROSPAN, &program))
                                .into(),
                        };

                        ctx.stack.last_mut().unwrap().registers[*s.reg as usize] = val;

                        ctx.ip = *s.jump_pos as usize;
                        continue;
                    } else {
                        ctx.errored = Some(err);
                        mem::drop(ctx);

                        let mut ctx = self.context_stack.last_mut().yeet_current().unwrap();
                        ctx.stack.pop();

                        out_contexts.push(ctx);

                        continue;
                    }
                }
            }

            macro_rules! handle_ctx_result {
                ($e:expr, ($($r:tt)*) => {$($t:tt)*}) => {
                    match $e {
                        Ok($($r)*) => {
                            $($t)*
                        },
                        Err((mut ctx, err)) => {
                            ctx.set_error(err);
                            self.context_stack.last_mut().contexts.push(ctx);
                        },
                    }
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
                // println!(
                //     "{}    {:<10}{:<6}{}",
                //     self.context_stack.last().contexts.len(),
                //     format!(
                //         "{}{},",
                //         " ".repeat(self.context_stack.current().unique_id),
                //         self.context_stack.current().unique_id,
                //     ),
                //     format!("{}{}->", " ".repeat(func), func),
                //     ip
                // );
                match opcode {
                    Opcode::LoadConst { id, to } => {
                        let value = Value::from_const(self, program.get_constant(id));
                        self.change_reg(
                            to,
                            value.into_stored(Vm::make_area(opcode_span, &program)),
                        );
                    },
                    Opcode::CopyDeep { from, to } => self.change_reg(to, self.deep_clone(from)),
                    Opcode::CopyShallow { from, to } => {
                        let new = self.get_reg_ref(from).borrow().clone();
                        self.change_reg(to, new)
                    },
                    Opcode::CopyRef { from, to } => {
                        let v = self.get_reg_ref(from).clone();
                        self.change_reg(to, v);
                    },
                    Opcode::Write { from, to } => {
                        let v = self.get_reg_ref(from).borrow().clone();
                        self.write_pointee(to, v)
                    },
                    Opcode::WriteDeep { from, to } => self.write_pointee(to, self.deep_clone(from)),
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
                        self.context_stack.current_mut().ip = *to as usize;
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::JumpIfFalse { check, to } => {
                        let b = value_ops::to_bool(
                            &self.get_reg_ref(check).borrow(),
                            opcode_span,
                            self,
                            &program,
                        )?;

                        if !b {
                            self.context_stack.current_mut().ip = *to as usize;
                            return Ok(LoopFlow::ContinueLoop);
                        }
                    },
                    Opcode::JumpIfTrue { check, to } => {
                        let b = value_ops::to_bool(
                            &self.get_reg_ref(check).borrow(),
                            opcode_span,
                            self,
                            &program,
                        )?;

                        if b {
                            self.context_stack.current_mut().ip = *to as usize;
                            return Ok(LoopFlow::ContinueLoop);
                        }
                    },
                    Opcode::UnwrapOrJump { check, to } => {
                        let v = self.get_reg_ref(check).clone();
                        let vref = v.borrow();

                        match &vref.value {
                            Value::Maybe(v) => match v {
                                Some(k) => {
                                    let val = self.deep_clone(k);

                                    mem::drop(vref);

                                    self.change_reg(check, val);
                                },
                                None => {
                                    mem::drop(vref);
                                    self.context_stack.current_mut().ip = *to as usize;
                                    return Ok(LoopFlow::ContinueLoop);
                                },
                            },
                            v => unreachable!("{:?}", v),
                        };
                    },
                    Opcode::IntoIterator { src, dest } => todo!(),
                    Opcode::IterNext { src, dest } => todo!(),
                    Opcode::AllocArray { dest, len } => self.change_reg(
                        dest,
                        StoredValue {
                            value: Value::Array(Vec::with_capacity(len as usize)),
                            area: Vm::make_area(opcode_span, &program),
                        },
                    ),
                    Opcode::PushArrayElem { elem, dest } => {
                        let push = self.deep_clone_ref(elem);
                        match &mut self.get_reg_ref(dest).borrow_mut().value {
                            Value::Array(v) => v.push(push),
                            _ => unreachable!(),
                        }
                    },
                    Opcode::AllocDict { dest, capacity } => self.change_reg(
                        dest,
                        StoredValue {
                            value: Value::Dict(AHashMap::with_capacity(capacity as usize)),
                            area: Vm::make_area(opcode_span, &program),
                        },
                    ),
                    Opcode::InsertDictElem { elem, dest, key } => {
                        let push = self.deep_clone_ref(elem);

                        let key = self.borrow_reg(key, |key| match &key.value {
                            Value::String(s) => Ok(s.clone()),
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
                            Value::String(s) => Ok(s.clone()),
                            _ => unreachable!(),
                        })?;

                        match &mut self.get_reg_ref(dest).borrow_mut().value {
                            Value::Dict(v) => {
                                v.insert(key, VisSource::Private(push, program.src.clone()))
                            },
                            _ => unreachable!(),
                        };
                    },
                    Opcode::MakeInstance { base, items, dest } => {
                        let base = self.get_reg_ref(base).borrow();

                        let t = match &base.value {
                            Value::Type(ValueType::Custom(t)) => *t,
                            Value::Type(t) => {
                                return Err(RuntimeError::CannotInstanceBuiltinType {
                                    area: Vm::make_area(opcode_span, &program),
                                    typ: *t,
                                    call_stack: self.get_call_stack(),
                                })
                            },
                            v => {
                                return Err(RuntimeError::TypeMismatch {
                                    value_type: v.get_type(),
                                    value_area: base.area.clone(),
                                    expected: &[ValueType::Type],
                                    area: Vm::make_area(opcode_span, &program),
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        };
                        let items = match &self.get_reg_ref(items).borrow().value {
                            Value::Dict(t) => t.clone(),
                            _ => unreachable!(),
                        };
                        mem::drop(base);
                        self.change_reg(
                            dest,
                            Value::Instance { typ: t, items }
                                .into_stored(Vm::make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::EnterArrowStatement { skip } => {
                        // println!("futa");
                        // println!("smeagol: {}", self.context_stack.current().unique_id);
                        self.split_current_context();
                        // println!("nari");
                        self.context_stack.current_mut().ip = *skip as usize;
                    },
                    Opcode::YeetContext => {
                        self.context_stack.last_mut().yeet_current();
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::LoadEmpty { to } => self.change_reg(
                        to,
                        Value::Empty.into_stored(Vm::make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadNone { to } => self.change_reg(
                        to,
                        Value::Maybe(None).into_stored(Vm::make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadBuiltins { to } => self.change_reg(
                        to,
                        Value::Builtins.into_stored(Vm::make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadEpsilon { to } => self.change_reg(
                        to,
                        Value::Epsilon.into_stored(Vm::make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadArbitraryID { class, dest } => {
                        let id = Id::Arbitrary(self.next_id(class));
                        let v = match class {
                            IDClass::Group => Value::Group(id),
                            IDClass::Channel => Value::Channel(id),
                            IDClass::Block => Value::Block(id),
                            IDClass::Item => Value::Item(id),
                        };

                        self.change_reg(
                            dest,
                            StoredValue {
                                value: v,
                                area: Vm::make_area(opcode_span, &program),
                            },
                        )
                    },
                    Opcode::ApplyStringFlag { flag, reg } => todo!(),
                    Opcode::WrapMaybe { from, to } => {
                        let v = self.deep_clone_ref(from);
                        self.change_reg(
                            to,
                            Value::Maybe(Some(v)).into_stored(Vm::make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::Return { src, module_ret } => {
                        let ret_val = self.get_reg_ref(src).clone();

                        if module_ret {
                            let mut r = ret_val.borrow_mut();

                            match &r.value {
                                Value::Dict(d) => {
                                    r.value = Value::Module {
                                        exports: d
                                            .iter()
                                            .map(|(s, v)| (s.clone(), v.value().clone()))
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

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.stack.pop();

                        top.returned = Some(ret_val);
                        out_contexts.push(top);

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Dbg { reg, show_ptr } => {
                        let r = self.get_reg_ref(reg).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        handle_ctx_result! {
                            self.runtime_display(
                                top,
                                &r,
                                &Vm::make_area(opcode_span, &program)
                            ),
                            (s) => {
                                self.insert_multi(s, |ctx, v| {
                                    println!(
                                        "{} {} {} {} {}",
                                        v,
                                        "::".dimmed(),
                                        ctx.unique_id.to_string().bright_blue(),
                                        ctx.group.fmt("g").green(),
                                        format!("{:?}", r.as_ptr()).dimmed(),
                                        // ctx.stack.len(),
                                    );
                                });
                            }
                        }

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Throw { reg } => todo!(),
                    Opcode::Import { id, dest } => todo!(),
                    Opcode::ToString { from, dest } => {
                        let r = self.get_reg_ref(from).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let area = Vm::make_area(opcode_span, &program);

                        handle_ctx_result! {
                            self.runtime_display(
                                top,
                                &r,
                                &area
                            ),
                            (s) => {
                                self.change_reg_multi(
                                    dest,
                                    s.map(|ctx, v| {
                                        (
                                            ctx,
                                            ValueRef::new(
                                                Value::String(v.chars().collect())
                                                    .into_stored(area.clone()),
                                            ),
                                        )
                                    }),
                                );
                            }
                        }

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Index { base, dest, index } => {
                        let base = self.get_reg_ref(base).clone();
                        let index = self.get_reg_ref(index).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        handle_ctx_result! {
                            self.index_value(
                                top,
                                &base,
                                &index,
                                &Vm::make_area(opcode_span, &program),
                            ),
                            (ret) => {
                                self.change_reg_multi(dest, ret);
                            }
                        }

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Member { from, dest, member } => {
                        let key = match &self.get_reg_ref(member).borrow().value {
                            Value::String(s) => s.clone(),
                            _ => unreachable!(),
                        };
                        let key_str = key.iter().collect::<String>();

                        let value = self.get_reg_ref(from).borrow();

                        let special = match (&value.value, &key_str[..]) {
                            (Value::String(s), "length") => Some(Value::Int(s.len() as i64)),

                            (Value::Range { start, .. }, "start") => Some(Value::Int(*start)),
                            (Value::Range { end, .. }, "end") => Some(Value::Int(*end)),
                            (Value::Range { step, .. }, "step") => Some(Value::Int(*step as i64)),

                            (Value::Array(v), "length") => Some(Value::Int(v.len() as i64)),
                            (Value::Dict(v), "length") => Some(Value::Int(v.len() as i64)),

                            (Value::Builtins, "obj_props") => Some(Value::Dict({
                                let mut map = AHashMap::new();
                                for (n, k) in OBJECT_KEYS.iter() {
                                    map.insert(
                                        n.chars().collect_vec().into(),
                                        VisSource::Public(ValueRef::new(
                                            Value::ObjectKey(*k)
                                                .into_stored(Vm::make_area(opcode_span, &program)),
                                        )),
                                    );
                                }
                                map
                            })), // what he are doing????????????????

                            _ => None,
                        };

                        macro_rules! error {
                            ($type:ident) => {
                                return Err(RuntimeError::NonexistentMember {
                                    area: Vm::make_area(opcode_span, &program),
                                    member: key_str,
                                    base_type: $type,
                                    call_stack: self.get_call_stack(),
                                })
                            };
                        }

                        if let Some(v) = special {
                            mem::drop(value);

                            self.change_reg(
                                dest,
                                v.into_stored(Vm::make_area(opcode_span, &program)),
                            );
                        } else {
                            let base_type = value.value.get_type();

                            'out: {
                                match &value.value {
                                    Value::Dict(v) => {
                                        if let Some(v) = v.get(&key) {
                                            let v = v.value().clone();

                                            mem::drop(value);

                                            self.change_reg(dest, v);
                                            break 'out;
                                        }
                                    },
                                    Value::Instance { items, .. } => {
                                        if let Some(v) = items.get(&key) {
                                            if let VisSource::Private(v, src) = v {
                                                if src != &program.src {
                                                    return Err(
                                                        RuntimeError::PrivateMemberAccess {
                                                            area: Vm::make_area(
                                                                opcode_span,
                                                                &program,
                                                            ),
                                                            member: key.iter().collect(),
                                                            call_stack: self.get_call_stack(),
                                                        },
                                                    );
                                                }
                                            }

                                            let v = v.value().clone();

                                            mem::drop(value);

                                            self.change_reg(dest, v);
                                            break 'out;
                                        }
                                    },
                                    Value::Module { exports, .. } => {
                                        if let Some(v) = exports.get(&key) {
                                            let v = v.clone();

                                            mem::drop(value);

                                            self.change_reg(dest, v);
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
                                            area: Vm::make_area(opcode_span, &program),
                                            def_area: v.area.clone(),
                                            member_name: key.iter().collect(),
                                            member_type: v.value.get_type(),
                                            base_type,
                                            call_stack: self.get_call_stack(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::AssociatedMemberNotAMethod {
                                        area: Vm::make_area(opcode_span, &program),
                                        def_area: v.area.clone(),
                                        member_name: key.iter().collect(),
                                        member_type: v.value.get_type(),
                                        base_type,
                                        call_stack: self.get_call_stack(),
                                    });
                                }

                                mem::drop(value);

                                self.change_reg(dest, v);
                            }
                        }
                    },
                    Opcode::Associated { from, dest, member } => {
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
                                            area: Vm::make_area(opcode_span, &program),
                                            member: key.iter().collect(),
                                            base_type: *t,
                                            call_stack: self.get_call_stack(),
                                        })
                                    };
                                }
                                match self.impls.get(t) {
                                    Some(members) => match members.get(&key) {
                                        Some(elem) => {
                                            let v = elem.value().clone();

                                            mem::drop(value);

                                            self.change_reg(dest, v);
                                        },
                                        None => error!(),
                                    },
                                    None => error!(),
                                }
                            },
                            _ => {
                                return Err(RuntimeError::TypeMismatch {
                                    value_type: value.value.get_type(),
                                    value_area: value.area.clone(),
                                    area: Vm::make_area(opcode_span, &program),
                                    expected: &[ValueType::Type],
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        }
                    },
                    Opcode::TypeMember { from, dest, member } => todo!(),
                    Opcode::TypeOf { src, dest } => {
                        let t = self.get_reg_ref(src).borrow().value.get_type();
                        self.change_reg(
                            dest,
                            Value::Type(t).into_stored(Vm::make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::Len { src, dest } => {
                        let len = match &self.get_reg_ref(src).borrow().value {
                            Value::Array(v) => v.len(),
                            Value::Dict(v) => v.len(),
                            Value::String(v) => v.len(),
                            v => {
                                unreachable!()
                            },
                        };

                        self.change_reg(
                            dest,
                            Value::Int(len as i64)
                                .into_stored(Vm::make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::MismatchThrowIfFalse {
                        check_reg,
                        value_reg,
                    } => {},
                    Opcode::PushTryCatch { reg, to } => {
                        let mut ctx = self.context_stack.current_mut();
                        ctx.stack
                            .last_mut()
                            .unwrap()
                            .try_catches
                            .push(TryCatch { jump_pos: to, reg });
                    },
                    Opcode::PopTryCatch => {
                        self.context_stack
                            .current_mut()
                            .stack
                            .last_mut()
                            .unwrap()
                            .try_catches
                            .pop();
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

                        self.change_reg(
                            dest,
                            value.into_stored(Vm::make_area(opcode_span, &program)),
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
                        // println!("spumcock {}", self.context_stack.current().unique_id);
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

                        let base = self.get_reg_ref(base).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        handle_ctx_result! {
                            self.call_value(
                                top,
                                base,
                                &call.positional,
                                &call.named,
                                Vm::make_area(opcode_span, &program),
                            ),
                            (ret) => {
                                if let Some(dest) = call.dest {
                                    self.change_reg_multi(dest, ret);
                                }
                            }
                        }

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
                                    //     area: Vm::make_area(opcode_span, &program),
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
                    Opcode::RunBuiltin { args, dest } => todo!(),
                    Opcode::MakeTriggerFunc { src, dest } => {
                        let group = match &self.get_reg_ref(src).borrow().value {
                            Value::Group(g) => *g,
                            _ => unreachable!(),
                        };

                        self.change_reg(
                            dest,
                            Value::TriggerFunction {
                                group,
                                prev_context: self.context_stack.current().group,
                            }
                            .into_stored(Vm::make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::CallTriggerFunc { func } => {
                        let v = self.get_reg_ref(func).borrow();
                        let target = match &v.value {
                            Value::TriggerFunction { group, .. } => *group,
                            Value::Group(g) => *g,
                            _ => {
                                return Err(RuntimeError::TypeMismatch {
                                    value_type: v.value.get_type(),
                                    value_area: v.area.clone(),
                                    area: Vm::make_area(opcode_span, &program),
                                    expected: &[ValueType::TriggerFunction], // OR group idk how to make this error do that
                                    call_stack: self.get_call_stack(),
                                });
                            },
                        };
                        mem::drop(v);
                        let trigger =
                            make_spawn_trigger(self.context_stack.current().group, target, self);
                        self.triggers.push(trigger);
                    },
                    Opcode::SetContextGroup { reg } => {
                        let group = match &self.get_reg_ref(reg).borrow().value {
                            Value::Group(g) => *g,
                            Value::TriggerFunction { prev_context, .. } => *prev_context,
                            _ => unreachable!(),
                        };
                        self.context_stack.current_mut().group = group;
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
                    self.set_error(err);
                    continue;
                },
            }

            {
                let mut current = self.context_stack.current_mut();
                current.ip += 1;
            };

            // self.try_merge_contexts();
        }

        self.context_stack.pop().unwrap();
        out_contexts
            .into_iter()
            .map(|mut v| {
                let ret = match v.returned.take() {
                    Some(v) => v,
                    None => ValueRef::new(Value::Empty.into_stored(CodeArea {
                        src: Rc::clone(&program.src),
                        span: CodeSpan::internal(),
                    })),
                };
                // v.frame_stack.last_mut().unwrap().stack.push(ret);
                v.ip = original_ip;

                (v, ret)
            })
            .collect()
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

    pub fn call_value(
        &mut self,
        ctx: Context,
        base: ValueRef,
        positional_args: &[ValueRef],
        named_args: &[(ImmutStr, ValueRef)],
        call_area: CodeArea,
    ) -> RuntimeCtxResult<Multi<ValueRef>> {
        let base = base.borrow();
        let base_area = base.area.clone();

        match &base.value {
            Value::Macro(data) | Value::Iterator(data) => {
                let (args, spread_arg): (Box<dyn Iterator<Item = Option<&ImmutStr>>>, _) =
                    match &data.target {
                        MacroTarget::Spwn { func, .. } => {
                            let f = func.get_func();
                            (
                                Box::new(f.args.iter().map(|v| v.value.as_ref())),
                                f.spread_arg,
                            )
                        },
                        MacroTarget::FullyRust {
                            args, spread_arg, ..
                        } => (Box::new(args.iter().map(Some)), *spread_arg),
                    };

                #[derive(Clone)]
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

                if data.is_method && data.self_arg.is_some() {
                    match fill.get_mut(next_arg_idx) {
                        Some(ArgFill::Single(v, _)) => {
                            *v = data.self_arg.clone();
                            next_arg_idx += 1;
                        },
                        _ => unreachable!(),
                    }
                }

                for arg in positional_args.iter() {
                    match fill.get_mut(next_arg_idx) {
                        Some(ArgFill::Single(opt, ..)) => {
                            *opt = Some(arg.clone());

                            next_arg_idx += 1;
                        },
                        Some(ArgFill::Spread(s)) => s.push(arg.clone()),
                        None => {
                            return Err((
                                ctx,
                                RuntimeError::TooManyArguments {
                                    call_area,
                                    macro_def_area: base_area,
                                    call_arg_amount: positional_args.len(),
                                    macro_arg_amount: fill.len(),
                                    call_stack: self.get_call_stack(),
                                },
                            ))
                        },
                    }
                }

                for (name, arg) in named_args.iter() {
                    let Some(idx) = arg_name_map.get(name) else {
                    return Err((ctx, RuntimeError::UnknownKeywordArgument {
                        name: name.to_string(),
                        macro_def_area: base_area,
                        call_area,
                        call_stack: self.get_call_stack(),
                    }));
                };
                    match &mut fill[*idx] {
                        ArgFill::Single(opt, ..) => {
                            *opt = Some(arg.clone());
                        },
                        ArgFill::Spread(_) => {
                            return Err((
                                ctx,
                                RuntimeError::UnknownKeywordArgument {
                                    name: name.to_string(),
                                    macro_def_area: base_area,
                                    call_area,
                                    call_stack: self.get_call_stack(),
                                },
                            ))
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

                        for (i, arg) in fill.iter().enumerate() {
                            if let ArgFill::Single(None, name) = arg {
                                return Err((
                                    ctx,
                                    RuntimeError::ArgumentNotSatisfied {
                                        call_area: call_area.clone(),
                                        macro_def_area: base_area,
                                        arg: if let Some(name) = name {
                                            Either::Left(name.to_string())
                                        } else {
                                            Either::Right(i)
                                        },
                                        call_stack: self.get_call_stack(),
                                    },
                                ));
                            }
                        }

                        mem::drop(base);

                        Ok(self.run_function(
                            ctx,
                            CallInfo {
                                func,
                                call_area: Some(call_area.clone()),
                                is_builtin,
                            },
                            Box::new(move |vm| {
                                // println!("babagaga1");
                                let arg_amount = fill.len();
                                // println!("babagaga2");
                                for (i, arg) in fill.into_iter().enumerate() {
                                    match arg {
                                        ArgFill::Single(Some(r), ..) => {
                                            vm.change_reg(Register(i as u8), r);
                                        },
                                        ArgFill::Spread(v) => {
                                            vm.change_reg(
                                                Register(i as u8),
                                                StoredValue {
                                                    value: Value::Array(v),
                                                    area: call_area.clone(),
                                                },
                                            );
                                        },
                                        ArgFill::Single(None, ..) => unreachable!(),
                                    }
                                }
                                // println!("babagaga3");

                                for (i, v) in captured.iter().enumerate() {
                                    vm.change_reg(Register((arg_amount + i) as u8), v.clone());
                                }
                                // println!("babagaga4");
                            }),
                            // ContextSplitMode::Allow,
                        ))
                    },
                    MacroTarget::FullyRust { fn_ptr, .. } => {
                        todo!()
                        // let fn_ptr = fn_ptr.clone();
                        // mem::drop(base);

                        // self.run_rust_instrs(
                        //     program,
                        //     &[
                        //         &|vm| {
                        //             fn_ptr.0.borrow_mut()(
                        //                 {
                        //                     let mut out = Vec::with_capacity(fill.len());
                        //                     for (i, arg) in fill.clone().into_iter().enumerate() {
                        //                         out.push(match arg {
                        //                             ArgFill::Single(Some(r), ..) => r,
                        //                             ArgFill::Spread(v) => {
                        //                                 ValueRef::new(StoredValue {
                        //                                     value: Value::Array(v),
                        //                                     area: area.clone(),
                        //                                 })
                        //                             },
                        //                             ArgFill::Single(None, name) => {
                        //                                 return Err(
                        //                                     RuntimeError::ArgumentNotSatisfied {
                        //                                         call_area: area.clone(),
                        //                                         macro_def_area: macro_area.clone(),
                        //                                         arg: if let Some(name) = name {
                        //                                             Either::Left(name.to_string())
                        //                                         } else {
                        //                                             Either::Right(i)
                        //                                         },
                        //                                         call_stack: vm.get_call_stack(),
                        //                                     },
                        //                                 )
                        //                             },
                        //                         })
                        //                     }
                        //                     out
                        //                 },
                        //                 vm,
                        //                 program,
                        //                 area.clone(),
                        //             )?;
                        //             Ok(LoopFlow::ContinueLoop)
                        //         },
                        //         &|vm| {
                        //             if let Some(r) = call_expr.dest {
                        //                 let v = vm
                        //                     .context_stack
                        //                     .current_mut()
                        //                     .pop_extra_stack()
                        //                     .unwrap();
                        //                 vm.set_reg(r, v);
                        //             }
                        //             Ok(LoopFlow::Normal)
                        //         },
                        //     ],
                        // )?;
                    },
                }
            },
            v => Err((
                ctx,
                RuntimeError::TypeMismatch {
                    value_type: v.get_type(),
                    value_area: base_area,
                    expected: &[ValueType::Macro, ValueType::Iterator],
                    area: call_area.clone(),
                    call_stack: self.get_call_stack(),
                },
            )),
        }
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

        let value = op(
            &self.get_reg_ref(left).borrow(),
            &self.get_reg_ref(right).borrow(),
            span,
            self,
            program,
        )?;

        self.change_reg(
            dest,
            StoredValue {
                value,
                area: Vm::make_area(span, program),
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
        let value = op(
            &self.get_reg_ref(left).borrow(),
            &self.get_reg_ref(right).borrow(),
            span,
            self,
            program,
        )?;

        self.get_reg_ref(left).borrow_mut().value = value;
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
        let value = op(&self.get_reg_ref(value).borrow(), span, self, program)?;

        self.change_reg(
            dest,
            StoredValue {
                value,
                area: Vm::make_area(span, program),
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
            (Value::Macro(data), ValueType::Iterator) => Value::Iterator(data.clone()),
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

    pub fn runtime_display(
        &mut self,
        ctx: Context,
        value: &ValueRef,
        area: &CodeArea,
    ) -> RuntimeCtxResult<Multi<String>> {
        Ok(match &value.borrow().value {
            Value::Int(v) => Multi::new_single(ctx, v.to_string()),
            Value::Float(v) => Multi::new_single(ctx, v.to_string()),
            Value::Bool(v) => Multi::new_single(ctx, v.to_string()),
            Value::String(v) => Multi::new_single(ctx, v.iter().collect()),
            Value::Array(arr) => {
                let mut ret = Multi::new_single(ctx, vec![]);

                for elem in arr {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem, area)?;

                        Ok(g.map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(new_elem);
                            (ctx, v)
                        }))
                    })?;
                }

                ret.map(|c, v| (c, format!("[{}]", v.iter().join(", "))))
            },
            Value::Dict(map) => {
                let mut ret = Multi::new_single(ctx, vec![]);

                for (key, elem) in map {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem.value(), area)?;

                        Ok(g.map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(format!("{}: {}", key.iter().collect::<String>(), new_elem));
                            (ctx, v)
                        }))
                    })?;
                }

                ret.map(|c, v| (c, format!("{{{}}}", v.iter().join(", "))))
            },
            Value::Group(_) => todo!(),
            Value::Channel(_) => todo!(),
            Value::Block(_) => todo!(),
            Value::Item(_) => todo!(),
            Value::Builtins => todo!(),
            Value::Range { start, end, step } => todo!(),
            Value::Maybe(_) => todo!(),
            Value::Empty => todo!(),
            Value::Macro(_) => todo!(),
            Value::Iterator(_) => todo!(),
            Value::Type(_) => todo!(),
            Value::Module { exports, types } => todo!(),
            Value::TriggerFunction {
                group,
                prev_context,
            } => todo!(),
            Value::Error(err) => Multi::new_single(
                ctx,
                format!(
                    "{} {{...}}",
                    RuntimeError::VARIANT_NAMES
                        [unsafe { mem::transmute::<_, u64>(mem::discriminant(err)) as usize }]
                ),
            ),
            Value::ObjectKey(_) => todo!(),
            Value::Epsilon => todo!(),
            Value::Chroma { r, g, b, a } => todo!(),
            Value::Instance { typ, items } => {
                if let Some(v) = self.get_impl(ValueType::Custom(*typ), "_display_") {
                    let ret = self
                        .call_value(ctx, v.value().clone(), &[value.clone()], &[], area.clone())
                        .map_err(|(c, e)| {
                            (
                                c,
                                RuntimeError::WhileCallingOverload {
                                    area: area.clone(),
                                    error: Box::new(e),
                                    builtin: "_display_",
                                },
                            )
                        })?;

                    return Ok(ret.map(|ctx, v| match &v.borrow().value {
                        Value::String(v) => (ctx, v.iter().collect()),
                        _ => todo!(),
                    }));
                }

                let t = ValueType::Custom(*typ).runtime_display(self);

                let mut ret = Multi::new_single(ctx, vec![]);

                for (key, elem) in items {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem.value(), area)?;

                        Ok(g.map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(format!("{}: {}", key.iter().collect::<String>(), new_elem));
                            (ctx, v)
                        }))
                    })?;
                }

                ret.map(|c, v| (c, format!("{}::{{{}}}", t, v.iter().join(", "))))
            },
        })
    }

    pub fn index_value(
        &mut self,
        ctx: Context,
        base: &ValueRef,
        index: &ValueRef,
        area: &CodeArea,
    ) -> RuntimeCtxResult<Multi<ValueRef>> {
        let base_ref = base.borrow();
        let index_ref = index.borrow();

        let index_wrap = |idx: i64, len: usize, typ: ValueType| {
            let index_calc = if idx >= 0 { idx } else { len as i64 + idx };

            if index_calc < 0 || index_calc >= len as i64 {
                return Err(RuntimeError::IndexOutOfBounds {
                    len,
                    index: idx,
                    area: area.clone(),
                    typ,
                    call_stack: self.get_call_stack(),
                });
            }

            Ok(index_calc as usize)
        };

        macro_rules! index_wrap_mapped {
            ($($t:tt)*) => {
                match index_wrap($($t)*) {
                    Ok(v) => v,
                    Err(err) => return Err((ctx, err)),
                }
            };
        }

        Ok(match (&base_ref.value, &index_ref.value) {
            (Value::Array(arr), Value::Int(idx)) => {
                let v = &arr[index_wrap_mapped!(*idx, arr.len(), ValueType::Array)];

                let v = v.clone();
                Multi::new_single(ctx, v)
            },
            (Value::String(s), Value::Int(index)) => {
                let idx = index_wrap_mapped!(*index, s.len(), ValueType::String);
                let c = s[idx];

                let v = Value::String(Rc::new([c])).into_stored(area.clone());
                Multi::new_single(ctx, ValueRef::new(v))
            },
            (Value::Dict(v), Value::String(s)) => match v.get(s) {
                Some(v) => {
                    let v = v.value();

                    let v = v.clone();
                    Multi::new_single(ctx, v)
                },
                None => {
                    return Err((
                        ctx,
                        RuntimeError::NonexistentMember {
                            area: area.clone(),
                            member: s.iter().collect(),
                            base_type: base_ref.value.get_type(),
                            call_stack: self.get_call_stack(),
                        },
                    ))
                },
            },
            (other, _) => {
                if let Some(v) = self.get_impl(other.get_type(), "_index_") {
                    let ret = self
                        .call_value(
                            ctx,
                            v.value().clone(),
                            &[base.clone(), index.clone()],
                            &[],
                            area.clone(),
                        )
                        .map_err(|(c, e)| {
                            (
                                c,
                                RuntimeError::WhileCallingOverload {
                                    area: area.clone(),
                                    error: Box::new(e),
                                    builtin: "_index_",
                                },
                            )
                        })?;

                    return Ok(ret);
                }
                return Err((
                    ctx,
                    RuntimeError::InvalidIndex {
                        base: (base_ref.value.get_type(), base_ref.area.clone()),
                        index: (index_ref.value.get_type(), index_ref.area.clone()),
                        area: area.clone(),
                        call_stack: self.get_call_stack(),
                    },
                ));
            },
        })
    }

    // pub fn runtime_display(&self, value: &ValueRef, with_ptr: bool) -> String {
    //     let mut out = if with_ptr {
    //         format!("{:?}: ", value.as_ptr()).dimmed().to_string()
    //     } else {
    //         "".to_string()
    //     };

    //     out += &match &value.borrow().value {
    //         Value::Int(n) => n.to_string(),
    //         Value::Float(n) => n.to_string(),
    //         Value::Bool(b) => b.to_string(),
    //         Value::String(s) => format!("{:?}", s.iter().collect::<String>()),
    //         Value::Array(arr) => format!(
    //             "[{}]",
    //             arr.iter()
    //                 .map(|k| self.runtime_display(k, with_ptr))
    //                 .join(", ")
    //         ),
    //         Value::Dict(d) => format!(
    //             "{{ {} }}",
    //             d.iter()
    //                 .map(|(s, v)| format!(
    //                     "{}: {}",
    //                     s.iter().collect::<String>(),
    //                     self.runtime_display(v.value(), with_ptr)
    //                 ))
    //                 .join(", ")
    //         ),
    //         Value::Group(id) => id.fmt("g"),
    //         Value::Channel(id) => id.fmt("c"),
    //         Value::Block(id) => id.fmt("b"),
    //         Value::Item(id) => id.fmt("i"),
    //         Value::Builtins => "$".to_string(),
    //         Value::Chroma { r, g, b, a } => format!("@chroma::rgb8({r}, {g}, {b}, {a})"),
    //         Value::Range { start, end, step } => {
    //             if *step == 1 {
    //                 format!("{start}..{end}")
    //             } else {
    //                 format!("{start}..{step}..{end}")
    //             }
    //         },
    //         Value::Maybe(o) => match o {
    //             Some(k) => format!("({})?", self.runtime_display(k, with_ptr)),
    //             None => "?".into(),
    //         },
    //         Value::Empty => "()".into(),

    //         Value::Macro(MacroData { defaults, .. }) => {
    //             format!("<{}-arg macro at {:?}>", defaults.len(), value.as_ptr())
    //         },
    //         Value::Iterator(_) => {
    //             format!("<iterator at {:?}>", value.as_ptr())
    //         },
    //         Value::TriggerFunction { .. } => "!{...}".to_string(),
    //         Value::Type(t) => t.runtime_display(self),
    //         // Value::Object(map, typ) => format!(
    //         //     "{} {{ {} }}",
    //         //     match typ {
    //         //         ObjectType::Object => "obj",
    //         //         ObjectType::Trigger => "trigger",
    //         //     },
    //         //     map.iter()
    //         //         .map(|(s, k)| format!("{s}: {k:?}"))
    //         //         .collect::<Vec<_>>()
    //         //         .join(", ")
    //         // ),
    //         Value::Epsilon => "$.epsilon()".to_string(),
    //         Value::Module { exports, types } => format!(
    //             "module {{ {}{} }}",
    //             exports
    //                 .iter()
    //                 .map(|(s, k)| format!(
    //                     "{}: {}",
    //                     s.iter().collect::<String>(),
    //                     self.runtime_display(k, with_ptr)
    //                 ))
    //                 .join(", "),
    //             if types.iter().any(|p| p.is_pub()) {
    //                 format!(
    //                     "; {}",
    //                     types
    //                         .iter()
    //                         .filter(|p| p.is_pub())
    //                         .map(|p| ValueType::Custom(*p.value()).runtime_display(self))
    //                         .join(", ")
    //                 )
    //             } else {
    //                 "".into()
    //             }
    //         ),

    //         // Value::Iterator(_) => "<iterator>".into(),
    //         // Value::ObjectKey(k) => format!("$.obj_props.{}", <ObjectKey as Into<&str>>::into(*k)),
    //         Value::Error(id) => {
    //             use delve::VariantNames;
    //             format!(
    //                 "{} {{...}}",
    //                 crate::interpreting::error::ErrorDiscriminants::VARIANT_NAMES[*id]
    //             )
    //         },

    //         Value::Instance { typ, items } => format!(
    //             "@{}::{{ {} }}",
    //             self.type_def_map[typ].name.iter().collect::<String>(),
    //             items
    //                 .iter()
    //                 .map(|(s, v)| format!(
    //                     "{}: {}",
    //                     s.iter().collect::<String>(),
    //                     self.runtime_display(v.value(), with_ptr)
    //                 ))
    //                 .join(", ")
    //         ),
    //         Value::ObjectKey(_) => todo!(),
    //         // todo: iterator, object
    //     };
    //     out
    // }
}

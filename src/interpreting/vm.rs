use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::{self};
use std::rc::Rc;

use ahash::AHashMap;
use colored::Colorize;
use derive_more::{Deref, DerefMut};
use itertools::{Either, Itertools};

use super::context::{CallInfo, CloneMap, Context, ContextStack, FullContext, StackItem};
use super::multi::Multi;
use super::value::{MacroTarget, StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::{
    Bytecode, CallExpr, Constant, Function, Mutability, OptRegister, Register,
};
use crate::compiling::opcodes::{CallExprID, ConstID, Opcode};
use crate::gd::gd_object::{make_spawn_trigger, TriggerObject, TriggerOrder};
use crate::gd::ids::{IDClass, Id};
use crate::gd::object_keys::{ObjectKey, OBJECT_KEYS};
use crate::interpreting::builtins;
use crate::interpreting::context::TryCatch;
use crate::interpreting::error::RuntimeError;
use crate::interpreting::value::{BuiltinClosure, MacroData};
use crate::parsing::ast::{Vis, VisSource, VisTrait};
use crate::parsing::operators::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spanned, SpwnSource, TypeDefMap};
use crate::util::{ImmutCloneStr32, ImmutStr, Str32, String32};

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

    //penis
    pub context_stack: ContextStack,

    pub triggers: Vec<TriggerObject>,
    pub trigger_order_count: TriggerOrder,

    pub id_counters: [u16; 4],

    pub impls: AHashMap<ValueType, AHashMap<ImmutCloneStr32, VisSource<ValueRef>>>,
    pub overloads: AHashMap<Operator, Vec<ValueRef>>,

    pub pattern_mismatch_id_count: usize,
}

impl Vm {
    pub fn new(type_def_map: TypeDefMap, bytecode_map: BytecodeMap) -> Self {
        Self {
            context_stack: ContextStack(vec![]),
            triggers: vec![],
            trigger_order_count: TriggerOrder::new(),
            type_def_map,
            bytecode_map,
            id_counters: Default::default(),
            impls: AHashMap::new(),
            overloads: AHashMap::new(),
            pattern_mismatch_id_count: 0,
        }
    }

    pub fn make_area(&self, span: CodeSpan, program: &Rc<Program>) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&program.src),
        }
    }

    pub fn insert_multi<T, F>(
        &mut self,
        v: Multi<RuntimeResult<T>>,
        mut f: F,
        out: &mut Vec<Context>,
    ) where
        F: FnMut(&mut Context, T),
    {
        for (mut ctx, v) in v {
            match v {
                Ok(v) => {
                    f(&mut ctx, v);
                    self.context_stack.last_mut().contexts.push(ctx)
                },
                Err(err) => {
                    self.handle_errored_ctx(ctx, err, out);

                    // ctx.stack.pop();
                    // ctx.ret_error(err);

                    // out.push(ctx);
                },
            }
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

    pub fn change_reg_multi<T: Into<ValueRef>>(
        &mut self,
        reg: OptRegister,
        v: Multi<RuntimeResult<T>>,
        out: &mut Vec<Context>,
    ) {
        // self.contexts.last_mut().unwrap().yeet_current().unwrap();

        self.insert_multi(
            v,
            |ctx, v| {
                ctx.stack.last_mut().unwrap().registers[*reg as usize] = v.into();
            },
            out,
        );
    }

    pub fn write_pointee(&mut self, reg: OptRegister, v: StoredValue) {
        let mut binding = self.context_stack.current_mut();
        let mut g = binding.stack.last_mut().unwrap().registers[reg.0 as usize].borrow_mut();
        *g = v;
    }

    pub fn write_pointee_multi(
        &mut self,
        reg: OptRegister,
        v: Multi<RuntimeResult<StoredValue>>,
        out: &mut Vec<Context>,
    ) {
        self.insert_multi(
            v,
            |ctx, v| {
                let mut g = ctx.stack.last_mut().unwrap().registers[reg.0 as usize].borrow_mut();
                *g = v;
            },
            out,
        );
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
            .and_then(|v| v.get(String32::from_str(name).as_utfstr()))
    }
}

impl DeepClone<&StoredValue> for Vm {
    fn deep_clone_map(&self, input: &StoredValue, map: &mut Option<&mut CloneMap>) -> StoredValue {
        let area = input.area.clone();

        let mut deep_clone_dict_items = |v: &AHashMap<ImmutCloneStr32, VisSource<ValueRef>>| {
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
                    MacroTarget::FullyRust { fn_ptr, .. } => {
                        let new = fn_ptr.borrow().shallow_clone();
                        new.borrow_mut().deep_clone_inner_refs(self, map);
                        *fn_ptr = new;
                    },
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
    ) -> Multi<RuntimeResult<ValueRef>> {
        let program = call_info.func.program.clone();
        let func = call_info.func.func;
        let is_builtin = call_info.is_builtin;

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

            if self.context_stack.len() > RECURSION_LIMIT {
                // return Err(RuntimeError::RecursionLimit {
                //     area: self.make_area(opcode_span, &program),
                //     call_stack: self.get_call_stack(),
                // });
                todo!()
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

                macro_rules! bin_op {
                    ($op:ident, $a:ident, $b:ident, $to:ident) => {{
                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        self.bin_op(
                            BinOp::$op,
                            &program,
                            $a,
                            $b,
                            $to,
                            top,
                            &mut out_contexts,
                            opcode_span,
                        );
                        return Ok(LoopFlow::ContinueLoop);
                    }};
                }
                macro_rules! assign_op {
                    ($op:ident, $a:ident, $b:ident, $to:ident) => {{
                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let ret = self.bin_op(BinOp::$op, &program, $a, $b, top, opcode_span);

                        self.change_reg_multi($to, ret, &mut out_contexts);
                        return Ok(LoopFlow::ContinueLoop);
                    }};
                }

                match opcode {
                    Opcode::LoadConst { id, to } => {
                        let value = Value::from_const(self, program.get_constant(id));
                        self.change_reg(
                            to,
                            value.into_stored(self.make_area(opcode_span, &program)),
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
                        bin_op!(Plus, a, b, to);
                    },
                    Opcode::Minus { a, b, to } => {
                        bin_op!(Minus, a, b, to);
                    },
                    Opcode::Mult { a, b, to } => {
                        bin_op!(Mult, a, b, to);
                    },
                    Opcode::Div { a, b, to } => {
                        bin_op!(Div, a, b, to);
                    },
                    Opcode::Mod { a, b, to } => {
                        bin_op!(Mod, a, b, to);
                    },
                    Opcode::Pow { a, b, to } => {
                        bin_op!(Pow, a, b, to);
                    },
                    Opcode::Eq { a, b, to } => {
                        bin_op!(Eq, a, b, to);
                    },
                    Opcode::Neq { a, b, to } => {
                        bin_op!(Neq, a, b, to);
                    },
                    Opcode::PureEq { a, b, to } => {
                        let v = Value::Bool(value_ops::equality(
                            &self.get_reg_ref(a).borrow().value,
                            &self.get_reg_ref(b).borrow().value,
                            self,
                        ));
                        self.change_reg(to, v.into_stored(self.make_area(opcode_span, &program)));
                    },
                    Opcode::PureNeq { a, b, to } => {
                        let v = Value::Bool(!value_ops::equality(
                            &self.get_reg_ref(a).borrow().value,
                            &self.get_reg_ref(b).borrow().value,
                            self,
                        ));
                        self.change_reg(to, v.into_stored(self.make_area(opcode_span, &program)));
                    },
                    Opcode::Gt { a, b, to } => {
                        bin_op!(Gt, a, b, to);
                    },
                    Opcode::Gte { a, b, to } => {
                        bin_op!(Gte, a, b, to);
                    },
                    Opcode::Lt { a, b, to } => {
                        bin_op!(Lt, a, b, to);
                    },
                    Opcode::Lte { a, b, to } => {
                        bin_op!(Lte, a, b, to);
                    },
                    Opcode::BinOr { a, b, to } => {
                        bin_op!(BinOr, a, b, to);
                    },
                    Opcode::BinAnd { a, b, to } => {
                        bin_op!(BinAnd, a, b, to);
                    },
                    Opcode::Range { a, b, to } => {
                        bin_op!(Range, a, b, to);
                    },
                    Opcode::In { a, b, to } => {
                        bin_op!(In, a, b, to);
                    },
                    Opcode::ShiftLeft { a, b, to } => {
                        bin_op!(ShiftLeft, a, b, to);
                    },
                    Opcode::ShiftRight { a, b, to } => {
                        bin_op!(ShiftRight, a, b, to);
                    },
                    Opcode::As { a, b, to } => {
                        let a = self.get_reg_ref(a).clone();
                        let b = self.get_reg_ref(b).borrow();

                        if let &Value::Type(typ) = &b.value {
                            mem::drop(b);

                            let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                            top.ip += 1;

                            let out = self.convert_type(top, &a, typ, opcode_span, &program);

                            mem::drop(a);

                            self.change_reg_multi(
                                to,
                                out.try_map(|ctx, v| {
                                    (
                                        ctx,
                                        Ok(v.into_stored(self.make_area(opcode_span, &program))),
                                    )
                                }),
                                &mut out_contexts,
                            );
                            return Ok(LoopFlow::ContinueLoop);
                        } else {
                            return Err(RuntimeError::TypeMismatch {
                                value_type: b.value.get_type(),
                                value_area: b.area.clone(),
                                area: self.make_area(opcode_span, &program),
                                expected: &[ValueType::Type],
                                call_stack: self.get_call_stack(),
                            });
                        }
                    },
                    Opcode::PlusEq { a, b } => {
                        self.assign_op(AssignOp::PlusEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::MinusEq { a, b } => {
                        self.assign_op(AssignOp::MinusEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::MultEq { a, b } => {
                        self.assign_op(AssignOp::MultEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::DivEq { a, b } => {
                        self.assign_op(AssignOp::DivEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::PowEq { a, b } => {
                        self.assign_op(AssignOp::PowEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::ModEq { a, b } => {
                        self.assign_op(AssignOp::ModEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::BinAndEq { a, b } => {
                        self.assign_op(AssignOp::BinAndEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::BinOrEq { a, b } => {
                        self.assign_op(AssignOp::BinOrEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::ShiftLeftEq { a, b } => {
                        self.assign_op(AssignOp::ShiftLeftEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::ShiftRightEq { a, b } => {
                        self.assign_op(AssignOp::ShiftRightEq, &program, a, b, opcode_span)?;
                    },
                    Opcode::Not { v, to } => {
                        self.unary_op(UnaryOp::ExclMark, &program, v, to, opcode_span)?;
                    },
                    Opcode::Negate { v, to } => {
                        self.unary_op(UnaryOp::Minus, &program, v, to, opcode_span)?;
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
                                    let val = k.clone();

                                    mem::drop(vref);

                                    self.change_reg(check, val);
                                },
                                None => {
                                    mem::drop(vref);
                                    self.context_stack.current_mut().ip = *to as usize;
                                    return Ok(LoopFlow::ContinueLoop);
                                },
                            },
                            other => {
                                return Err(RuntimeError::TypeMismatch {
                                    value_type: other.get_type(),
                                    value_area: vref.area.clone(),
                                    expected: &[ValueType::Maybe],
                                    area: self.make_area(opcode_span, &program),
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        };
                    },
                    Opcode::IntoIterator { src, dest } => {
                        let src = self.get_reg_ref(src).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let area = self.make_area(opcode_span, &program);

                        let s = self.value_to_iterator(top, &src, &area, &program);

                        self.change_reg_multi(
                            dest,
                            s.try_map(|ctx, v| {
                                (
                                    ctx,
                                    Ok(ValueRef::new(Value::Iterator(v).into_stored(area.clone()))),
                                )
                            }),
                            &mut out_contexts,
                        );

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::IterNext { src, dest } => {
                        let src = self.get_reg_ref(src).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let out = self.call_value(
                            top,
                            src,
                            &[],
                            &[],
                            self.make_area(opcode_span, &program),
                            &program,
                        );

                        self.change_reg_multi(dest, out, &mut out_contexts);

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::AllocArray { dest, len } => self.change_reg(
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
                    Opcode::AllocDict { dest, capacity } => self.change_reg(
                        dest,
                        StoredValue {
                            value: Value::Dict(AHashMap::with_capacity(capacity as usize)),
                            area: self.make_area(opcode_span, &program),
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
                                    area: self.make_area(opcode_span, &program),
                                    typ: *t,
                                    call_stack: self.get_call_stack(),
                                })
                            },
                            v => {
                                return Err(RuntimeError::TypeMismatch {
                                    value_type: v.get_type(),
                                    value_area: base.area.clone(),
                                    expected: &[ValueType::Type],
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
                        self.change_reg(
                            dest,
                            Value::Instance { typ: t, items }
                                .into_stored(self.make_area(opcode_span, &program)),
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
                        Value::Empty.into_stored(self.make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadNone { to } => self.change_reg(
                        to,
                        Value::Maybe(None).into_stored(self.make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadBuiltins { to } => self.change_reg(
                        to,
                        Value::Builtins.into_stored(self.make_area(opcode_span, &program)),
                    ),
                    Opcode::LoadEpsilon { to } => self.change_reg(
                        to,
                        Value::Epsilon.into_stored(self.make_area(opcode_span, &program)),
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
                                area: self.make_area(opcode_span, &program),
                            },
                        )
                    },
                    Opcode::ApplyStringFlag { flag, reg } => todo!(),
                    Opcode::WrapMaybe { from, to } => {
                        let v = self.deep_clone_ref(from);
                        self.change_reg(
                            to,
                            Value::Maybe(Some(v))
                                .into_stored(self.make_area(opcode_span, &program)),
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

                        top.returned = Ok(Some(ret_val));
                        out_contexts.push(top);

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Dbg { reg, .. } => {
                        let r = self.get_reg_ref(reg).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let s = self.runtime_display(
                            top,
                            &r,
                            &self.make_area(opcode_span, &program),
                            &program,
                        );

                        self.insert_multi(
                            s,
                            |ctx, v| {
                                println!(
                                    "{} {} {} {} {}",
                                    v,
                                    "::".dimmed(),
                                    ctx.unique_id.to_string().bright_blue(),
                                    ctx.group.fmt("g").green(),
                                    format!("{:?}", r.as_ptr()).dimmed(),
                                    // ctx.stack.len(),
                                );
                            },
                            &mut out_contexts,
                        );

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Throw { reg } => {
                        return Err(RuntimeError::ThrownError {
                            area: self.make_area(opcode_span, &program),
                            value: self.get_reg_ref(reg).clone(),
                            call_stack: self.get_call_stack(),
                        });
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

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;
                        // also i need to do mergig
                        let ret = self.run_function(
                            top,
                            CallInfo {
                                func: coord,
                                call_area: None,
                                is_builtin: None,
                            },
                            Box::new(|_| {}),
                        );
                        self.change_reg_multi(dest, ret, &mut out_contexts);
                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::ToString { from, dest } => {
                        let r = self.get_reg_ref(from).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let area = self.make_area(opcode_span, &program);

                        let s = self.runtime_display(top, &r, &area, &program);

                        self.change_reg_multi(
                            dest,
                            s.try_map(|ctx, v| {
                                (
                                    ctx,
                                    Ok(ValueRef::new(
                                        Value::String(String32::from(v).into())
                                            .into_stored(area.clone()),
                                    )),
                                )
                            }),
                            &mut out_contexts,
                        );

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Index { base, dest, index } => {
                        let base = self.get_reg_ref(base).clone();
                        let index = self.get_reg_ref(index).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let ret = self.index_value(
                            top,
                            &base,
                            &index,
                            &self.make_area(opcode_span, &program),
                            &program,
                        );

                        self.change_reg_multi(dest, ret, &mut out_contexts);

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    o @ (Opcode::MemberImmut { from, dest, member }
                    | Opcode::MemberMut { from, dest, member }) => {
                        let base_mut = matches!(o, Opcode::MemberMut { .. });

                        let key = match &self.get_reg_ref(member).borrow().value {
                            Value::String(s) => s.clone(),
                            _ => unreachable!(),
                        };
                        let key_str: String = key.as_ref().into();

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
                                        String32::from_str(n).into(),
                                        VisSource::Public(ValueRef::new(
                                            Value::ObjectKey(*k)
                                                .into_stored(self.make_area(opcode_span, &program)),
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
                                    area: self.make_area(opcode_span, &program),
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
                                v.into_stored(self.make_area(opcode_span, &program)),
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
                                                            area: self
                                                                .make_area(opcode_span, &program),
                                                            member: key_str,
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
                                    target,
                                    ..
                                }) = &mut v.value
                                {
                                    if *is_method {
                                        if let MacroTarget::Spwn { func, .. } = target {
                                            let needs_mut = func.get_func().args[0].value.1;

                                            if needs_mut && !base_mut {
                                                return Err(RuntimeError::ArgumentNotMutable {
                                                    call_area: self
                                                        .make_area(opcode_span, &program),
                                                    macro_def_area: v.area.clone(),
                                                    arg: Either::Left("self".into()),
                                                    call_stack: self.get_call_stack(),
                                                });
                                            }
                                        }

                                        *self_arg = Some(self.get_reg_ref(from).clone())
                                    } else {
                                        return Err(RuntimeError::AssociatedMemberNotAMethod {
                                            area: self.make_area(opcode_span, &program),
                                            def_area: v.area.clone(),
                                            member_name: key_str,
                                            member_type: v.value.get_type(),
                                            base_type,
                                            call_stack: self.get_call_stack(),
                                        });
                                    }
                                } else {
                                    return Err(RuntimeError::AssociatedMemberNotAMethod {
                                        area: self.make_area(opcode_span, &program),
                                        def_area: v.area.clone(),
                                        member_name: key_str,
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
                                            area: self.make_area(opcode_span, &program),
                                            member: key.as_ref().into(),
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
                                    area: self.make_area(opcode_span, &program),
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
                            Value::Type(t).into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::Len { src, dest } => {
                        let len = match &self.get_reg_ref(src).borrow().value {
                            Value::Array(v) => v.len(),
                            Value::Dict(v) => v.len(),
                            Value::String(v) => v.len(),
                            _ => {
                                unreachable!()
                            },
                        };

                        self.change_reg(
                            dest,
                            Value::Int(len as i64)
                                .into_stored(self.make_area(opcode_span, &program)),
                        )
                    },
                    Opcode::MismatchThrowIfFalse {
                        check_reg,
                        value_reg,
                    } => {
                        let id = self.pattern_mismatch_id_count;

                        let check = self.get_reg_ref(check_reg).borrow();
                        let matches = match &check.value {
                            Value::Bool(b) => *b,
                            _ => unreachable!(),
                        };

                        if !matches {
                            let pattern_area = check.area.clone();

                            mem::drop(check);
                            self.pattern_mismatch_id_count += 1;

                            let v = self.get_reg_ref(value_reg).borrow();
                            let v = (v.value.get_type(), v.area.clone());

                            return Err(RuntimeError::PatternMismatch {
                                v,
                                pattern_area,
                                call_stack: self.get_call_stack(),
                                id,
                            });
                        }
                    },
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

                        let defaults = vec![None; f.args.len()];

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
                        // println!("spumcock {}", self.context_stack.current().unique_id);
                        let call = program.get_call_expr(call);
                        let call = CallExpr {
                            dest: call.dest,
                            positional: call
                                .positional
                                .iter()
                                .map(|(r, m)| (self.get_reg_ref(*r).clone(), *m))
                                .collect_vec()
                                .into(),
                            named: call
                                .named
                                .iter()
                                .map(|(s, r, m)| (s.clone(), self.get_reg_ref(*r).clone(), *m))
                                .collect_vec()
                                .into(),
                        };

                        let base = self.get_reg_ref(base).clone();

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        let ret = self.call_value(
                            top,
                            base,
                            &call.positional,
                            &call.named,
                            self.make_area(opcode_span, &program),
                            &program,
                        );
                        if let Some(dest) = call.dest {
                            self.change_reg_multi(dest, ret, &mut out_contexts);
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
                                let s: String = k.as_ref().into();
                                if let Some(f) = t.get_override_fn(&s) {
                                    *is_builtin = Some(f)
                                }
                            }
                        }

                        self.impls.entry(t).or_insert(AHashMap::new()).extend(map);
                    },
                    Opcode::RunBuiltin { args, dest } => {
                        if let Some(f) = is_builtin {
                            let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                            top.ip += 1;

                            let ret = f.0(
                                (0..args)
                                    .map(|r| {
                                        top.stack.last_mut().unwrap().registers[r as usize].clone()
                                    })
                                    .collect_vec(),
                                top,
                                self,
                                &program,
                                self.make_area(opcode_span, &program),
                            );

                            self.change_reg_multi(dest, ret, &mut out_contexts);

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

                        self.change_reg(
                            dest,
                            Value::TriggerFunction {
                                group,
                                prev_context: self.context_stack.current().group,
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
                                    value_type: v.value.get_type(),
                                    value_area: v.area.clone(),
                                    area: self.make_area(opcode_span, &program),
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
                    Opcode::AddOperatorOverload { from, op } => {
                        let v = self.get_reg_ref(from).clone();
                        self.overloads.entry(op).or_insert(vec![]).push(v);
                    },
                    Opcode::IncMismatchIdCount => {
                        self.pattern_mismatch_id_count += 1;
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
                    let ctx = self.context_stack.last_mut().yeet_current().unwrap();
                    self.handle_errored_ctx(ctx, err, &mut out_contexts);
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
            .map(|mut ctx| {
                let ret = mem::replace(&mut ctx.returned, Ok(None)).map(|mut v| match v.take() {
                    Some(v) => v,
                    None => ValueRef::new(Value::Empty.into_stored(CodeArea {
                        src: Rc::clone(&program.src),
                        span: CodeSpan::internal(),
                    })),
                });

                ctx.ip = original_ip;

                (ctx, ret)
            })
            .collect()
    }

    fn handle_errored_ctx(
        &mut self,
        mut ctx: Context,
        err: RuntimeError,
        out_contexts: &mut Vec<Context>,
    ) {
        if let Some(t) = ctx.stack.last_mut().unwrap().try_catches.pop() {
            let val = match err {
                RuntimeError::ThrownError { value, .. } => value,
                _ => Value::Error(unsafe {
                    mem::transmute::<_, u64>(mem::discriminant(&err)) as usize
                })
                .into_stored(err.get_main_area().clone())
                .into(),
            };
            ctx.stack.last_mut().unwrap().registers[*t.reg as usize] = val;
            ctx.ip = *t.jump_pos as usize;
            self.context_stack.last_mut().contexts.push(ctx);
        } else {
            ctx.stack.pop();
            ctx.ret_error(err);
            out_contexts.push(ctx);
        }
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
        positional_args: &[(ValueRef, Mutability)],
        named_args: &[(ImmutStr, ValueRef, Mutability)],
        call_area: CodeArea,
        program: &Rc<Program>,
    ) -> Multi<RuntimeResult<ValueRef>> {
        let base = base.borrow();
        let base_area = base.area.clone();

        match &base.value {
            Value::Macro(data) | Value::Iterator(data) => {
                let (args, spread_arg): (
                    Box<dyn Iterator<Item = (Option<&ImmutStr>, Mutability)>>,
                    _,
                ) = match &data.target {
                    MacroTarget::Spwn { func, .. } => {
                        let f = func.get_func();
                        (
                            Box::new(f.args.iter().map(|v| (v.value.0.as_ref(), v.value.1))),
                            f.spread_arg,
                        )
                    },
                    MacroTarget::FullyRust {
                        args, spread_arg, ..
                    } => (Box::new(args.iter().map(|v| (Some(v), false))), *spread_arg),
                };

                #[derive(Clone)]
                enum ArgFill {
                    Single(Option<ValueRef>, Option<ImmutStr>, bool),
                    Spread(Vec<ValueRef>, bool),
                }

                let mut arg_name_map = AHashMap::new();

                let mut fill = args
                    .enumerate()
                    .map(|(i, (arg, m))| {
                        if let Some(name) = &arg {
                            arg_name_map.insert(&**name, i);
                        }

                        if spread_arg == Some(i as u8) {
                            ArgFill::Spread(vec![], m)
                        } else {
                            ArgFill::Single(None, arg.cloned(), m)
                        }
                    })
                    .collect_vec();

                let mut next_arg_idx = 0;

                if data.is_method && data.self_arg.is_some() {
                    match fill.get_mut(next_arg_idx) {
                        Some(ArgFill::Single(v, ..)) => {
                            *v = data.self_arg.clone();
                            next_arg_idx += 1;
                        },
                        _ => unreachable!(),
                    }
                }

                for (idx, (arg, is_mutable)) in positional_args.iter().enumerate() {
                    match fill.get_mut(next_arg_idx) {
                        Some(ArgFill::Single(opt, n, needs_mutable)) => {
                            if *needs_mutable && !is_mutable {
                                return Multi::new_single(
                                    ctx,
                                    Err(RuntimeError::ArgumentNotMutable {
                                        call_area,
                                        macro_def_area: base_area,
                                        arg: match n {
                                            Some(name) => Either::Left(name.to_string()),
                                            None => Either::Right(idx),
                                        },
                                        call_stack: self.get_call_stack(),
                                    }),
                                );
                            }

                            *opt = Some(arg.clone());
                            next_arg_idx += 1;
                        },
                        Some(ArgFill::Spread(s, needs_mutable)) => {
                            if *needs_mutable && !is_mutable {
                                return Multi::new_single(
                                    ctx,
                                    Err(RuntimeError::ArgumentNotMutable {
                                        call_area,
                                        macro_def_area: base_area,
                                        arg: Either::Right(idx),
                                        call_stack: self.get_call_stack(),
                                    }),
                                );
                            }
                            s.push(arg.clone())
                        },
                        None => {
                            return Multi::new_single(
                                ctx,
                                Err(RuntimeError::TooManyArguments {
                                    call_area,
                                    macro_def_area: base_area,
                                    call_arg_amount: positional_args.len(),
                                    macro_arg_amount: fill.len(),
                                    call_stack: self.get_call_stack(),
                                }),
                            )
                        },
                    }
                }

                for (name, arg, is_mutable) in named_args.iter() {
                    let Some(idx) = arg_name_map.get(name) else {
                        return Multi::new_single(
                            ctx,
                            Err(RuntimeError::UnknownKeywordArgument {
                                name: name.to_string(),
                                macro_def_area: base_area,
                                call_area,
                                call_stack: self.get_call_stack(),
                            }),
                        )
                    };
                    match &mut fill[*idx] {
                        ArgFill::Single(opt, _, needs_mutable) => {
                            if *needs_mutable && !is_mutable {
                                return Multi::new_single(
                                    ctx,
                                    Err(RuntimeError::ArgumentNotMutable {
                                        call_area,
                                        macro_def_area: base_area,
                                        arg: Either::Left(name.to_string()),
                                        call_stack: self.get_call_stack(),
                                    }),
                                );
                            }
                            *opt = Some(arg.clone());
                        },
                        ArgFill::Spread(..) => {
                            return Multi::new_single(
                                ctx,
                                Err(RuntimeError::UnknownKeywordArgument {
                                    name: name.to_string(),
                                    macro_def_area: base_area,
                                    call_area,
                                    call_stack: self.get_call_stack(),
                                }),
                            )
                        },
                    }
                }

                for (i, arg) in fill.iter().enumerate() {
                    if let ArgFill::Single(None, name, ..) = arg {
                        return Multi::new_single(
                            ctx,
                            Err(RuntimeError::ArgumentNotSatisfied {
                                call_area: call_area.clone(),
                                macro_def_area: base_area,
                                arg: if let Some(name) = name {
                                    Either::Left(name.to_string())
                                } else {
                                    Either::Right(i)
                                },
                                call_stack: self.get_call_stack(),
                            }),
                        );
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

                        self.run_function(
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
                                        ArgFill::Spread(v, ..) => {
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
                        )
                    },
                    MacroTarget::FullyRust { fn_ptr, .. } => {
                        let fn_ptr = fn_ptr.clone();
                        mem::drop(base);

                        let x = fn_ptr.borrow_mut()(
                            {
                                let mut out = Vec::with_capacity(fill.len());
                                for (_, arg) in fill.clone().into_iter().enumerate() {
                                    out.push(match arg {
                                        ArgFill::Single(Some(r), ..) => r,
                                        ArgFill::Spread(v, ..) => ValueRef::new(StoredValue {
                                            value: Value::Array(v),
                                            area: call_area.clone(),
                                        }),
                                        ArgFill::Single(None, ..) => unreachable!(),
                                    })
                                }
                                out
                            },
                            ctx,
                            self,
                            program,
                            call_area.clone(),
                        );
                        x
                    },
                }
            },
            v => Multi::new_single(
                ctx,
                Err(RuntimeError::TypeMismatch {
                    value_type: v.get_type(),
                    value_area: base_area,
                    expected: &[ValueType::Macro, ValueType::Iterator],
                    area: call_area.clone(),
                    call_stack: self.get_call_stack(),
                }),
            ),
        }
    }

    #[inline]
    pub fn bin_op(
        &mut self,
        op: BinOp,
        program: &Rc<Program>,
        left: OptRegister,
        right: OptRegister,
        to: OptRegister,
        ctx: Context,
        out_contexts: &mut Vec<Context>,
        span: CodeSpan,
    ) {
        // TODO: overloads // no longer todo you absolute fucking fuckwit, you fucking dumbass, you are so fucking dumb holy fucking shit, i wish i was an egirl

        let left_ref = ctx.stack.last().unwrap().registers[*left as usize].clone();
        let right_ref = ctx.stack.last().unwrap().registers[*right as usize].clone();

        let mut out = Multi::new_single(ctx, Ok(None));

        if let Some(v) = self.overloads.get(&Operator::Bin(op)).cloned() {
            for base in v {
                out = out.flat_map(|ctx, v| {
                    if v.as_ref().is_ok_and(|v| v.is_none()) {
                        let check_id = self.pattern_mismatch_id_count;
                        self.call_value(
                            ctx,
                            base.clone(),
                            &[(left_ref.clone(), false), (right_ref.clone(), false)],
                            &[],
                            self.make_area(span, program),
                            program,
                        )
                        .map(|ctx, v| match v {
                            Ok(v) => (ctx, Ok(Some(v))),
                            Err(err) => {
                                if let RuntimeError::PatternMismatch { id, .. } = err {
                                    if id == check_id {
                                        return (ctx, Ok(None));
                                    }
                                    println!("GIG: {id} {check_id}");
                                }
                                (ctx, Err(err))
                            },
                        })
                    } else {
                        Multi::new_single(ctx, v)
                    }
                });
            }
        }

        let out = out.map(|ctx, v| {
            let v = match v {
                Ok(Some(v)) => Ok(v),
                Err(err) => Err(err),
                Ok(None) => op.get_fn()(
                    &ctx.stack.last().unwrap().registers[*left as usize].borrow(),
                    &ctx.stack.last().unwrap().registers[*right as usize].borrow(),
                    span,
                    self,
                    program,
                )
                .map(|v| ValueRef::new(v.into_stored(self.make_area(span, program)))),
            };
            (ctx, v)
        });

        self.change_reg_multi(to, out, out_contexts);
    }

    #[inline]
    fn assign_op(
        &mut self,
        op: AssignOp,
        // op: fn(&StoredValue, &StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>,
        program: &Rc<Program>,
        left: OptRegister,
        right: OptRegister,
        span: CodeSpan,
    ) -> Result<(), RuntimeError> {
        let value = op.get_fn()(
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
        op: UnaryOp,
        //op: fn(&StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>,
        program: &Rc<Program>,
        value: OptRegister,
        dest: OptRegister,
        span: CodeSpan,
    ) -> Result<(), RuntimeError> {
        let value = op.get_fn()(&self.get_reg_ref(value).borrow(), span, self, program)?;

        self.change_reg(
            dest,
            StoredValue {
                value,
                area: self.make_area(span, program),
            },
        );
        Ok(())
    }
}

impl Vm {
    pub fn convert_type(
        &mut self,
        ctx: Context,
        v: &ValueRef,
        b: ValueType,
        span: CodeSpan, // ✍️
        program: &Rc<Program>,
    ) -> Multi<RuntimeResult<Value>> {
        if v.borrow().value.get_type() == b {
            return Multi::new_single(ctx, Ok(v.borrow().value.clone()));
        }

        let v = match (&v.borrow().value, b) {
            (Value::Macro(data), ValueType::Iterator) => Value::Iterator(data.clone()),

            (Value::Int(i), ValueType::Group) => Value::Group(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Channel) => Value::Channel(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Block) => Value::Block(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Item) => Value::Item(Id::Specific(*i as u16)),
            (Value::Int(i), ValueType::Float) => Value::Float(*i as f64),
            (Value::Int(i), ValueType::Chroma) => {
                if *i > 0xffffff {
                    Value::Chroma {
                        a: (i & 0xFF) as u8,
                        b: ((i >> 8) & 0xFF) as u8,
                        g: ((i >> 16) & 0xFF) as u8,
                        r: ((i >> 24) & 0xFF) as u8,
                    }
                } else {
                    Value::Chroma {
                        a: 0,
                        b: (i & 0xFF) as u8,
                        g: ((i >> 8) & 0xFF) as u8,
                        r: ((i >> 16) & 0xFF) as u8,
                    }
                }
            },
            (Value::Int(i), ValueType::Error) => Value::Error(*i as usize),
            (Value::Float(i), ValueType::Int) => Value::Int(*i as i64),

            (_, ValueType::String) => {
                return self
                    .runtime_display(ctx, v, &self.make_area(span, program), program)
                    .try_map(|ctx, v| (ctx, Ok(Value::String(String32::from_str(&v).into()))))
            },
            (Value::Bool(b), ValueType::Int) => Value::Int(*b as i64),
            (Value::Bool(b), ValueType::Float) => Value::Float(*b as i64 as f64),

            (Value::Dict(map), ValueType::Array) => {
                let a: BTreeMap<_, _> = map.iter().collect();
                let area = self.make_area(span, program);
                Value::Array(
                    a.into_iter()
                        .map(|(k, v)| {
                            let s =
                                ValueRef::new(Value::String(k.clone()).into_stored(area.clone()));
                            let v = self.deep_clone_ref(v.value());
                            ValueRef::new(Value::Array(vec![s, v]).into_stored(area.clone()))
                        })
                        .collect(),
                )
            },

            (Value::Range { .. }, ValueType::Array) => todo!(),

            (Value::TriggerFunction { group, .. }, ValueType::Group) => Value::Group(*group),

            (Value::String(s), ValueType::Float) => match s.as_ref().to_string().parse() {
                Ok(v) => Value::Float(v),
                Err(_) => {
                    return Multi::new_single(
                        ctx,
                        Err(RuntimeError::InvalidStringForConversion {
                            area: self.make_area(span, program),
                            to: ValueType::Float,
                        }),
                    )
                },
            },
            (Value::String(s), ValueType::Int) => match s.as_ref().to_string().parse() {
                Ok(v) => Value::Int(v),
                Err(_) => {
                    return Multi::new_single(
                        ctx,
                        Err(RuntimeError::InvalidStringForConversion {
                            area: self.make_area(span, program),
                            to: ValueType::Int,
                        }),
                    )
                },
            },

            (Value::String(s), ValueType::Array) => Value::Array(
                s.chars()
                    .map(|c| {
                        ValueRef::new(
                            Value::String(Str32::from_char_slice(&[c]).into())
                                .into_stored(v.borrow().area.clone()),
                        )
                    })
                    .collect(),
            ),

            (v, ValueType::Type) => Value::Type(v.get_type()),

            _ => {
                return Multi::new_single(
                    ctx,
                    Err(RuntimeError::CannotConvert {
                        from_type: v.borrow().value.get_type(),
                        from_area: v.borrow().area.clone(),
                        to: b,
                    }),
                )
            },
        };

        Multi::new_single(ctx, Ok(v))
    }

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
                        (fn_ptr.as_ref() as *const _ as *const () as usize).hash(state);
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
        program: &Rc<Program>,
    ) -> Multi<RuntimeResult<String>> {
        let s = match &value.borrow().value {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::String(v) => v.to_string(),
            Value::Array(arr) => {
                let mut ret = Multi::new_single(ctx, Ok(vec![]));

                for elem in arr {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem, area, program);

                        g.try_map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(new_elem);
                            (ctx, Ok(v))
                        })
                    });
                }

                return ret.try_map(|c, v| (c, Ok(format!("[{}]", v.iter().join(", ")))));
            },
            Value::Dict(map) => {
                let mut ret = Multi::new_single(ctx, Ok(vec![]));

                for (key, elem) in map {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem.value(), area, program);

                        g.try_map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(format!("{}: {}", key, new_elem));
                            (ctx, Ok(v))
                        })
                    });
                }

                return ret.try_map(|c, v| (c, Ok(format!("{{{}}}", v.iter().join(", ")))));
            },
            Value::Group(id) => id.fmt("g"),
            Value::Channel(id) => id.fmt("c"),
            Value::Block(id) => id.fmt("b"),
            Value::Item(id) => id.fmt("i"),
            Value::Builtins => "$".into(),
            Value::Range { start, end, step } => {
                if *step == 1 {
                    format!("{start}..{end}")
                } else {
                    format!("{start}..{step}..{end}")
                }
            },
            Value::Maybe(None) => "?".into(),
            Value::Maybe(Some(v)) => {
                return self
                    .runtime_display(ctx, v, area, program)
                    .try_map(|ctx, v| (ctx, Ok(format!("({v})?"))))
            },
            Value::Empty => "()".into(),
            Value::Macro(MacroData { defaults, .. }) => {
                format!("<{}-arg macro at {:?}>", defaults.len(), value.as_ptr())
            },
            Value::Iterator(_) => format!("<iterator at {:?}>", value.as_ptr()),
            Value::Type(t) => t.runtime_display(self),
            Value::Module { exports, types } => {
                let types_str = if types.iter().any(|p| p.is_pub()) {
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
                };

                let mut ret = Multi::new_single(ctx, Ok(vec![]));

                for (key, elem) in exports {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem, area, program);

                        g.try_map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(format!("{}: {}", key, new_elem));
                            (ctx, Ok(v))
                        })
                    });
                }

                return ret.try_map(|c, v| {
                    (
                        c,
                        Ok(format!("module {{ {}{} }}", v.iter().join(", "), types_str)),
                    )
                });
            },
            Value::TriggerFunction { .. } => "!{...}".to_string(),
            Value::Error(id) => {
                use delve::VariantNames;
                format!(
                    "{} {{...}}",
                    crate::interpreting::error::ErrorDiscriminants::VARIANT_NAMES[*id]
                )
            },
            Value::ObjectKey(k) => format!("$.obj_props.{}", <ObjectKey as Into<&str>>::into(*k)),
            Value::Epsilon => "$.epsilon()".to_string(),
            Value::Chroma { r, g, b, a } => format!("@chroma::rgb8({r}, {g}, {b}, {a})"),
            Value::Instance { typ, items } => {
                if let Some(v) = self.get_impl(ValueType::Custom(*typ), "_display_") {
                    let ret = self.call_value(
                        ctx,
                        v.value().clone(),
                        &[(value.clone(), false)],
                        &[],
                        area.clone(),
                        program,
                    );

                    return ret.try_map(|ctx, v| match &v.borrow().value {
                        Value::String(v) => (ctx, Ok(v.as_ref().into())),
                        _ => todo!(),
                    });
                }

                let t = ValueType::Custom(*typ).runtime_display(self);

                let mut ret = Multi::new_single(ctx, Ok(vec![]));

                for (key, elem) in items {
                    ret = ret.try_flat_map(|ctx, v| {
                        let g = self.runtime_display(ctx, elem.value(), area, program);

                        g.try_map(|ctx, new_elem| {
                            let mut v = v.clone();
                            v.push(format!("{}: {}", key, new_elem));
                            (ctx, Ok(v))
                        })
                    });
                }

                return ret.try_map(|c, v| (c, Ok(format!("{}::{{{}}}", t, v.iter().join(", ")))));
            },
        };
        Multi::new_single(ctx, Ok(s))
    }

    pub fn index_value(
        &mut self,
        ctx: Context,
        base: &ValueRef,
        index: &ValueRef,
        area: &CodeArea,
        program: &Rc<Program>,
    ) -> Multi<RuntimeResult<ValueRef>> {
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

        match (&base_ref.value, &index_ref.value) {
            (Value::Array(arr), Value::Int(idx)) => Multi::new_single(
                ctx,
                index_wrap(*idx, arr.len(), ValueType::Array).map(|idx| {
                    let v = &arr[idx];

                    v.clone()
                }),
            ),
            (Value::String(s), Value::Int(index)) => Multi::new_single(
                ctx,
                index_wrap(*index, s.len(), ValueType::String).map(|idx| {
                    let c = s.as_char_slice()[idx];

                    let v = Value::String(Str32::from_char_slice(&[c]).into())
                        .into_stored(area.clone());
                    ValueRef::new(v)
                }),
            ),
            (Value::Dict(v), Value::String(s)) => Multi::new_single(
                ctx,
                match v.get(s) {
                    Some(v) => {
                        let v = v.value();

                        Ok(v.clone())
                    },
                    None => Err(RuntimeError::NonexistentMember {
                        area: area.clone(),
                        member: s.as_ref().into(),
                        base_type: base_ref.value.get_type(),
                        call_stack: self.get_call_stack(),
                    }),
                },
            ),
            (other, _) => {
                if let Some(v) = self.get_impl(other.get_type(), "_index_") {
                    let ret = self.call_value(
                        ctx,
                        v.value().clone(),
                        &[(base.clone(), true), (index.clone(), false)],
                        &[],
                        area.clone(),
                        program,
                    );

                    return ret;
                }
                Multi::new_single(
                    ctx,
                    Err(RuntimeError::InvalidIndex {
                        base: (base_ref.value.get_type(), base_ref.area.clone()),
                        index: (index_ref.value.get_type(), index_ref.area.clone()),
                        area: area.clone(),
                        call_stack: self.get_call_stack(),
                    }),
                )
            },
        }
    }

    pub fn value_to_iterator(
        &mut self,
        ctx: Context,
        value: &ValueRef,
        area: &CodeArea,
        program: &Rc<Program>,
    ) -> Multi<RuntimeResult<MacroData>> {
        let data = match &value.borrow().value {
            Value::Array(arr) => {
                let n = 0usize;

                let arr = arr.clone();

                builtins::raw_macro! {
                    let next = [ self;
                        arr: Vec<ValueRef> => self.arr.iter_mut(),
                        n: usize,
                    ] () {
                        let ret = if extra.n >= extra.arr.len() {
                            Value::Maybe(None)
                        } else {
                            let n = extra.arr[extra.n].clone();
                            Value::Maybe(Some(n))
                        };
                        extra.n += 1;

                        Multi::new_single(ctx, Ok(ValueRef::new(ret.into_stored(area))))
                    } ctx vm program area extra
                }
                MacroData {
                    target: MacroTarget::FullyRust {
                        fn_ptr: Rc::new(RefCell::new(next)),
                        args: Box::new([]),
                        spread_arg: None,
                    },

                    defaults: Box::new([]),
                    self_arg: None,

                    is_method: false,
                }
            },
            Value::Dict(map) => {
                let n = 0usize;

                let arr = map
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect_vec();

                builtins::raw_macro! {
                    let next = [ self;
                        arr: Vec<(ImmutCloneStr32, VisSource<ValueRef>)> => self.arr.iter_mut().map(|(_, v)| v.value_mut()),
                        n: usize,
                    ] () {
                        let ret = if extra.n >= extra.arr.len() {
                            Value::Maybe(None)
                        } else {
                            let (k, v) = extra.arr[extra.n].clone();
                            let k = ValueRef::new(Value::String(k).into_stored(area.clone()));
                            let v = v.take_value();
                            Value::Maybe(Some(ValueRef::new(Value::Array(vec![k, v]).into_stored(area.clone()))))
                        };
                        extra.n += 1;

                        Multi::new_single(ctx, Ok(ValueRef::new(ret.into_stored(area))))
                    } ctx vm program area extra
                }
                MacroData {
                    target: MacroTarget::FullyRust {
                        fn_ptr: Rc::new(RefCell::new(next)),
                        args: Box::new([]),
                        spread_arg: None,
                    },

                    defaults: Box::new([]),
                    self_arg: None,

                    is_method: false,
                }
            },
            Value::Range { start, end, step } => {
                builtins::raw_macro! {
                    let next = [ self;
                        n: i64 = *start,
                        end: i64 = *end,
                        step: usize = *step,
                    ] () {
                        let ret = if extra.n >= extra.end {
                            Value::Maybe(None)
                        } else {
                            Value::Maybe(Some(ValueRef::new(Value::Int(extra.n).into_stored(area.clone()))))
                        };
                        extra.n += extra.step as i64;

                        Multi::new_single(ctx, Ok(ValueRef::new(ret.into_stored(area))))
                    } ctx vm program area extra
                }
                MacroData {
                    target: MacroTarget::FullyRust {
                        fn_ptr: Rc::new(RefCell::new(next)),
                        args: Box::new([]),
                        spread_arg: None,
                    },

                    defaults: Box::new([]),
                    self_arg: None,

                    is_method: false,
                }
            },
            v => {
                if let Some(v) = self.get_impl(v.get_type(), "_iter_") {
                    let ret = self.call_value(
                        ctx,
                        v.value().clone(),
                        &[(value.clone(), true)],
                        &[],
                        area.clone(),
                        program,
                    );

                    return ret.try_map(|ctx, v| match &v.borrow().value {
                        Value::Iterator(v) => (ctx, Ok(v.clone())),
                        _ => todo!(),
                    });
                }

                return Multi::new_single(
                    ctx,
                    Err(RuntimeError::CannotIterate {
                        value: (v.get_type(), value.borrow().area.clone()),
                        area: area.clone(),
                        call_stack: self.get_call_stack(),
                    }),
                );
            },
        };

        Multi::new_single(ctx, Ok(data))

        // fn biddy() {
        //     crate::interpreting::builtins::raw_macro! {
        //         let dig = (&mut Array(slf) as "self", gaga) {
        //             todo!()
        //         } ctx vm program area
        //     }
        // }
    }
}

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
use crate::compiling::bytecode::{
    Bytecode, CallExpr, Constant, Function, Mutability, OptBytecode, OptFunction, OptRegister,
    Register,
};
use crate::compiling::opcodes::{CallExprID, ConstID, Opcode, RuntimeStringFlag};
use crate::gd::gd_object::{make_spawn_trigger, GdObject, TriggerObject, TriggerOrder};
use crate::gd::ids::{IDClass, Id};
use crate::gd::object_keys::{ObjectKey, ObjectKeyValueType, OBJECT_KEYS};
use crate::interpreting::builtins;
use crate::interpreting::context::TryCatch;
use crate::interpreting::error::RuntimeError;
use crate::interpreting::value::{BuiltinClosure, MacroData};
use crate::parsing::ast::{ObjectType, Vis, VisSource, VisTrait};
use crate::parsing::operators::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spanned, SpwnSource, TypeDefMap, ZEROSPAN};
use crate::util::{ImmutCloneStr32, ImmutStr, ImmutVec, Str32, String32};

const RECURSION_LIMIT: usize = 256;

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub trait DeepClone<I> {
    fn deep_clone_map(
        &self,
        input: I,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) -> StoredValue;
    fn deep_clone(&self, input: I, context_split: bool) -> StoredValue {
        self.deep_clone_map(input, &mut None, context_split)
    }
    fn deep_clone_ref(&self, input: I, context_split: bool) -> ValueRef {
        let v: StoredValue = self.deep_clone(input, context_split);
        ValueRef::new(v)
    }
    fn deep_clone_ref_map(
        &self,
        input: I,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) -> ValueRef {
        let v: StoredValue = self.deep_clone_map(input, map, context_split);
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

    pub fn deep_clone_checked(
        &self,
        vm: &Vm,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) -> Self {
        if let Some(m) = map {
            if let Some(v) = m.get(self) {
                return v.clone();
            }
            let new = vm.deep_clone_ref_map(self, map, context_split);
            // goofy thing to avoid borrow checker
            if let Some(m) = map {
                m.insert(self, new.clone());
            }
            return new;
        }
        vm.deep_clone_ref_map(self, map, context_split)
    }
}

#[derive(Debug)]
pub struct Program {
    pub src: Rc<SpwnSource>,
    pub bytecode: Rc<OptBytecode>,
}

impl Program {
    pub fn get_constant(&self, id: ConstID) -> &Constant {
        &self.bytecode.constants[*id as usize]
    }

    pub fn get_call_expr(&self, id: CallExprID) -> &CallExpr<OptRegister, OptRegister, ImmutStr> {
        &self.bytecode.call_exprs[*id as usize]
    }

    pub fn get_function(&self, id: usize) -> &OptFunction {
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
    pub fn get_func(&self) -> &OptFunction {
        self.program.get_function(self.func)
    }
}

pub struct Vm {
    // readonly
    pub bytecode_map: BytecodeMap,
    pub type_def_map: TypeDefMap,

    //penis
    pub context_stack: ContextStack,

    pub objects: Vec<GdObject>,
    pub triggers: Vec<TriggerObject>,
    pub trigger_order_count: TriggerOrder,

    pub id_counters: [u16; 4],

    pub impls: AHashMap<ValueType, AHashMap<ImmutCloneStr32, VisSource<ValueRef>>>,
    pub overloads: AHashMap<Operator, Vec<VisSource<ValueRef>>>,

    pub pattern_mismatch_id_count: usize,

    pub trailing_args: Vec<String>,

    pub import_cache: AHashMap<Rc<SpwnSource>, ValueRef>,
}

impl Vm {
    pub fn new(type_def_map: TypeDefMap, bytecode_map: BytecodeMap, trailing: Vec<String>) -> Self {
        Self {
            context_stack: ContextStack(vec![]),
            objects: vec![],
            triggers: vec![],
            trigger_order_count: TriggerOrder::new(),
            type_def_map,
            bytecode_map,
            id_counters: Default::default(),
            impls: AHashMap::new(),
            overloads: AHashMap::new(),
            pattern_mismatch_id_count: 0,
            trailing_args: trailing,
            import_cache: AHashMap::new(),
        }
    }

    pub fn make_area(&self, span: CodeSpan, program: &Rc<Program>) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&program.src),
        }
    }

    pub fn insert_ctx_val<T, F>(
        &mut self,
        mut ctx: Context,
        v: RuntimeResult<T>,
        mut f: F,
        out: &mut Vec<Context>,
    ) where
        F: FnMut(&mut Context, T),
    {
        match v {
            Ok(v) => {
                f(&mut ctx, v);
                self.context_stack.last_mut().contexts.push(ctx)
            },
            Err(err) => {
                self.handle_errored_ctx(ctx, err, out);
            },
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
        for (ctx, v) in v {
            self.insert_ctx_val(ctx, v, &mut f, out);
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
        if self.id_counters[c as usize] == u16::MAX {
            panic!("{:?}", c);
        }

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
    fn deep_clone_map(
        &self,
        input: &StoredValue,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) -> StoredValue {
        let area = input.area.clone();

        let mut deep_clone_dict_items = |v: &AHashMap<ImmutCloneStr32, VisSource<ValueRef>>| {
            v.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.clone()
                            .map(|v| v.deep_clone_checked(self, map, context_split)),
                    )
                })
                .collect()
        };

        let value = match &input.value {
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| v.deep_clone_checked(self, map, context_split))
                    .collect(),
            ),
            Value::Dict(map) => Value::Dict(deep_clone_dict_items(map)),
            Value::Maybe(v) => Value::Maybe(
                v.as_ref()
                    .map(|v| v.deep_clone_checked(self, map, context_split)),
            ),
            Value::Instance { typ, items } => Value::Instance {
                typ: *typ,
                items: deep_clone_dict_items(items),
            },
            Value::Module { exports, types } => Value::Module {
                exports: exports
                    .iter()
                    .map(|(k, v)| (Rc::clone(k), v.deep_clone_checked(self, map, context_split)))
                    .collect(),
                types: types.clone(),
            },
            v @ (Value::Macro(data) | Value::Iterator(data)) => {
                let mut new_data = data.clone();
                if context_split {
                    for r in new_data.defaults.iter_mut().flatten() {
                        *r = r.deep_clone_checked(self, map, context_split)
                        // }
                    }
                }
                if let Some(r) = &mut new_data.self_arg {
                    *r = r.deep_clone_checked(self, map, context_split)
                }

                match &mut new_data.target {
                    MacroTarget::Spwn { captured, .. } => {
                        if context_split {
                            for r in captured.iter_mut() {
                                *r = r.deep_clone_checked(self, map, context_split)
                            }
                        }
                    },
                    MacroTarget::FullyRust { fn_ptr, .. } => {
                        if context_split {
                            let new = fn_ptr.borrow().shallow_clone();
                            new.borrow_mut()
                                .deep_clone_inner_refs(self, map, context_split);
                            *fn_ptr = new;
                        }
                    },
                }
                if matches!(v, Value::Macro(_)) {
                    Value::Macro(new_data)
                } else {
                    Value::Iterator(new_data)
                }
            },
            v => v.clone(),
        };

        value.into_stored(area)
    }
}

impl DeepClone<&ValueRef> for Vm {
    fn deep_clone_map(
        &self,
        input: &ValueRef,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) -> StoredValue {
        let v = input.borrow();

        self.deep_clone_map(&*v, map, context_split)
    }
}

impl DeepClone<OptRegister> for Vm {
    fn deep_clone_map(
        &self,
        input: OptRegister,
        map: &mut Option<&mut CloneMap>,
        context_split: bool,
    ) -> StoredValue {
        let v = &self.context_stack.current().stack.last().unwrap().registers[*input as usize];
        self.deep_clone_map(v, map, context_split)
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
                    ($op:ident, $a:ident, $b:ident, $left_mut:expr) => {{
                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        self.assign_op(
                            Either::Left(AssignOp::$op),
                            $left_mut,
                            &program,
                            $a,
                            $b,
                            top,
                            &mut out_contexts,
                            opcode_span,
                        );
                        return Ok(LoopFlow::ContinueLoop);
                    }};
                    (= $deep:expr, $a:ident, $b:ident, $left_mut:expr) => {{
                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        self.assign_op(
                            Either::Right($deep),
                            $left_mut,
                            &program,
                            $a,
                            $b,
                            top,
                            &mut out_contexts,
                            opcode_span,
                        );
                        return Ok(LoopFlow::ContinueLoop);
                    }};
                }
                macro_rules! unary_op {
                    ($op:ident, $v:ident, $to:ident) => {{
                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.ip += 1;

                        self.unary_op(
                            UnaryOp::$op,
                            &program,
                            $v,
                            $to,
                            top,
                            &mut out_contexts,
                            opcode_span,
                        );
                        return Ok(LoopFlow::ContinueLoop);
                    }};
                }

                match opcode {
                    Opcode::LoadConst { id, to } => {
                        let value = Value::from_const(
                            self,
                            program.get_constant(id),
                            self.make_area(opcode_span, &program),
                        );
                        self.change_reg(
                            to,
                            value.into_stored(self.make_area(opcode_span, &program)),
                        );
                    },
                    Opcode::CopyDeep { from, to } => {
                        self.change_reg(to, self.deep_clone(from, false))
                    },
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
                    Opcode::WriteDeep { from, to } => {
                        self.write_pointee(to, self.deep_clone(from, false))
                    },
                    Opcode::AssignRef { from, to, left_mut } => {
                        assign_op!(= false, to, from, left_mut);
                    },
                    Opcode::AssignDeep { from, to, left_mut } => {
                        assign_op!(= true, to, from, left_mut);
                    },
                    Opcode::Plus { a, b, to } => {
                        // println!(
                        //     "{:#?}",
                        //     self.overloads
                        //         .get(&Operator::Bin(BinOp::Plus))
                        //         .map(|v| v.len())
                        // );
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
                        ));
                        self.change_reg(to, v.into_stored(self.make_area(opcode_span, &program)));
                    },
                    Opcode::PureNeq { a, b, to } => {
                        let v = Value::Bool(!value_ops::equality(
                            &self.get_reg_ref(a).borrow().value,
                            &self.get_reg_ref(b).borrow().value,
                        ));
                        self.change_reg(to, v.into_stored(self.make_area(opcode_span, &program)));
                    },
                    Opcode::PureGte { a, b, to } => {
                        // let v = value_ops::e
                        let v = value_ops::gte(
                            &self.get_reg_ref(a).borrow(),
                            &self.get_reg_ref(b).borrow(),
                            opcode_span,
                            self,
                            &program,
                        )?;
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
                    Opcode::BinXor { a, b, to } => {
                        bin_op!(BinXor, a, b, to);
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
                    Opcode::PlusEq { a, b, left_mut } => {
                        assign_op!(PlusEq, a, b, left_mut);
                    },
                    Opcode::MinusEq { a, b, left_mut } => {
                        assign_op!(MinusEq, a, b, left_mut);
                    },
                    Opcode::MultEq { a, b, left_mut } => {
                        assign_op!(MultEq, a, b, left_mut);
                    },
                    Opcode::DivEq { a, b, left_mut } => {
                        assign_op!(DivEq, a, b, left_mut);
                    },
                    Opcode::PowEq { a, b, left_mut } => {
                        assign_op!(PowEq, a, b, left_mut);
                    },
                    Opcode::ModEq { a, b, left_mut } => {
                        assign_op!(ModEq, a, b, left_mut);
                    },
                    Opcode::BinAndEq { a, b, left_mut } => {
                        assign_op!(BinAndEq, a, b, left_mut);
                    },
                    Opcode::BinOrEq { a, b, left_mut } => {
                        assign_op!(BinOrEq, a, b, left_mut);
                    },
                    Opcode::BinXorEq { a, b, left_mut } => {
                        assign_op!(BinXorEq, a, b, left_mut);
                    },
                    Opcode::ShiftLeftEq { a, b, left_mut } => {
                        assign_op!(ShiftLeftEq, a, b, left_mut);
                    },
                    Opcode::ShiftRightEq { a, b, left_mut } => {
                        assign_op!(ShiftRightEq, a, b, left_mut);
                    },
                    Opcode::Not { v, to } => {
                        unary_op!(ExclMark, v, to);
                    },
                    Opcode::Negate { v, to } => {
                        unary_op!(Minus, v, to);
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
                                (ctx, Ok(Value::Iterator(v).into_value_ref(area.clone())))
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
                        let push = self.deep_clone_ref(elem, false);
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
                        let push = self.deep_clone_ref(elem, false);

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
                        let push = self.deep_clone_ref(elem, false);

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

                    Opcode::AllocObject { dest, capacity } => self.change_reg(
                        dest,
                        StoredValue {
                            value: Value::Object {
                                params: AHashMap::with_capacity(capacity as usize),
                                typ: ObjectType::Object,
                            },
                            area: self.make_area(opcode_span, &program),
                        },
                    ),
                    Opcode::AllocTrigger { dest, capacity } => self.change_reg(
                        dest,
                        StoredValue {
                            value: Value::Object {
                                params: AHashMap::with_capacity(capacity as usize),
                                typ: ObjectType::Trigger,
                            },
                            area: self.make_area(opcode_span, &program),
                        },
                    ),
                    Opcode::PushObjectElemKey {
                        elem,
                        obj_key,
                        dest,
                    } => {
                        // Objec
                        let push = self.deep_clone(elem, false);

                        let param = {
                            let types = obj_key.types();

                            let mut valid = false;

                            for t in types {
                                match (t, &push.value) {
                                    (ObjectKeyValueType::Int, Value::Int(_))
                                    | (
                                        ObjectKeyValueType::Float,
                                        Value::Float(_) | Value::Int(_),
                                    )
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
                                    },

                                    (ObjectKeyValueType::GroupArray, Value::Array(v))
                                        if v.iter().all(|k| {
                                            matches!(&k.borrow().value, Value::Group(_))
                                        }) =>
                                    {
                                        valid = true;
                                        break;
                                    },

                                    _ => (),
                                }
                            }

                            if !valid {
                                println!("{:?} {:?}", types, &push.value);
                                // todo!()
                                todo!(
                                    "\n\nOk   heres the deal!!! I not this yet XDXDC😭😭🤣🤣 \nLOl"
                                )
                            }

                            value_ops::to_obj_param(&push, opcode_span, self, &program)?
                        };

                        match &mut self.get_reg_ref(dest).borrow_mut().value {
                            Value::Object { params, .. } => {
                                params.insert(obj_key.id(), param);
                            },
                            _ => unreachable!(),
                        }
                    },
                    Opcode::PushObjectElemUnchecked {
                        elem,
                        obj_key,
                        dest,
                    } => todo!(),
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
                    Opcode::ApplyStringFlag { flag, reg } => {
                        let area = self.make_area(opcode_span, &program);

                        let val = match &self.get_reg_ref(reg).borrow().value {
                            Value::String(s) => {
                                let mut v = vec![];

                                match flag {
                                    RuntimeStringFlag::ByteString => {
                                        for c in s.chars() {
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
                                        let s = unindent::unindent(&s.to_string());

                                        Value::String(String32::from(s).into())
                                    },
                                    RuntimeStringFlag::Base64 => {
                                        let s = base64::engine::general_purpose::URL_SAFE
                                            .encode(s.to_string());

                                        Value::String(String32::from(s).into())
                                    },
                                }
                            },
                            _ => unreachable!(),
                        };

                        self.change_reg(reg, val.into_stored(area))
                    },
                    Opcode::WrapMaybe { from, to } => {
                        let v = self.deep_clone_ref(from, false);
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

                            mem::drop(r);

                            self.import_cache
                                .insert(program.src.clone(), ret_val.clone());
                        }

                        self.context_stack.last_mut().have_returned = true;

                        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
                        top.stack.pop();

                        top.returned = Ok(Some(ret_val));
                        out_contexts.push(top);

                        return Ok(LoopFlow::ContinueLoop);
                    },
                    Opcode::Dbg { reg, .. } => {
                        self.olga_sex(reg, opcode_span, &program, &mut out_contexts);

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

                        // for (src, _) in &self.import_cache {
                        //     println!("fetus {:?}", src);
                        // }

                        if let Some(v) = self.import_cache.get(import) {
                            // println!("gorgonzola {:?}", import);
                            self.change_reg(dest, v.clone());
                        } else {
                            // println!("{:?}", import);
                            self.import_cache.insert(
                                Rc::new(import.clone()),
                                ValueRef::new(
                                    Value::Dict(AHashMap::new())
                                        .into_stored(self.make_area(ZEROSPAN, &program)),
                                ),
                            );

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
                        }
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
                                    Ok(Value::String(String32::from(v).into())
                                        .into_value_ref(area.clone())),
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
                            (Value::Range { step, .. }, "step") => Some(Value::Int(*step)),

                            (Value::TriggerFunction { group, .. }, "start_group") => {
                                Some(Value::Group(*group))
                            },

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
                                            if let VisSource::Private(_, src) = v {
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

                                let mut v = self.deep_clone(r.value(), false);

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

                        let key_str: String = key.as_ref().into();

                        let value = self.get_reg_ref(from).borrow();

                        match (&value.value, &key_str[..]) {
                            (Value::Type(ValueType::Error), k)
                                if RuntimeError::VARIANT_NAMES.contains(&k) =>
                            {
                                mem::drop(value);
                                self.change_reg(
                                    dest,
                                    Value::Error(
                                        RuntimeError::VARIANT_NAMES
                                            .iter()
                                            .position(|v| *v == k)
                                            .unwrap(),
                                    )
                                    .into_stored(self.make_area(opcode_span, &program)),
                                );
                            },
                            _ => match &value.value {
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
                            },
                        }
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
                                        type_name: key.to_string(),
                                        call_stack: self.get_call_stack(),
                                    })?;

                                if typ.is_priv() {
                                    return Err(RuntimeError::PrivateType {
                                        area: self.make_area(opcode_span, &program),
                                        type_name: key.to_string(),
                                        call_stack: self.get_call_stack(),
                                    });
                                }

                                let typ = *typ.value();

                                mem::drop(from);

                                self.change_reg(
                                    dest,
                                    StoredValue {
                                        value: Value::Type(ValueType::Custom(typ)),
                                        area: self.make_area(opcode_span, &program),
                                    },
                                );
                            },
                            v => {
                                return Err(RuntimeError::TypeMismatch {
                                    value_type: v.get_type(),
                                    value_area: from.area.clone(),
                                    area: self.make_area(opcode_span, &program),
                                    expected: &[ValueType::Module],
                                    call_stack: self.get_call_stack(),
                                })
                            },
                        }
                    },
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
                            Value::Instance { items, .. } => items.len(),
                            Value::Module { exports, .. } => exports.len(),
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
                    Opcode::ArgAmount { src, dest } => {
                        let amount = match &self.get_reg_ref(src).borrow().value {
                            Value::Macro(data) => data.defaults.len(),
                            _ => {
                                unreachable!()
                            },
                        };

                        self.change_reg(
                            dest,
                            Value::Int(amount as i64)
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
                                // if !matches!(
                                //     &*program.src,
                                //     SpwnSource::Core(_) | SpwnSource::Std(_)
                                // ) {
                                //     return Err(RuntimeError::ImplOnBuiltin {
                                //         area: self.make_area(opcode_span, &program),
                                //         call_stack: self.get_call_stack(),
                                //     });
                                // }
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
                        self.overloads
                            .entry(op)
                            .or_insert(vec![])
                            .push(VisSource::Public(v));
                    },
                    Opcode::AddPrivateOperatorOverload { from, op } => {
                        let v = self.get_reg_ref(from).clone();
                        self.overloads
                            .entry(op)
                            .or_insert(vec![])
                            .push(VisSource::Private(v, Rc::clone(&program.src)));
                    },
                    Opcode::IncMismatchIdCount => {
                        self.pattern_mismatch_id_count += 1;
                    },
                    // Opcode::TestEq { a, b, .. } => todo!(),
                }
                Ok(LoopFlow::Normal)
            };
            let run = run_opcode(opcode);

            // println!(
            //     "{ip}: {:?}",
            //     self.context_stack
            //         .current()
            //         .stack
            //         .last()
            //         .unwrap()
            //         .registers
            //         .iter()
            //         .map(|v| format!("{:?}", v.borrow().value))
            //         .join(", ")
            // );

            match run {
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
                    None => Value::Empty.into_value_ref(CodeArea {
                        src: Rc::clone(&program.src),
                        span: CodeSpan::internal(),
                    }),
                });

                ctx.ip = original_ip;

                (ctx, ret)
            })
            .collect()
    }

    fn olga_sex(
        &mut self,
        reg: Register<u8>,
        opcode_span: CodeSpan,
        program: &Rc<Program>,
        out_contexts: &mut Vec<Context>,
    ) {
        let r = self.get_reg_ref(reg).clone();

        let mut top = self.context_stack.last_mut().yeet_current().unwrap();
        top.ip += 1;

        let s = self.runtime_display(top, &r, &self.make_area(opcode_span, program), program);

        let gog = self
            .context_stack
            .iter()
            .map(|c| c.contexts.len())
            .join(", ");

        self.insert_multi(
            s,
            |ctx, v| {
                println!(
                    "{} {} {} {} {} {}",
                    v,
                    "::".dimmed(),
                    ctx.unique_id.to_string().bright_blue(),
                    ctx.group.fmt("g").green(),
                    format!("{:?}", r.as_ptr()).dimmed(),
                    gog // ctx.stack.len(),
                );
            },
            out_contexts,
        );
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
        let base_ref = base.borrow();
        let base_area = base_ref.area.clone();

        match &base_ref.value {
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
                        );
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

                for (i, arg) in fill.iter_mut().enumerate() {
                    if let ArgFill::Single(opt @ None, name, ..) = arg {
                        if let Some(v) = &data.defaults[i] {
                            *opt = Some(v.clone());
                        } else {
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
                }

                match &data.target {
                    MacroTarget::Spwn {
                        func,
                        is_builtin,
                        captured,
                    } => {
                        let func = func.clone();
                        let is_builtin = *is_builtin;
                        let captured = captured
                            .iter()
                            .zip(func.get_func().captured_regs.iter())
                            .map(|(v, (_, to))| (v.clone(), *to))
                            .collect_vec();

                        mem::drop(base_ref);

                        self.run_function(
                            ctx,
                            CallInfo {
                                func,
                                call_area: Some(call_area.clone()),
                                is_builtin,
                            },
                            Box::new(move |vm| {
                                // println!("babagaga1");
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

                                for (i, (v, to)) in captured.into_iter().enumerate() {
                                    vm.change_reg(to, v.clone());
                                }
                                // println!("babagaga4");
                            }),
                            // ContextSplitMode::Allow,
                        )
                    },
                    MacroTarget::FullyRust { fn_ptr, .. } => {
                        let fn_ptr = fn_ptr.clone();
                        mem::drop(base_ref);

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
            Value::Type(vt) => {
                todo!()
            },
            v => {
                if let Some(v) = self.get_impl(v.get_type(), "_call_") {
                    let mut positional = vec![(base.clone(), true)];
                    for i in positional_args {
                        positional.push(i.clone())
                    }
                    return self.call_value(
                        ctx,
                        v.value().clone(),
                        &positional,
                        named_args,
                        call_area.clone(),
                        program,
                    );
                }

                Multi::new_single(
                    ctx,
                    Err(RuntimeError::CannotCall {
                        value: (v.get_type(), base_area),
                        area: call_area.clone(),
                        call_stack: self.get_call_stack(),
                    }),
                )
            },
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
        // TOODO: overloads // no longer toodo you absolute fucking fuckwit, you fucking dumbass, you are so fucking dumb holy fucking shit, i wish i was an egirl

        let mut out = Multi::new_single(ctx, None);

        if let Some(v) = self.overloads.get(&Operator::Bin(op)).cloned() {
            for base in v {
                out = out.flat_map(|ctx, v| {
                    if v.as_ref().is_none() {
                        let check_id = self.pattern_mismatch_id_count;

                        let left_ref = ctx.stack.last().unwrap().registers[*left as usize].clone();
                        let right_ref =
                            ctx.stack.last().unwrap().registers[*right as usize].clone();

                        self.call_value(
                            ctx,
                            base.value().clone(),
                            &[(left_ref, false), (right_ref, false)],
                            &[],
                            self.make_area(span, program),
                            program,
                        )
                        .map(|ctx, v| match v {
                            Ok(v) => (ctx, Some(Ok(v))),
                            Err(err) => {
                                if let RuntimeError::PatternMismatch { id, .. } = err {
                                    if id == check_id {
                                        return (ctx, None);
                                    }
                                    println!("GIG: {id} {check_id}");
                                }
                                (ctx, Some(Err(err)))
                            },
                        })
                    } else {
                        Multi::new_single(ctx, v)
                    }
                });
            }
        }

        let func = op.get_fn();

        let out = match func {
            Either::Left(func) => out.map(|ctx, v| {
                let v = match v {
                    Some(Ok(v)) => Ok(v),
                    Some(Err(err)) => Err(err),
                    None => func(
                        &ctx.stack.last().unwrap().registers[*left as usize].borrow(),
                        &ctx.stack.last().unwrap().registers[*right as usize].borrow(),
                        span,
                        self,
                        program,
                    )
                    .map(|v| v.into_value_ref(self.make_area(span, program))),
                };
                (ctx, v)
            }),
            Either::Right(func) => out.flat_map(|ctx, v| {
                let v = match v {
                    Some(Ok(v)) => Ok(v),
                    Some(Err(err)) => Err(err),
                    None => {
                        return func(left, right, ctx, span, self, program).try_map(|ctx, v| {
                            (ctx, Ok(v.into_value_ref(self.make_area(span, program))))
                        })
                    },
                };
                Multi::new_single(ctx, v)
            }),
        };

        // let out = out.map(|ctx, v| {
        //     let v = match v {
        //         Some(Ok(v)) => Ok(v),
        //         Some(Err(err)) => Err(err),
        //         None => op.get_fn()(
        //             &ctx.stack.last().unwrap().registers[*left as usize].borrow(),
        //             &ctx.stack.last().unwrap().registers[*right as usize].borrow(),
        //             span,
        //             self,
        //             program,
        //         )
        //         .map(|v| ValueRef::new(v.into_stored(self.make_area(span, program)))),
        //     };
        //     (ctx, v)
        // });

        self.change_reg_multi(to, out, out_contexts);
    }

    #[inline]
    fn assign_op(
        &mut self,
        // left: any assign eq
        // right: normal assign, bool: is by ref
        op: Either<AssignOp, bool>,
        left_mut: bool,
        program: &Rc<Program>,
        left: OptRegister,
        right: OptRegister,
        mut ctx: Context,
        out_contexts: &mut Vec<Context>,
        span: CodeSpan,
    ) {
        if let Either::Right(true) = op {
            let v = ctx.stack.last().unwrap().registers[*right as usize].clone();
            ctx.stack.last_mut().unwrap().registers[*right as usize] =
                self.deep_clone_ref(&v, false)
        }

        let mut out: Multi<Option<Result<(), RuntimeError>>> = Multi::new_single(ctx, None);

        if let Some(v) = self
            .overloads
            .get(&match op {
                Either::Left(op) => Operator::Assign(op),
                Either::Right(_) => Operator::EqAssign,
            })
            .cloned()
        {
            for base in v {
                out = out.flat_map(|ctx, v| {
                    if v.as_ref().is_none() {
                        let check_id = self.pattern_mismatch_id_count;

                        let left_ref = ctx.stack.last().unwrap().registers[*left as usize].clone();
                        let right_ref =
                            ctx.stack.last().unwrap().registers[*right as usize].clone();

                        self.call_value(
                            ctx,
                            base.value().clone(),
                            &[(left_ref, left_mut), (right_ref, false)],
                            &[],
                            self.make_area(span, program),
                            program,
                        )
                        .map(|ctx, v| match v {
                            Ok(_) => (ctx, Some(Ok(()))),
                            Err(err) => {
                                if let RuntimeError::PatternMismatch { id, .. } = err {
                                    if id == check_id {
                                        return (ctx, None);
                                    }
                                    println!("GIG: {id} {check_id}");
                                }
                                (ctx, Some(Err(err)))
                            },
                        })
                    } else {
                        Multi::new_single(ctx, v)
                    }
                });
            }
        }

        // Right(deep) -> do normal `=` assign shit
        // Left(None) -> aint do nun
        // Left(Some(v)) -> will store `v` in `left`

        let out: Multi<RuntimeResult<Either<Option<StoredValue>, bool>>> = out.map(|ctx, v| {
            let v = match v {
                Some(Ok(_)) => Ok(Either::Left(None)),
                Some(Err(err)) => Err(err),
                None => match op {
                    Either::Left(op) => {
                        // println!("usumyd {}", left_mut);
                        if left_mut {
                            op.get_fn()(
                                &ctx.stack.last().unwrap().registers[*left as usize].borrow(),
                                &ctx.stack.last().unwrap().registers[*right as usize].borrow(),
                                span,
                                self,
                                program,
                            )
                            .map(|v| v.into_stored(self.make_area(span, program)))
                            .map(|v| Either::Left(Some(v)))
                        } else {
                            Err(RuntimeError::ImmutableAssign {
                                area: self.make_area(span, program),
                                def_area: ctx.stack.last().unwrap().registers[*left as usize]
                                    .borrow()
                                    .area
                                    .clone(),
                            })
                        }
                    },
                    Either::Right(deep) => {
                        if !left_mut {
                            Err(RuntimeError::ImmutableAssign {
                                area: self.make_area(span, program),
                                def_area: ctx.stack.last().unwrap().registers[*left as usize]
                                    .borrow()
                                    .area
                                    .clone(),
                            })
                        } else {
                            Ok(Either::Right(deep))
                        }
                    },
                },
            };
            (ctx, v)
        });

        self.insert_multi(
            out,
            |ctx, v| {
                match v {
                    Either::Left(Some(v)) => {
                        let mut g =
                            ctx.stack.last_mut().unwrap().registers[*left as usize].borrow_mut();
                        *g = v;
                    },
                    Either::Right(deep) => {
                        // println!("halquimura {}", deep);
                        if deep {
                            let right_val = ctx.stack.last().unwrap().registers[*right as usize]
                                .borrow()
                                .clone();
                            // println!("olgacock {:?}", right_val.value);
                            let mut g = ctx.stack.last_mut().unwrap().registers[*left as usize]
                                .borrow_mut();
                            *g = right_val;
                        } else {
                            let right_ref =
                                ctx.stack.last().unwrap().registers[*right as usize].clone();
                            ctx.stack.last_mut().unwrap().registers[*left as usize] = right_ref;
                        }
                    },
                    _ => (),
                }
            },
            out_contexts,
        );
    }

    #[inline]
    fn unary_op(
        &mut self,
        op: UnaryOp,
        program: &Rc<Program>,
        value: OptRegister,
        dest: OptRegister,
        ctx: Context,
        out_contexts: &mut Vec<Context>,
        span: CodeSpan,
    ) {
        let mut out = Multi::new_single(ctx, None);

        if let Some(v) = self.overloads.get(&Operator::Unary(op)).cloned() {
            for base in v {
                out = out.flat_map(|ctx, v| {
                    if v.as_ref().is_none() {
                        let check_id = self.pattern_mismatch_id_count;
                        let value_ref =
                            ctx.stack.last().unwrap().registers[*value as usize].clone();
                        self.call_value(
                            ctx,
                            base.value().clone(),
                            &[(value_ref.clone(), false)],
                            &[],
                            self.make_area(span, program),
                            program,
                        )
                        .map(|ctx, v| match v {
                            Ok(v) => (ctx, Some(Ok(v))),
                            Err(err) => {
                                if let RuntimeError::PatternMismatch { id, .. } = err {
                                    if id == check_id {
                                        return (ctx, None);
                                    }
                                    println!("GIG: {id} {check_id}");
                                }
                                (ctx, Some(Err(err)))
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
                Some(Ok(v)) => Ok(v),
                Some(Err(err)) => Err(err),
                None => op.get_fn()(
                    &ctx.stack.last().unwrap().registers[*value as usize].borrow(),
                    span,
                    self,
                    program,
                )
                .map(|v| v.into_value_ref(self.make_area(span, program))),
            };
            (ctx, v)
        });

        self.change_reg_multi(dest, out, out_contexts);
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

            (Value::Bool(b), ValueType::Int) => Value::Int(*b as i64),
            (Value::Bool(b), ValueType::Float) => Value::Float(*b as i64 as f64),

            (Value::Dict(map), ValueType::Array) => {
                let a: BTreeMap<_, _> = map.iter().collect();
                let area = self.make_area(span, program);
                Value::Array(
                    a.into_iter()
                        .map(|(k, v)| {
                            let s = Value::String(k.clone()).into_value_ref(area.clone());
                            let v = self.deep_clone_ref(v.value(), false);
                            Value::Array(vec![s, v]).into_value_ref(area.clone())
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
                        Value::String(Str32::from_char_slice(&[c]).into())
                            .into_value_ref(v.borrow().area.clone())
                    })
                    .collect(),
            ),

            (_, ValueType::String) => {
                return self
                    .runtime_display(ctx, v, &self.make_area(span, program), program)
                    .try_map(|ctx, v| (ctx, Ok(Value::String(String32::from_str(&v).into()))))
            },
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
            Value::Object { params, typ } => {
                for (id, p) in params {
                    id.hash(state);
                    p.hash(state);
                }
                typ.hash(state);
            },
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
                format!("{} {{...}}", RuntimeError::VARIANT_NAMES[*id])
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

                    // todo: while calling overload error

                    return ret.try_map(|ctx, v| match &v.borrow().value {
                        Value::String(v) => (ctx, Ok(v.as_ref().into())),
                        v => (
                            ctx,
                            Err(RuntimeError::InvalidReturnTypeForBuiltin {
                                area: area.clone(),
                                found: v.get_type(),
                                expected: ValueType::String,
                                builtin: "_display_",
                            }),
                        ),
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
            Value::Object { params, typ } => format!(
                "{} {{ {} }}",
                match typ {
                    ObjectType::Object => "obj",
                    ObjectType::Trigger => "trigger",
                },
                params.iter().map(|(s, k)| format!("{s}: {k:?}")).join(", ")
            ),
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

                        Multi::new_single(ctx, Ok(ret.into_value_ref(area)))
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
                            let k = Value::String(k).into_value_ref(area.clone());
                            let v = v.take_value();
                            Value::Maybe(Some(Value::Array(vec![k, v]).into_value_ref(area.clone())))
                        };
                        extra.n += 1;

                        Multi::new_single(ctx, Ok(ret.into_value_ref(area)))
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
                        step: i64 = *step,
                    ] () {
                        let ret = if extra.n == extra.end {
                            Value::Maybe(None)
                        } else {
                            Value::Maybe(Some(Value::Int(extra.n).into_value_ref(area.clone())))
                        };
                        extra.n += extra.step;

                        Multi::new_single(ctx, Ok(ret.into_value_ref(area)))
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

                    // todo: while calling overload error

                    return ret.try_map(|ctx, v| match &v.borrow().value {
                        Value::Iterator(v) => (ctx, Ok(v.clone())),
                        v => (
                            ctx,
                            Err(RuntimeError::InvalidReturnTypeForBuiltin {
                                area: area.clone(),
                                found: v.get_type(),
                                expected: ValueType::Iterator,
                                builtin: "_iter_",
                            }),
                        ),
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
    }
}

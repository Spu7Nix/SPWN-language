use std::collections::HashMap;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use super::error::CompilerError;
use crate::{
    interpreter::{interpreter::StoredValue, value::Value},
    leveldata::object_data::ObjectMode,
    parser::{
        ast::{ASTData, ExprKey, Expression, Statement, Statements, StmtKey},
        lexer::Token,
    },
    sources::{CodeArea, CodeSpan, SpwnSource},
};

// use crate::interpreter::value::Value;

pub type InstrNum = u16;

#[derive(Serialize, Deserialize)]
pub struct UniqueRegister<T> {
    vec: Vec<T>,
}
impl<T: PartialEq> UniqueRegister<T> {
    pub fn new() -> Self {
        Self { vec: vec![] }
    }
    pub fn add(&mut self, c: T) -> InstrNum {
        (match self.vec.iter().position(|v| v == &c) {
            Some(i) => i,
            None => {
                self.vec.push(c);
                self.vec.len() - 1
            }
        }) as InstrNum
    }
    pub fn get(&self, idx: InstrNum) -> &T {
        &self.vec[idx as usize]
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum Const {
    Int(u32),
    Float(f64),
    String(String),
    Bool(bool),
}
impl Const {
    pub fn to_value(&self) -> Value {
        match self {
            Const::Int(v) => Value::Int(*v as i64),
            Const::Float(v) => Value::Float(*v),
            Const::String(v) => Value::String(v.clone()),
            Const::Bool(v) => Value::Bool(*v),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BytecodeFunc {
    pub instructions: Vec<Instruction>,
    pub arg_ids: Vec<InstrNum>,
    pub capture_ids: Vec<InstrNum>,
    pub scoped_var_ids: Vec<InstrNum>,
}

#[derive(Serialize, Deserialize)]
pub struct Code {
    pub source: SpwnSource,

    pub constants: UniqueRegister<Const>,
    pub names: UniqueRegister<String>,
    pub destinations: UniqueRegister<usize>,
    pub name_sets: UniqueRegister<Vec<String>>,
    pub scope_vars: UniqueRegister<Vec<InstrNum>>,
    #[allow(clippy::type_complexity)]
    pub macro_build_info: UniqueRegister<(usize, Vec<(String, bool, bool)>)>,

    pub var_count: usize,
    pub bytecode_funcs: Vec<BytecodeFunc>,

    pub bytecode_spans: HashMap<(usize, usize), CodeSpan>,
    pub macro_arg_spans: HashMap<(usize, usize), Vec<CodeSpan>>,
}
impl Code {
    pub fn new(source: SpwnSource) -> Self {
        Self {
            source,
            constants: UniqueRegister::new(),
            names: UniqueRegister::new(),
            destinations: UniqueRegister::new(),
            name_sets: UniqueRegister::new(),
            macro_build_info: UniqueRegister::new(),
            scope_vars: UniqueRegister::new(),
            bytecode_funcs: vec![],
            var_count: 0,
            bytecode_spans: HashMap::new(),
            macro_arg_spans: HashMap::new(),
        }
    }
    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            source: self.source.clone(),
        }
    }
    pub fn dummy_value(&self) -> StoredValue {
        Value::Empty.into_stored(self.make_area((0..0).into()))
    }

    pub fn get_bytecode_span(&self, func: usize, i: usize) -> CodeSpan {
        *self.bytecode_spans.get(&(func, i)).unwrap()
    }

    pub fn debug(&self) {
        for (
            i,
            BytecodeFunc {
                instructions,
                arg_ids,
                capture_ids,
                scoped_var_ids,
            },
        ) in self.bytecode_funcs.iter().enumerate()
        {
            println!(
                "============================> Func {}      {:?}      {:?}      {:?}",
                i, arg_ids, capture_ids, scoped_var_ids
            );
            for (i, instr) in instructions.iter().enumerate() {
                use ansi_term::Color::Yellow;
                let instr_len = 25;

                let col = Yellow.bold();

                let instr_str = format!("{:?}", instr);
                let instr_str =
                    instr_str.clone() + &String::from(" ").repeat(instr_len - instr_str.len());

                let mut s = format!("{}\t{}", i, instr_str);

                match instr {
                    Instruction::LoadConst(idx) => {
                        s += &col.paint(format!("{:?}", self.constants.get(*idx)))
                    }
                    Instruction::LoadType(idx) => {
                        s += &col.paint(format!("@{}", self.names.get(*idx)))
                    }
                    Instruction::Jump(idx) => {
                        s += &col.paint(format!("to {:?}", self.destinations.get(*idx)))
                    }
                    Instruction::JumpIfFalse(idx) => {
                        s += &col.paint(format!("to {:?}", self.destinations.get(*idx)))
                    }
                    Instruction::IterNext(idx) => {
                        s += &col.paint(format!("to {:?}", self.destinations.get(*idx)))
                    }
                    Instruction::BuildDict(idx) => {
                        s += &col.paint(format!("with {:?}", self.name_sets.get(*idx)))
                    }
                    Instruction::MakeMacro(idx) => {
                        s += &col.paint(format!(
                            "Func {:?}, args: {:?}",
                            self.macro_build_info.get(*idx).0,
                            self.macro_build_info.get(*idx).1
                        ))
                    }
                    Instruction::Call(idx) => {
                        s += &col.paint(format!("params: {:?}", self.name_sets.get(*idx)))
                    }
                    Instruction::TypeDef(idx) => {
                        s += &col.paint(format!("@{}", self.names.get(*idx)))
                    }
                    Instruction::Impl(idx) => {
                        s += &col.paint(format!("with {:?}", self.name_sets.get(*idx)))
                    }
                    Instruction::Instance(idx) => {
                        s += &col.paint(format!("with {:?}", self.name_sets.get(*idx)))
                    }
                    Instruction::PushVars(idx) => {
                        s += &col.paint(format!("scope vars: {:?}", self.scope_vars.get(*idx)))
                    }
                    Instruction::PopVars(idx) => {
                        s += &col.paint(format!("scope vars: {:?}", self.scope_vars.get(*idx)))
                    }
                    _ => (),
                }

                println!("{}", s);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Instruction {
    // ok am writing docstring for each instruction so i understand
    /// loads constant value and pushes to stack ???
    LoadConst(InstrNum),

    /// pops two elements from the stack and pushes their sum
    Plus,
    /// pops two elements from the stack and pushes their difference
    Minus,
    /// pops two elements from the stack and pushes their product
    Mult,
    /// pops two elements from the stack and pushes their quotient
    Div,
    /// pops two elements from the stack and pushes their modulatoratron (yes that is the same; source: i made it the fuck up)
    Mod,
    /// pops two elements from the stack and pushes their supernumber
    Pow,

    // these are all the same i suppose
    Eq,
    NotEq,
    Greater,
    GreaterEq,
    Lesser,
    LesserEq,

    Negate,
    Not,

    Is,

    /// loads mutable variable and pushes to stack
    LoadVar(InstrNum),
    /// mutates variable
    SetVar(InstrNum),
    /// loads type and then pushes type inddicator to stack maybe ??
    LoadType(InstrNum),
    /// makes array from last n elements on the stack
    BuildArray(InstrNum),

    /// pushes () to the stack
    PushEmpty,
    /// pops stack
    PopTop,

    /// jumps to instruction at index
    Jump(InstrNum),
    /// jumps to instruction at index if top of stack is false
    JumpIfFalse(InstrNum),
    /// lol what
    ToIter,
    /// for loop something something
    IterNext(InstrNum),
    /// makes dict from last n elements on the stack
    BuildDict(InstrNum),
    /// makes gd object data structure from last n elements on the stack
    BuildObject(InstrNum),
    BuildTrigger(InstrNum),
    AddObject,

    /// returns to callsite
    ReturnValue(bool),

    /// ig it takes the elements on the stack and uses them to make macro value
    MakeMacro(InstrNum),
    /// pushes wildcard patterbn
    PushAnyPattern,
    /// makes macro patern
    MakeMacroPattern(InstrNum),
    /// pushes nth index of array where n and array are on the stack
    Index,
    /// calls macro with n elements on the stack
    Call(InstrNum),
    /// calls trigger function
    TriggerFuncCall,

    /// merges contexts
    MergeContexts,
    // wait is this the ? Option syntax thing lol
    PushNone,
    WrapMaybe,

    /// ?g basically
    LoadArbitraryGroup,
    /// Splits context, sends one to the end of the trigger func, sets the group of the other
    EnterArrowStatement(InstrNum),
    // aaa
    EnterTriggerFunction(InstrNum),
    /// stops and deletes the context
    YeetContext,

    /// make type
    TypeDef(InstrNum),
    /// make impl ?? wwhy is this a bytecode instruction shouldnt this be compiletime or somethinghb
    /// // wait is it for like @aa::b = v
    Impl(InstrNum),
    /// ???????
    Instance(InstrNum),

    Print,
    Split,

    PushVars(InstrNum),
    PopVars(InstrNum),

    PushWhile,

    /// pops from the block stack. if `true`, it will keep popping until it pops a For or While block
    PopBlock(bool),
}

pub struct Scope {
    pub vars: AHashMap<String, (InstrNum, bool, CodeSpan)>,
    pub parent: Option<ScopeKey>,
}
impl Scope {
    pub fn base() -> Scope {
        Self {
            vars: AHashMap::new(),
            parent: None,
        }
    }
    pub fn var_ids(&self) -> Vec<InstrNum> {
        self.vars
            .iter()
            .map(|(_, (id, ..))| *id)
            .collect::<Vec<_>>()
    }
}

new_key_type! { pub struct ScopeKey; }

#[derive(Debug)]
pub struct MacroQueuedExec {
    code_key: ExprKey,
    func_id: usize,
    derived_scope: ScopeKey,
    original_scope: ScopeKey,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecLayer {
    Loop {
        breaks: Vec<usize>,
        continues: Vec<usize>,
    },
    Async,
    Macro,
}
impl ExecLayer {
    pub fn new_loop() -> Self {
        ExecLayer::Loop {
            breaks: vec![],
            continues: vec![],
        }
    }
    pub fn new_func() -> Self {
        ExecLayer::Macro
    }
    pub fn new_async() -> Self {
        ExecLayer::Async
    }
    pub fn into_loop(self) -> Self {
        match self {
            ExecLayer::Loop { .. } => self,
            ExecLayer::Async => self,
            ExecLayer::Macro => panic!("called into_loop on func layer"),
        }
    }
}

pub struct Compiler {
    pub source: SpwnSource,

    pub ast_data: ASTData,
    pub code: Code,
    #[allow(clippy::type_complexity)]
    pub macro_exec_queue: Vec<Vec<MacroQueuedExec>>,

    pub scopes: SlotMap<ScopeKey, Scope>,
    pub layers: Vec<ExecLayer>,
}

impl Compiler {
    pub fn new(data: ASTData, source: SpwnSource) -> Self {
        Self {
            code: Code::new(source.clone()),
            source,
            ast_data: data,
            macro_exec_queue: vec![],
            scopes: SlotMap::default(),
            layers: vec![],
        }
    }
    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            source: self.source.clone(),
        }
    }

    // pub fn run_stmts_scoped(
    //     &mut self,
    //     code: Statements,
    //     scope: ScopeKey,
    //     func: usize,
    // ) -> Result<(), CompilerError> {
    //     let enter = self.push_instr(Instruction::PushVars(0), func);
    //     self.compile_stmts(code, scope, func)?;

    //     let vars = self.scopes[scope]
    //         .vars
    //         .iter()
    //         .map(|(_, (id, _, _))| *id)
    //         .collect();

    //     let idx = self.code.scope_vars.add(vars);

    //     self.set_instr(Instruction::PushVars(idx), func, enter);
    //     self.push_instr(Instruction::PopVars(idx), func);

    //     Ok(())
    // }

    pub fn run_scoped<F>(&mut self, scope: ScopeKey, func: usize, f: F) -> Result<(), CompilerError>
    where
        F: FnOnce(&mut Compiler) -> Result<(), CompilerError>,
    {
        let enter = self.push_instr(Instruction::PushVars(0), func);
        f(self)?;
        let vars = self.scopes[scope]
            .vars
            .iter()
            .map(|(_, (id, _, _))| *id)
            .collect();

        let idx = self.code.scope_vars.add(vars);

        self.set_instr(Instruction::PushVars(idx), func, enter);
        self.push_instr(Instruction::PopVars(idx), func);

        Ok(())
    }

    pub fn derived(&mut self, key: ScopeKey) -> ScopeKey {
        self.scopes.insert(Scope {
            vars: AHashMap::new(),
            parent: Some(key),
        })
    }
    pub fn get_var_data(
        &self,
        name: &String,
        scope: ScopeKey,
    ) -> Option<(InstrNum, bool, CodeSpan)> {
        let scope = &self.scopes[scope];
        match scope.vars.get(name) {
            Some((n, m, a)) => Some((*n, *m, *a)),
            None => match scope.parent {
                Some(parent) => self.get_var_data(name, parent),
                None => None,
            },
        }
    }
    pub fn new_var(
        &mut self,
        name: String,
        scope: ScopeKey,
        mutable: bool,
        span: CodeSpan,
    ) -> InstrNum {
        self.scopes[scope]
            .vars
            .insert(name, (self.code.var_count as InstrNum, mutable, span));
        self.code.var_count += 1;
        self.code.var_count as InstrNum - 1
    }
    pub fn get_accessible_vars(&self, scope: ScopeKey) -> Vec<InstrNum> {
        let mut vars = vec![];
        let mut scope = &self.scopes[scope];
        loop {
            for (_, (id, _, _)) in &scope.vars {
                vars.push(*id);
            }
            match scope.parent {
                Some(p) => scope = &self.scopes[p],
                None => return vars,
            }
        }
    }

    fn push_instr(&mut self, i: Instruction, func: usize) -> usize {
        self.code.bytecode_funcs[func].instructions.push(i);
        self.code.bytecode_funcs[func].instructions.len() - 1
    }
    fn push_instr_with_span(&mut self, i: Instruction, span: CodeSpan, func: usize) -> usize {
        let key = (func, self.func_len(func));
        self.code.bytecode_spans.insert(key, span);
        self.code.bytecode_funcs[func].instructions.push(i);
        self.code.bytecode_funcs[func].instructions.len() - 1
    }
    fn set_instr(&mut self, i: Instruction, func: usize, n: usize) {
        self.code.bytecode_funcs[func].instructions[n] = i;
    }
    fn instr_len(&mut self, func: usize) -> usize {
        self.code.bytecode_funcs[func].instructions.len()
    }
    fn func_len(&self, func: usize) -> usize {
        self.code.bytecode_funcs[func].instructions.len()
    }

    fn compile_expr(
        &mut self,
        expr_key: ExprKey,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(), CompilerError> {
        let expr = self.ast_data.get_expr(expr_key);
        let expr_span = self.ast_data.get_span(expr_key);

        let constant = |s: &mut Compiler, c| {
            let c_id = s.code.constants.add(c);
            s.push_instr_with_span(Instruction::LoadConst(c_id), expr_span, func);
        };

        match expr {
            Expression::Int(n) => constant(self, Const::Int(n)),
            Expression::Float(n) => constant(self, Const::Float(n)),
            Expression::String(s) => constant(self, Const::String(s)),
            Expression::Bool(b) => constant(self, Const::Bool(b)),
            Expression::Type(name) => {
                let v_id = self.code.names.add(name);
                self.push_instr_with_span(Instruction::LoadType(v_id), expr_span, func);
            }
            Expression::Op(a, op, b) => {
                self.compile_expr(a, scope, func)?;
                self.compile_expr(b, scope, func)?;

                macro_rules! op_helper {
                    ( $($v:ident)* ) => {
                        match op {
                            $(
                                Token::$v => self.push_instr_with_span(
                                    Instruction::$v,
                                    expr_span,
                                    func
                                ),
                            )*
                            _ => unreachable!(),
                        }
                    };
                }

                op_helper!(Plus Minus Mult Div Mod Pow Eq NotEq Greater GreaterEq Lesser LesserEq Is);
            }
            Expression::Unary(op, v) => {
                self.compile_expr(v, scope, func)?;

                let instr = match op {
                    Token::Minus => Instruction::Negate,
                    Token::ExclMark => Instruction::Not,
                    _ => unreachable!(),
                };

                self.push_instr_with_span(instr, expr_span, func);
            }
            Expression::Var(v) => {
                if let Some((id, _, _)) = self.get_var_data(&v, scope) {
                    self.push_instr(Instruction::LoadVar(id), func);
                } else {
                    return Err(CompilerError::NonexistentVar {
                        name: v,
                        area: self.make_area(expr_span),
                    });
                }
            }
            Expression::Array(v) => {
                for i in &v {
                    self.compile_expr(*i, scope, func)?;
                }
                self.push_instr_with_span(
                    Instruction::BuildArray(v.len() as InstrNum),
                    expr_span,
                    func,
                );
            }
            Expression::Empty => {
                self.push_instr_with_span(Instruction::PushEmpty, expr_span, func);
            }
            Expression::Dict(items) => {
                let key_spans = self.ast_data.dictlike_spans[expr_key].clone();
                let idx = self
                    .code
                    .name_sets
                    .add(items.iter().map(|(s, _)| s.clone()).rev().collect());
                for ((name, v), span) in items.into_iter().zip(key_spans) {
                    if let Some(v) = v {
                        self.compile_expr(v, scope, func)?;
                    } else if let Some((id, _, _)) = self.get_var_data(&name, scope) {
                        self.push_instr(Instruction::LoadVar(id), func);
                    } else {
                        return Err(CompilerError::NonexistentVar {
                            name,
                            area: self.make_area(span),
                        });
                    }
                }
                self.push_instr_with_span(Instruction::BuildDict(idx), expr_span, func);
            }
            Expression::Block(code) => {
                let derived = self.derived(scope);
                self.run_scoped(derived, func, |comp| {
                    comp.compile_stmts(code, derived, func)?;
                    Ok(())
                })?;
                self.push_instr_with_span(Instruction::PushEmpty, expr_span, func);
            }
            Expression::Func {
                args,
                ret_type,
                code,
            } => {
                let mut arg_ids = vec![];

                self.code.bytecode_funcs.push(BytecodeFunc::default());
                let func_id = self.code.bytecode_funcs.len() - 1;

                let derived_scope = self.derived(scope);

                self.macro_exec_queue
                    .last_mut()
                    .unwrap()
                    .push(MacroQueuedExec {
                        code_key: code,
                        func_id,
                        derived_scope,
                        original_scope: scope,
                    });
                // self.compile_expr(code, &mut scope.derived(), func_id)?;

                let idx = self.code.macro_build_info.add((
                    func_id,
                    args.iter()
                        .map(|(s, t, d)| (s.clone(), t.is_some(), d.is_some()))
                        .rev()
                        .collect(),
                ));

                let mut arg_spans = self.ast_data.func_arg_spans[expr_key].clone();
                for ((n, t, d), span) in args.into_iter().zip(&arg_spans) {
                    let arg_id = self.new_var(n.clone(), derived_scope, false, *span);
                    arg_ids.push(arg_id);
                    if let Some(t) = t {
                        self.compile_expr(t, scope, func)?;
                    }
                    if let Some(d) = d {
                        self.compile_expr(d, scope, func)?;
                    }
                }
                self.code.bytecode_funcs[func_id].arg_ids = arg_ids;

                if let Some(ret) = ret_type {
                    self.compile_expr(ret, scope, func)?;
                } else {
                    self.push_instr_with_span(Instruction::PushAnyPattern, expr_span, func);
                }
                arg_spans.reverse();
                self.code
                    .macro_arg_spans
                    .insert((func, self.func_len(func)), arg_spans);
                self.push_instr_with_span(Instruction::MakeMacro(idx), expr_span, func);
            }
            Expression::FuncPattern { args, ret_type } => {
                for i in &args {
                    self.compile_expr(*i, scope, func)?;
                }
                self.compile_expr(ret_type, scope, func)?;
                self.push_instr_with_span(
                    Instruction::MakeMacroPattern(args.len() as InstrNum),
                    expr_span,
                    func,
                );
            }
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => {
                self.compile_expr(cond, scope, func)?;
                let jump_if_false = self.push_instr(Instruction::JumpIfFalse(0), func);
                self.compile_expr(if_true, scope, func)?;
                let jump = self.push_instr(Instruction::Jump(0), func);

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);
                self.set_instr(Instruction::JumpIfFalse(idx), func, jump_if_false);

                self.compile_expr(if_false, scope, func)?;

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);
                self.set_instr(Instruction::Jump(idx), func, jump);
            }
            Expression::Index { base, index } => {
                self.compile_expr(base, scope, func)?;
                self.compile_expr(index, scope, func)?;

                self.push_instr(Instruction::Index, func);
            }
            Expression::Call {
                base,
                params,
                named_params,
            } => {
                let mut param_spans = self.ast_data.func_arg_spans[expr_key].clone();
                let idx = self.code.name_sets.add(
                    params
                        .iter()
                        .map(|_| "".into())
                        .chain(named_params.iter().map(|(s, _)| s.clone()))
                        .rev()
                        .collect(),
                );
                for v in params {
                    self.compile_expr(v, scope, func)?;
                }
                for (_, v) in named_params {
                    self.compile_expr(v, scope, func)?;
                }
                param_spans.reverse();
                self.compile_expr(base, scope, func)?;
                self.code
                    .macro_arg_spans
                    .insert((func, self.func_len(func)), param_spans);
                self.push_instr_with_span(Instruction::Call(idx), expr_span, func);
            }
            Expression::TriggerFuncCall(v) => {
                self.compile_expr(v, scope, func)?;
                self.push_instr(Instruction::TriggerFuncCall, func);
            }
            Expression::Maybe(expr) => {
                if let Some(expr) = expr {
                    self.compile_expr(expr, scope, func)?;
                    self.push_instr_with_span(Instruction::WrapMaybe, expr_span, func);
                } else {
                    self.push_instr_with_span(Instruction::PushNone, expr_span, func);
                }
            }
            Expression::TriggerFunc(code) => {
                // self.push_instr(Instruction::LoadArbitraryGroup, func);
                // self.push_instr(Instruction::PushTriggerFnValue, func);
                self.layers.push(ExecLayer::new_async());

                let enter = self.push_instr(Instruction::EnterTriggerFunction(0), func);

                let derived = self.derived(scope);
                self.compile_stmts(code, derived, func)?;

                self.push_instr(Instruction::YeetContext, func);
                self.layers.pop();

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);

                self.code.bytecode_spans.insert((func, enter), expr_span);
                self.set_instr(Instruction::EnterTriggerFunction(idx), func, enter);
            }
            Expression::Instance(typ, items) => {
                let key_spans = self.ast_data.dictlike_spans[expr_key].clone();
                let idx = self
                    .code
                    .name_sets
                    .add(items.iter().map(|(s, _)| s.clone()).collect());
                for ((name, v), span) in items.into_iter().zip(key_spans) {
                    if let Some(v) = v {
                        self.compile_expr(v, scope, func)?;
                    } else if let Some((id, _, _)) = self.get_var_data(&name, scope) {
                        self.push_instr(Instruction::LoadVar(id), func);
                    } else {
                        return Err(CompilerError::NonexistentVar {
                            name,
                            area: self.make_area(span),
                        });
                    }
                }
                self.compile_expr(typ, scope, func)?;
                self.push_instr(Instruction::Instance(idx), func);
            }
            Expression::Split(a, b) => {
                self.compile_expr(a, scope, func)?;
                self.compile_expr(b, scope, func)?;
                self.push_instr(Instruction::Split, func);
            }
            Expression::Obj(mode, vals) => {
                let l = vals.len() as InstrNum;
                for (k, v) in vals.into_iter() {
                    self.compile_expr(k, scope, func)?;
                    self.compile_expr(v, scope, func)?;
                }
                self.push_instr_with_span(
                    match mode {
                        ObjectMode::Object => Instruction::BuildObject(l),
                        ObjectMode::Trigger => Instruction::BuildTrigger(l),
                    },
                    expr_span,
                    func,
                );
            }
        }

        Ok(())
    }

    fn compile_stmt(
        &mut self,
        stmt_key: StmtKey,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(), CompilerError> {
        let is_arrow = self.ast_data.stmt_arrows[stmt_key];
        let stmt = self.ast_data.get_stmt(stmt_key);
        let stmt_span = self.ast_data.get_span(stmt_key);

        // if is_arrow && !matches!(&stmt, Statement::Return(_)) {
        //     self.push_instr(Instruction::SaveContexts, func);
        // }

        let enter = if is_arrow {
            self.layers.push(ExecLayer::new_async());
            Some(self.push_instr(Instruction::EnterArrowStatement(0), func))
        } else {
            None
        };

        match stmt {
            Statement::Expr(expr) => {
                self.compile_expr(expr, scope, func)?;
                self.push_instr(Instruction::PopTop, func);
            }
            Statement::Print(expr) => {
                self.compile_expr(expr, scope, func)?;
                self.push_instr(Instruction::Print, func);
            }
            Statement::Add(expr) => {
                self.compile_expr(expr, scope, func)?;
                self.push_instr(Instruction::AddObject, func);
            }
            Statement::Let(name, value) => {
                self.compile_expr(value, scope, func)?;

                let var_id = self.new_var(name, scope, true, stmt_span);

                self.push_instr(Instruction::SetVar(var_id), func);
            }
            Statement::Assign(name, value) => {
                self.compile_expr(value, scope, func)?;

                if let Some((id, mutable, span)) = self.get_var_data(&name, scope) {
                    if !mutable {
                        return Err(CompilerError::ModifyImmutable {
                            name,
                            area: self.make_area(stmt_span),
                            def_area: self.make_area(span),
                        });
                    }
                    self.push_instr(Instruction::SetVar(id), func);
                } else {
                    let var_id = self.new_var(name, scope, false, stmt_span);
                    self.push_instr(Instruction::SetVar(var_id), func);
                }
            }
            Statement::If {
                branches,
                else_branch,
            } => {
                let mut end_jumps = vec![];

                for (cond, code) in branches {
                    self.compile_expr(cond, scope, func)?;
                    let jump_idx = self.push_instr(Instruction::JumpIfFalse(0), func);

                    let derived = self.derived(scope);
                    // self.layers.push
                    self.run_scoped(derived, func, |comp| {
                        comp.compile_stmts(code, derived, func)?;
                        Ok(())
                    })?;

                    let j = self.push_instr(Instruction::Jump(0), func);
                    end_jumps.push(j);

                    let next_pos = self.instr_len(func);
                    let idx = self.code.destinations.add(next_pos);
                    self.set_instr(Instruction::JumpIfFalse(idx), func, jump_idx);
                }

                if let Some(code) = else_branch {
                    let derived = self.derived(scope);
                    self.run_scoped(derived, func, |comp| {
                        comp.compile_stmts(code, derived, func)?;
                        Ok(())
                    })?;
                }

                let next_pos = self.instr_len(func);
                for i in end_jumps {
                    let idx = self.code.destinations.add(next_pos);
                    self.set_instr(Instruction::Jump(idx), func, i);
                }
            }
            Statement::While { cond, code } => {
                self.push_instr(Instruction::PushWhile, func);
                let cond_pos = self.instr_len(func);
                self.compile_expr(cond, scope, func)?;
                let jump_pos = self.push_instr(Instruction::JumpIfFalse(0), func);

                let derived = self.derived(scope);

                self.layers.push(ExecLayer::new_loop());
                self.run_scoped(derived, func, |comp| {
                    comp.compile_stmts(code, derived, func)?;
                    Ok(())
                })?;

                let start_idx = self.code.destinations.add(cond_pos);
                self.push_instr(Instruction::Jump(start_idx), func);

                let next_pos = self.instr_len(func);
                let end_idx = self.code.destinations.add(next_pos);
                self.push_instr(Instruction::PopBlock(false), func);
                self.set_instr(Instruction::JumpIfFalse(end_idx), func, jump_pos);

                match self.layers.pop().unwrap() {
                    ExecLayer::Loop { breaks, continues } => {
                        for i in continues {
                            self.set_instr(Instruction::Jump(start_idx), func, i);
                        }
                        for i in breaks {
                            self.set_instr(Instruction::Jump(end_idx), func, i);
                        }
                    }
                    ExecLayer::Macro => unreachable!(),
                    ExecLayer::Async => unreachable!(),
                }
            }
            Statement::For {
                var,
                iterator,
                code,
            } => {
                let var_span = self.ast_data.for_loop_iter_spans[stmt_key];

                self.compile_expr(iterator, scope, func)?;
                self.push_instr_with_span(Instruction::ToIter, stmt_span, func);
                let iter_pos = self.push_instr(Instruction::IterNext(0), func);

                let derived_scope = self.derived(scope);
                let var_id = self.new_var(var, derived_scope, false, var_span);

                self.layers.push(ExecLayer::new_loop());
                self.run_scoped(derived_scope, func, |comp| {
                    comp.push_instr(Instruction::SetVar(var_id), func);
                    comp.compile_stmts(code, derived_scope, func)?;
                    Ok(())
                })?;

                let start_idx = self.code.destinations.add(iter_pos);
                self.push_instr(Instruction::Jump(start_idx), func);
                let jump_pos = self.push_instr(Instruction::PopBlock(false), func);

                let end_idx = self.code.destinations.add(jump_pos);
                self.set_instr(Instruction::IterNext(end_idx), func, iter_pos);

                match self.layers.pop().unwrap() {
                    ExecLayer::Loop { breaks, continues } => {
                        for i in continues {
                            self.set_instr(Instruction::Jump(start_idx), func, i);
                        }
                        for i in breaks {
                            self.set_instr(Instruction::Jump(end_idx), func, i);
                        }
                    }
                    ExecLayer::Macro => unreachable!(),
                    ExecLayer::Async => unreachable!(),
                }
            }
            Statement::Return(val) => {
                if !self.layers.iter().any(|l| matches!(l, ExecLayer::Macro)) {
                    return Err(CompilerError::ReturnOutsideMacro {
                        area: self.make_area(stmt_span),
                    });
                }

                if let Some(val) = val {
                    self.compile_expr(val, scope, func)?;
                } else {
                    self.push_instr_with_span(Instruction::PushEmpty, stmt_span, func);
                }
                self.push_instr(Instruction::ReturnValue(is_arrow), func);
            }
            Statement::Break => {
                self.push_instr(Instruction::PopBlock(true), func);
                let idx = self.push_instr(Instruction::Jump(0), func);
                match self.layers.last_mut() {
                    Some(ExecLayer::Loop { breaks, .. }) => {
                        breaks.push(idx);
                    }
                    _ => {
                        return Err(CompilerError::BreakOutsideLoop {
                            area: self.make_area(stmt_span),
                        })
                    }
                }
            }
            Statement::Continue => {
                let idx = self.push_instr(Instruction::Jump(0), func);
                match self.layers.last_mut() {
                    Some(ExecLayer::Loop { continues, .. }) => {
                        continues.push(idx);
                    }
                    _ => {
                        return Err(CompilerError::ContinueOutsideLoop {
                            area: self.make_area(stmt_span),
                        })
                    }
                }
            }
            Statement::TypeDef(name) => {
                let v_id = self.code.names.add(name);
                self.push_instr(Instruction::TypeDef(v_id), func);
            }
            Statement::Impl(typ, impls) => {
                let key_spans = self.ast_data.impl_spans[stmt_key].clone();
                let idx = self
                    .code
                    .name_sets
                    .add(impls.iter().map(|(s, _)| s.clone()).collect());
                for ((key, v), span) in impls.into_iter().zip(key_spans) {
                    if let Some(v) = v {
                        self.compile_expr(v, scope, func)?;
                    } else if let Some((id, _, _)) = self.get_var_data(&key, scope) {
                        self.push_instr(Instruction::LoadVar(id), func);
                    } else {
                        return Err(CompilerError::NonexistentVar {
                            name: key,
                            area: self.make_area(span),
                        });
                    }
                }
                self.compile_expr(typ, scope, func)?;
                self.push_instr(Instruction::Impl(idx), func);
            }
            Statement::TryCatch {
                try_branch,
                catch,
                catch_var,
            } => todo!(),
        }

        // if is_arrow {
        //     self.push_instr(Instruction::ReviseContexts, func);
        // }
        if let Some(enter) = enter {
            self.layers.pop();
            self.push_instr(Instruction::YeetContext, func);

            let next_pos = self.instr_len(func);
            let idx = self.code.destinations.add(next_pos);
            self.set_instr(Instruction::EnterArrowStatement(idx), func, enter)
        }
        // really this should be done every time a variable is removed (or changed?)
        self.push_instr(Instruction::MergeContexts, func);
        Ok(())
    }

    pub fn resolve_queued(&mut self) -> Result<(), CompilerError> {
        for MacroQueuedExec {
            code_key,
            func_id,
            derived_scope,
            original_scope,
        } in self.macro_exec_queue.pop().unwrap()
        {
            self.macro_exec_queue.push(vec![]);
            self.layers.push(ExecLayer::Macro);
            self.compile_expr(code_key, derived_scope, func_id)?;
            self.layers.pop();
            self.resolve_queued()?;
            self.code.bytecode_funcs[func_id].capture_ids =
                self.get_accessible_vars(original_scope);
            self.code.bytecode_funcs[func_id].scoped_var_ids = self.scopes[derived_scope].var_ids();
        }
        Ok(())
    }

    pub fn compile_stmts(
        &mut self,
        stmts: Statements,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(), CompilerError> {
        self.macro_exec_queue.push(vec![]);
        for i in stmts {
            self.compile_stmt(i, scope, func)?;
        }
        self.resolve_queued()?;
        Ok(())
    }

    pub fn start_compile(&mut self, stmts: Statements) -> Result<ScopeKey, CompilerError> {
        let base_scope = self.scopes.insert(Scope::base());
        self.code.bytecode_funcs.push(BytecodeFunc::default());

        self.run_scoped(base_scope, 0, |comp| {
            comp.compile_stmts(stmts, base_scope, 0)?;
            Ok(())
        })?;

        Ok(base_scope)
    }
}

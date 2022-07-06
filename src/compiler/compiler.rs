use std::collections::HashMap;

use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use super::error::CompilerError;
use crate::parser::lexer::Token;
use crate::parser::parser::{ASTData, ASTKey, ExprKey, Expression, Statement, Statements, StmtKey};

use crate::interpreter::value::Value;
use crate::sources::CodeArea;

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

#[derive(Serialize, Deserialize)]
pub struct Code {
    pub constants: UniqueRegister<Value>,
    pub names: UniqueRegister<String>,
    pub destinations: UniqueRegister<usize>,
    pub name_sets: UniqueRegister<Vec<String>>,
    #[allow(clippy::type_complexity)]
    pub macro_build_info: UniqueRegister<(usize, Vec<(String, bool, bool)>)>,

    pub var_count: usize,

    pub instructions: Vec<(Vec<Instruction>, Vec<u16>)>,

    pub bytecode_areas: HashMap<(usize, usize), CodeArea>,

    pub path_base_areas: HashMap<(usize, usize), CodeArea>,
}
impl Code {
    pub fn new() -> Self {
        Self {
            constants: UniqueRegister::new(),
            names: UniqueRegister::new(),
            destinations: UniqueRegister::new(),
            name_sets: UniqueRegister::new(),
            macro_build_info: UniqueRegister::new(),
            instructions: vec![],
            var_count: 0,
            bytecode_areas: HashMap::new(),
            path_base_areas: HashMap::new(),
        }
    }

    pub fn get_bytecode_area(&self, func: usize, i: usize) -> CodeArea {
        self.bytecode_areas.get(&(func, i)).unwrap().clone()
    }

    pub fn debug(&self) {
        for (i, (instrs, args)) in self.instructions.iter().enumerate() {
            println!("============================> Func {}\t{:? }", i, args);
            for (i, instr) in instrs.iter().enumerate() {
                use ansi_term::Color::Yellow;
                let instr_len = 25;

                let col = Yellow.bold();

                let instr_str = format!("{:?}", instr);
                let instr_str =
                    instr_str.clone() + &String::from(" ").repeat(instr_len - instr_str.len());

                let mut s = format!("{}\t{}", i, instr_str);

                match instr {
                    Instruction::LoadConst(idx) => {
                        s += &col
                            .paint(format!("{:?}", self.constants.get(*idx)))
                            .to_string()
                    }
                    Instruction::LoadType(idx) => {
                        s += &col.paint(format!("@{}", self.names.get(*idx))).to_string()
                    }
                    Instruction::Jump(idx) => {
                        s += &col
                            .paint(format!("to {:?}", self.destinations.get(*idx)))
                            .to_string()
                    }
                    Instruction::JumpIfFalse(idx) => {
                        s += &col
                            .paint(format!("to {:?}", self.destinations.get(*idx)))
                            .to_string()
                    }
                    Instruction::IterNext(idx) => {
                        s += &col
                            .paint(format!("to {:?}", self.destinations.get(*idx)))
                            .to_string()
                    }
                    Instruction::BuildDict(idx) => {
                        s += &col
                            .paint(format!("with {:?}", self.name_sets.get(*idx)))
                            .to_string()
                    }
                    Instruction::MakeMacro(idx) => {
                        s += &col
                            .paint(format!(
                                "Func {:?}, args: {:?}",
                                self.macro_build_info.get(*idx).0,
                                self.macro_build_info.get(*idx).1
                            ))
                            .to_string()
                    }
                    Instruction::Call(idx) => {
                        s += &col
                            .paint(format!("params: {:?}", self.name_sets.get(*idx)))
                            .to_string()
                    }
                    Instruction::TypeDef(idx) => {
                        s += &col.paint(format!("@{}", self.names.get(*idx))).to_string()
                    }
                    Instruction::Impl(idx) => {
                        s += &col
                            .paint(format!("with {:?}", self.name_sets.get(*idx)))
                            .to_string()
                    }
                    Instruction::Instance(idx) => {
                        s += &col
                            .paint(format!("with {:?}", self.name_sets.get(*idx)))
                            .to_string()
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
    LoadConst(InstrNum),

    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    Pow,

    Eq,
    NotEq,
    Greater,
    GreaterEq,
    Lesser,
    LesserEq,

    Negate,
    Not,

    LoadVar(InstrNum),
    SetVar(InstrNum),

    LoadType(InstrNum),

    BuildArray(InstrNum),

    PushEmpty,
    PopTop,

    Jump(InstrNum),
    JumpIfFalse(InstrNum),

    ToIter,
    IterNext(InstrNum),

    BuildDict(InstrNum),

    Return,
    Continue,
    Break,

    MakeMacro(InstrNum),
    PushAnyPattern,

    MakeMacroPattern(InstrNum),

    Index,
    Call(InstrNum),
    TriggerFuncCall,

    SaveContexts,
    ReviseContexts,

    MergeContexts,

    PushNone,
    WrapMaybe,

    PushContextGroup,
    PopContextGroup,
    PushTriggerFnValue,

    TypeDef(InstrNum),
    Impl(InstrNum),
    Instance(InstrNum),

    EnterScope,
    ExitScope,

    Print,
}

pub struct Scope {
    vars: AHashMap<String, (InstrNum, bool, CodeArea)>,
    parent: Option<ScopeKey>,
}
impl Scope {
    pub fn base() -> Scope {
        Self {
            vars: AHashMap::new(),
            parent: None,
        }
    }
}

new_key_type! { pub struct ScopeKey; }

pub struct Compiler {
    pub ast_data: ASTData,
    pub code: Code,
    pub code_exec_queue: Vec<Vec<(Box<dyn ASTKey>, usize, ScopeKey)>>,

    pub scopes: SlotMap<ScopeKey, Scope>,
}

impl Compiler {
    pub fn new(data: ASTData) -> Self {
        Self {
            ast_data: data,
            code: Code::new(),
            code_exec_queue: vec![],
            scopes: SlotMap::default(),
        }
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
    ) -> Option<(InstrNum, bool, CodeArea)> {
        let scope = &self.scopes[scope];
        match scope.vars.get(name) {
            Some((n, m, a)) => Some((*n, *m, a.clone())),
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
        area: CodeArea,
    ) -> InstrNum {
        self.scopes[scope]
            .vars
            .insert(name, (self.code.var_count as InstrNum, mutable, area));
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
        self.code.instructions[func].0.push(i);
        self.code.instructions[func].0.len() - 1
    }
    fn push_instr_with_area(&mut self, i: Instruction, area: CodeArea, func: usize) -> usize {
        let key = (func, self.func_len(func));
        self.code.bytecode_areas.insert(key, area);
        self.code.instructions[func].0.push(i);
        self.code.instructions[func].0.len() - 1
    }
    fn set_instr(&mut self, i: Instruction, func: usize, n: usize) {
        self.code.instructions[func].0[n] = i;
    }
    fn instr_len(&mut self, func: usize) -> usize {
        self.code.instructions[func].0.len()
    }
    fn func_len(&self, func: usize) -> usize {
        self.code.instructions[func].0.len()
    }

    fn compile_expr(
        &mut self,
        expr_key: ExprKey,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(), CompilerError> {
        let expr = self.ast_data.get_expr(expr_key);
        let expr_area = self.ast_data.get_area(expr_key).clone();

        match expr {
            Expression::Literal(l) => {
                let val = l.to_value();
                let c_id = self.code.constants.add(val);
                self.push_instr_with_area(Instruction::LoadConst(c_id), expr_area, func);
            }
            Expression::Type(name) => {
                let v_id = self.code.names.add(name);
                self.push_instr_with_area(Instruction::LoadType(v_id), expr_area, func);
            }
            Expression::Op(a, op, b) => {
                self.compile_expr(a, scope, func)?;
                self.compile_expr(b, scope, func)?;

                macro_rules! op_helper {
                    ( $($v:ident)* ) => {
                        match op {
                            $(
                                Token::$v => self.push_instr_with_area(
                                    Instruction::$v,
                                    expr_area.clone(),
                                    func
                                ),
                            )*
                            _ => unreachable!(),
                        }
                    };
                }

                op_helper!(Plus Minus Mult Div Mod Pow Eq NotEq Greater GreaterEq Lesser LesserEq);
            }
            Expression::Unary(op, v) => {
                self.compile_expr(v, scope, func)?;

                let instr = match op {
                    Token::Minus => Instruction::Negate,
                    Token::ExclMark => Instruction::Not,
                    _ => unreachable!(),
                };

                self.push_instr_with_area(instr, expr_area, func);
            }
            Expression::Var(v) => {
                if let Some((id, _, _)) = self.get_var_data(&v, scope) {
                    self.push_instr(Instruction::LoadVar(id), func);
                } else {
                    return Err(CompilerError::NonexistentVar {
                        name: v,
                        area: expr_area,
                    });
                }
            }
            Expression::Array(v) => {
                for i in &v {
                    self.compile_expr(*i, scope, func)?;
                }
                self.push_instr_with_area(
                    Instruction::BuildArray(v.len() as InstrNum),
                    expr_area,
                    func,
                );
            }
            Expression::Empty => {
                self.push_instr_with_area(Instruction::PushEmpty, expr_area, func);
            }
            Expression::Dict(items) => {
                let key_areas = self.ast_data.dictlike_areas[expr_key].clone();
                let idx = self
                    .code
                    .name_sets
                    .add(items.iter().map(|(s, _)| s.clone()).rev().collect());
                for ((name, v), area) in items.into_iter().zip(key_areas) {
                    if let Some(v) = v {
                        self.compile_expr(v, scope, func)?;
                    } else if let Some((id, _, _)) = self.get_var_data(&name, scope) {
                        self.push_instr(Instruction::LoadVar(id), func);
                    } else {
                        return Err(CompilerError::NonexistentVar { name, area });
                    }
                }
                self.push_instr_with_area(Instruction::BuildDict(idx), expr_area, func);
            }
            Expression::Block(code) => {
                let derived = self.derived(scope);
                self.push_instr(Instruction::EnterScope, func);
                self.compile_stmts(code, derived, func)?;
                self.push_instr(Instruction::ExitScope, func);
                self.push_instr(Instruction::PushEmpty, func);
            }
            Expression::Func {
                args,
                ret_type,
                code,
            } => {
                let mut arg_ids = vec![];

                self.code.instructions.push((vec![], vec![]));
                let func_id = self.code.instructions.len() - 1;

                let derived_scope = self.derived(scope);

                self.code_exec_queue.last_mut().unwrap().push((
                    Box::new(code),
                    func_id,
                    derived_scope,
                ));
                // self.compile_expr(code, &mut scope.derived(), func_id)?;

                let idx = self.code.macro_build_info.add((
                    func_id,
                    args.iter()
                        .map(|(s, t, d)| (s.clone(), t.is_some(), d.is_some()))
                        .rev()
                        .collect(),
                ));

                let arg_areas = self.ast_data.func_arg_areas[expr_key].clone();

                for ((n, t, d), area) in args.into_iter().zip(arg_areas) {
                    let arg_id = self.new_var(n.clone(), derived_scope, false, area);
                    arg_ids.push(arg_id);
                    if let Some(t) = t {
                        self.compile_expr(t, scope, func)?;
                    }
                    if let Some(d) = d {
                        self.compile_expr(d, scope, func)?;
                    }
                }
                self.code.instructions[func_id].1 = arg_ids;

                if let Some(ret) = ret_type {
                    self.compile_expr(ret, scope, func)?;
                } else {
                    self.push_instr_with_area(Instruction::PushAnyPattern, expr_area.clone(), func);
                }

                self.push_instr_with_area(Instruction::MakeMacro(idx), expr_area, func);
            }
            Expression::FuncPattern { args, ret_type } => {
                for i in &args {
                    self.compile_expr(*i, scope, func)?;
                }
                self.compile_expr(ret_type, scope, func)?;
                self.push_instr(
                    Instruction::MakeMacroPattern(args.len() as InstrNum + 1),
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
                self.compile_expr(base, scope, func)?;
                self.code.path_base_areas.insert(
                    (func, self.func_len(func) + 1),
                    self.ast_data.get_area(base).clone(),
                );
                self.push_instr_with_area(Instruction::Call(idx), expr_area, func);
            }
            Expression::TriggerFuncCall(v) => {
                self.compile_expr(v, scope, func)?;
                self.push_instr(Instruction::TriggerFuncCall, func);
            }
            Expression::Maybe(expr) => {
                if let Some(expr) = expr {
                    self.compile_expr(expr, scope, func)?;
                    self.push_instr(Instruction::WrapMaybe, func);
                } else {
                    self.push_instr(Instruction::PushNone, func);
                }
            }
            Expression::TriggerFunc(code) => {
                self.push_instr(Instruction::PushContextGroup, func);
                let derived = self.derived(scope);
                self.push_instr(Instruction::EnterScope, func);
                self.compile_stmts(code, derived, func)?;
                self.push_instr(Instruction::ExitScope, func);
                self.push_instr(Instruction::PopContextGroup, func);
                self.push_instr(Instruction::PushTriggerFnValue, func);
            }
            Expression::Instance(typ, items) => {
                let key_areas = self.ast_data.dictlike_areas[expr_key].clone();
                let idx = self
                    .code
                    .name_sets
                    .add(items.iter().map(|(s, _)| s.clone()).collect());
                for ((name, v), area) in items.into_iter().zip(key_areas) {
                    if let Some(v) = v {
                        self.compile_expr(v, scope, func)?;
                    } else if let Some((id, _, _)) = self.get_var_data(&name, scope) {
                        self.push_instr(Instruction::LoadVar(id), func);
                    } else {
                        return Err(CompilerError::NonexistentVar { name, area });
                    }
                }
                self.compile_expr(typ, scope, func)?;
                self.push_instr(Instruction::Instance(idx), func);
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
        let stmt_area = self.ast_data.get_area(stmt_key).clone();

        if is_arrow {
            self.push_instr(Instruction::SaveContexts, func);
        }

        match stmt {
            Statement::Expr(expr) => {
                self.compile_expr(expr, scope, func)?;
                self.push_instr(Instruction::PopTop, func);
            }
            Statement::Print(expr) => {
                self.compile_expr(expr, scope, func)?;
                self.push_instr(Instruction::Print, func);
            }
            Statement::Let(name, value) => {
                self.compile_expr(value, scope, func)?;

                let var_id = self.new_var(name, scope, true, stmt_area);

                self.push_instr(Instruction::SetVar(var_id), func);
            }
            Statement::Assign(name, value) => {
                self.compile_expr(value, scope, func)?;

                if let Some((id, mutable, area)) = self.get_var_data(&name, scope) {
                    if !mutable {
                        return Err(CompilerError::ModifyImmutable {
                            name,
                            area: stmt_area,
                            def_area: area,
                        });
                    }
                    self.push_instr(Instruction::SetVar(id), func);
                } else {
                    let var_id = self.new_var(name, scope, false, stmt_area);
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
                    self.push_instr(Instruction::EnterScope, func);
                    self.compile_stmts(code, derived, func)?;
                    self.push_instr(Instruction::ExitScope, func);

                    let j = self.push_instr(Instruction::Jump(0), func);
                    end_jumps.push(j);

                    let next_pos = self.instr_len(func);
                    let idx = self.code.destinations.add(next_pos);
                    self.set_instr(Instruction::JumpIfFalse(idx), func, jump_idx);
                }

                if let Some(code) = else_branch {
                    let derived = self.derived(scope);
                    self.push_instr(Instruction::EnterScope, func);
                    self.compile_stmts(code, derived, func)?;
                    self.push_instr(Instruction::ExitScope, func);
                }

                let next_pos = self.instr_len(func);
                for i in end_jumps {
                    let idx = self.code.destinations.add(next_pos);
                    self.set_instr(Instruction::Jump(idx), func, i);
                }
            }
            Statement::While { cond, code } => {
                let cond_pos = self.instr_len(func);
                self.compile_expr(cond, scope, func)?;
                let jump_pos = self.push_instr(Instruction::JumpIfFalse(0), func);

                let derived = self.derived(scope);
                self.push_instr(Instruction::EnterScope, func);
                self.compile_stmts(code, derived, func)?;
                self.push_instr(Instruction::ExitScope, func);

                let idx = self.code.destinations.add(cond_pos);
                self.push_instr(Instruction::Jump(idx), func);

                let next_pos = self.instr_len(func);
                let idx = self.code.destinations.add(next_pos);
                self.set_instr(Instruction::JumpIfFalse(idx), func, jump_pos);
            }
            Statement::For {
                var,
                iterator,
                code,
            } => {
                let var_area = self.ast_data.for_loop_iter_areas[stmt_key].clone();

                self.compile_expr(iterator, scope, func)?;
                self.push_instr(Instruction::ToIter, func);
                let iter_pos = self.push_instr(Instruction::IterNext(0), func);

                let derived_scope = self.derived(scope);
                let var_id = self.new_var(var, derived_scope, false, var_area);

                self.push_instr(Instruction::EnterScope, func);
                self.push_instr(Instruction::SetVar(var_id), func);
                self.compile_stmts(code, derived_scope, func)?;
                self.push_instr(Instruction::ExitScope, func);

                let idx = self.code.destinations.add(iter_pos);
                let jump_pos = self.push_instr(Instruction::Jump(idx), func);

                let idx = self.code.destinations.add(jump_pos + 1);
                self.set_instr(Instruction::IterNext(idx), func, iter_pos);
            }
            Statement::Return(val) => {
                if let Some(val) = val {
                    self.compile_expr(val, scope, func)?;
                } else {
                    self.push_instr(Instruction::PushEmpty, func);
                }
                self.push_instr(Instruction::Return, func);
            }
            Statement::Break => {
                self.push_instr(Instruction::Break, func);
            }
            Statement::Continue => {
                self.push_instr(Instruction::Continue, func);
            }
            Statement::TypeDef(name) => {
                let v_id = self.code.names.add(name);
                self.push_instr(Instruction::TypeDef(v_id), func);
            }
            Statement::Impl(typ, impls) => {
                let idx = self
                    .code
                    .name_sets
                    .add(impls.iter().map(|(s, _)| s.clone()).collect());
                for (_, v) in impls {
                    self.compile_expr(v, scope, func)?;
                }
                self.compile_expr(typ, scope, func)?;
                self.push_instr(Instruction::Impl(idx), func);
            }
        }

        if is_arrow {
            self.push_instr(Instruction::ReviseContexts, func);
        }

        self.push_instr(Instruction::MergeContexts, func);
        Ok(())
    }

    pub fn compile_stmts(
        &mut self,
        stmts: Statements,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(), CompilerError> {
        self.code_exec_queue.push(vec![]);
        for i in stmts {
            self.compile_stmt(i, scope, func)?;
        }
        for (k, func_id, scope) in self.code_exec_queue.pop().unwrap() {
            unsafe {
                match k.into_key() {
                    crate::parser::parser::KeyType::Expr(k) => {
                        self.compile_expr(k, scope, func_id)?
                    }
                    crate::parser::parser::KeyType::StmtKey(k) => {
                        self.compile_stmt(k, scope, func_id)?
                    }
                }
            }
        }
        Ok(())
    }
}

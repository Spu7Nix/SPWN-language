use std::collections::HashMap;

use slotmap::{new_key_type, SlotMap};

use crate::{
    parsing::{
        ast::{ASTData, ASTInsert, ExprKey, Expression, Statement, StmtKey},
        lexer::Token,
    },
    sources::{CodeSpan, SpwnSource},
    vm::value::Value,
};

use super::{
    code::{
        BytecodeFunc, Code, ConstID, InstrNum, InstrPos, Instruction, KeysID, MacroBuildID, VarID,
    },
    error::CompilerError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl Constant {
    pub fn to_value(&self) -> Value {
        match self {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::String(v) => Value::String(v.clone()),
        }
    }
}

new_key_type! {
    pub struct ScopeKey;
}

#[derive(Debug, Clone)]
pub struct VarData {
    id: VarID,
    mutable: bool,
    def_span: CodeSpan,
}

pub struct Scope {
    pub vars: HashMap<String, VarData>,
    pub parent: Option<ScopeKey>,
}

pub struct URegister<T> {
    pub reg: Vec<T>,
}

impl<T> URegister<T> {
    pub fn new() -> Self {
        Self { reg: vec![] }
    }
}

impl<T: PartialEq> URegister<T> {
    pub fn insert(&mut self, value: T) -> usize {
        match self.reg.iter().position(|v| v == &value) {
            Some(id) => id,
            None => {
                self.reg.push(value);
                self.reg.len() - 1
            }
        }
    }
}

pub struct Compiler {
    pub ast_data: ASTData,
    pub code: Code,

    pub scopes: SlotMap<ScopeKey, Scope>,
}

impl Compiler {
    pub fn new(ast_data: ASTData, source: SpwnSource) -> Self {
        Self {
            ast_data,
            code: Code::new(source),
            scopes: SlotMap::default(),
        }
    }

    pub fn push_instr(&mut self, instr: Instruction, span: CodeSpan, func: usize) -> InstrPos {
        let instrs = &mut self.code.funcs[func].instructions;
        instrs.push((instr, span));
        InstrPos {
            func,
            idx: instrs.len() - 1,
        }
    }
    pub fn get_instr(&mut self, pos: InstrPos) -> &mut Instruction {
        &mut self.code.funcs[pos.func].instructions[pos.idx].0
    }

    pub fn func_len(&self, func: usize) -> usize {
        self.code.funcs[func].instructions.len()
    }

    pub fn get_var(&self, name: &str, scope_key: ScopeKey) -> Option<VarData> {
        let mut scope = &self.scopes[scope_key];
        loop {
            match scope.vars.get(name) {
                Some(data) => return Some(data.clone()),
                None => match scope.parent {
                    Some(p) => scope = &self.scopes[p],
                    None => return None,
                },
            }
        }
    }
    pub fn new_var(
        &mut self,
        name: &str,
        scope: ScopeKey,
        mutable: bool,
        def_span: CodeSpan,
    ) -> VarID {
        let id = VarID(self.code.var_count as u16);
        self.code.var_count += 1;
        self.scopes[scope].vars.insert(
            name.to_string(),
            VarData {
                id,
                mutable,
                def_span,
            },
        );
        id
    }
    pub fn get_accessible_vars(&self, scope: ScopeKey) -> Vec<VarID> {
        let mut vars = vec![];
        let mut scope = &self.scopes[scope];
        loop {
            for (_, VarData { id, .. }) in &scope.vars {
                vars.push(*id);
            }
            match scope.parent {
                Some(p) => scope = &self.scopes[p],
                None => return vars,
            }
        }
    }

    pub fn derive_scope(&mut self, scope: ScopeKey) -> ScopeKey {
        self.scopes.insert(Scope {
            vars: HashMap::new(),
            parent: Some(scope),
        })
    }

    pub fn compile_expr(
        &mut self,
        expr_key: ExprKey,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(usize, usize), CompilerError> {
        // the `Ok` contains the idx of the first instruction and the next idx after the last
        let start_idx = self.func_len(func);

        let expr = self.ast_data.get(expr_key);
        let span = self.ast_data.span(expr_key);

        match expr {
            Expression::Int(v) => {
                let id = self.code.const_register.insert(Constant::Int(v));
                self.push_instr(Instruction::LoadConst(ConstID(id as u16)), span, func);
            }
            Expression::Float(v) => {
                let id = self.code.const_register.insert(Constant::Float(v));
                self.push_instr(Instruction::LoadConst(ConstID(id as u16)), span, func);
            }
            Expression::String(v) => {
                let id = self.code.const_register.insert(Constant::String(v));
                self.push_instr(Instruction::LoadConst(ConstID(id as u16)), span, func);
            }
            Expression::Bool(v) => {
                let id = self.code.const_register.insert(Constant::Bool(v));
                self.push_instr(Instruction::LoadConst(ConstID(id as u16)), span, func);
            }
            Expression::Id { class, value } => todo!(),
            Expression::Op(a, op, b) => {
                self.compile_expr(a, scope, func)?;
                self.compile_expr(b, scope, func)?;
                match op {
                    Token::Plus => self.push_instr(Instruction::Plus, span, func),
                    Token::Minus => self.push_instr(Instruction::Minus, span, func),
                    Token::Mult => self.push_instr(Instruction::Mult, span, func),
                    Token::Div => self.push_instr(Instruction::Div, span, func),
                    Token::Mod => self.push_instr(Instruction::Modulo, span, func),
                    Token::Pow => self.push_instr(Instruction::Pow, span, func),

                    Token::Eq => self.push_instr(Instruction::Eq, span, func),
                    Token::Neq => self.push_instr(Instruction::Neq, span, func),
                    Token::Gt => self.push_instr(Instruction::Gt, span, func),
                    Token::Gte => self.push_instr(Instruction::Gte, span, func),
                    Token::Lt => self.push_instr(Instruction::Lt, span, func),
                    Token::Lte => self.push_instr(Instruction::Lte, span, func),
                    _ => unreachable!(),
                };
            }
            Expression::Unary(op, v) => {
                self.compile_expr(v, scope, func)?;
                match op {
                    Token::Minus => self.push_instr(Instruction::Negate, span, func),
                    Token::ExclMark => self.push_instr(Instruction::Not, span, func),
                    _ => unreachable!(),
                };
            }
            Expression::Var(name) => match self.get_var(&name, scope) {
                Some(data) => {
                    self.push_instr(Instruction::LoadVar(data.id), span, func);
                }
                None => {
                    return Err(CompilerError::NonexistentVariable {
                        name,
                        area: self.ast_data.source.area(span),
                    })
                }
            },
            Expression::Type(_) => todo!(),
            Expression::Array(arr) => {
                for i in &arr {
                    self.compile_expr(*i, scope, func)?;
                }
                self.push_instr(
                    Instruction::BuildArray(InstrNum(arr.len() as u16)),
                    span,
                    func,
                );
            }
            Expression::Dict(dict) => {
                let keys = self
                    .code
                    .keys_register
                    .insert(dict.iter().map(|(s, _)| s.clone()).rev().collect());
                let key_spans = self.ast_data.dictlike_spans[expr_key].clone();
                for ((name, v), span) in dict.into_iter().zip(key_spans) {
                    match v {
                        Some(e) => {
                            self.compile_expr(e, scope, func)?;
                        }
                        None => match self.get_var(&name, scope) {
                            Some(data) => {
                                self.push_instr(Instruction::LoadVar(data.id), span, func);
                            }
                            None => {
                                return Err(CompilerError::NonexistentVariable {
                                    name,
                                    area: self.ast_data.source.area(span),
                                })
                            }
                        },
                    }
                }
                self.push_instr(Instruction::BuildDict(KeysID(keys as u16)), span, func);
            }
            Expression::Empty => {
                self.push_instr(Instruction::PushEmpty, span, func);
            }
            Expression::Block(stmts) => {
                let derived = self.derive_scope(scope);
                self.compile_stmts(stmts, derived, func)?;
                self.push_instr(Instruction::PushEmpty, span, func);
            }
            Expression::Macro {
                args,
                ret_type,
                code,
            } => {
                self.code.funcs.push(BytecodeFunc {
                    instructions: vec![],
                    arg_ids: vec![],
                    capture_ids: self.get_accessible_vars(scope),
                });
                let func_id = self.code.funcs.len() - 1;

                let info = self.code.macro_build_register.insert((
                    func_id,
                    args.iter()
                        .map(|(s, t, d)| (s.clone(), t.is_some(), d.is_some()))
                        .collect(),
                ));

                let derived = self.derive_scope(scope);

                let arg_spans = self.ast_data.func_arg_spans[expr_key].clone();
                let mut arg_ids = vec![];
                for ((name, t, d), span) in args.into_iter().zip(arg_spans) {
                    let arg_id = self.new_var(&name, derived, false, span);
                    arg_ids.push(arg_id);
                    if let Some(t) = t {
                        self.compile_expr(t, scope, func)?;
                    }
                    if let Some(d) = d {
                        self.compile_expr(d, scope, func)?;
                    }
                }
                self.code.funcs[func_id].arg_ids = arg_ids;
                if let Some(ret) = ret_type {
                    self.compile_expr(ret, scope, func)?;
                } else {
                    self.push_instr(Instruction::PushAnyPattern, span, func);
                }

                self.compile_expr(code, derived, func_id)?;
                self.push_instr(Instruction::Return, span, func_id);

                self.push_instr(
                    Instruction::BuildMacro(MacroBuildID(info as u16)),
                    span,
                    func,
                );
            }
            Expression::MacroPattern { args, ret_type } => todo!(),
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => {
                self.compile_expr(cond, scope, func)?;
                let jump = self.push_instr(Instruction::JumpIfFalse(InstrNum(0)), span, func);
                self.compile_expr(if_true, scope, func)?;
                let skip = self.push_instr(Instruction::Jump(InstrNum(0)), span, func);
                self.get_instr(jump).modify_num((skip.idx + 1) as u16);
                let (_, to) = self.compile_expr(if_false, scope, func)?;
                self.get_instr(skip).modify_num(to as u16);
            }
            Expression::Index { base, index } => {
                self.compile_expr(base, scope, func)?;
                self.compile_expr(index, scope, func)?;
                self.push_instr(Instruction::Index, span, func);
            }
            Expression::Call {
                base,
                params,
                named_params,
            } => {
                for i in &params {
                    self.compile_expr(*i, scope, func)?;
                }
                self.compile_expr(base, scope, func)?;
                self.push_instr(Instruction::Call(InstrNum(params.len() as u16)), span, func);
            }
            Expression::TriggerFuncCall(e) => {
                self.compile_expr(e, scope, func)?;
                self.push_instr(Instruction::TriggerFuncCall, span, func);
            }
            Expression::Maybe(None) => {
                self.push_instr(Instruction::PushNone, span, func);
            }
            Expression::Maybe(Some(e)) => {
                self.compile_expr(e, scope, func)?;
                self.push_instr(Instruction::WrapMaybe, span, func);
            }
            Expression::TriggerFunc(stmts) => {
                let derived = self.derive_scope(scope);
                self.compile_stmts(stmts, derived, func)?;
                self.push_instr(Instruction::PushTriggerFn, span, func);
            }
            Expression::Instance(_, _) => todo!(),
            Expression::Split(_, _) => todo!(),
        }
        Ok((start_idx, self.func_len(func)))
    }

    pub fn compile_stmt(
        &mut self,
        stmt_key: StmtKey,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(usize, usize), CompilerError> {
        let start_idx = self.func_len(func);

        let stmt = self.ast_data.get(stmt_key);
        let span = self.ast_data.span(stmt_key);

        match stmt {
            Statement::Expr(e) => {
                self.compile_expr(e, scope, func)?;
                self.push_instr(Instruction::PopTop, span, func);
            }
            Statement::Let(name, value) => {
                let var_id = self.new_var(&name, scope, true, span);
                self.compile_expr(value, scope, func)?;
                self.push_instr(Instruction::SetVar(var_id), span, func);
            }
            Statement::Assign(name, value) => {
                if let Some(VarData {
                    id,
                    mutable,
                    def_span,
                }) = self.get_var(&name, scope)
                {
                    if !mutable {
                        return Err(CompilerError::ModifyImmutable {
                            name,
                            area: self.ast_data.source.area(span),
                            def_area: self.ast_data.source.area(def_span),
                        });
                    }
                    self.compile_expr(value, scope, func)?;
                    self.push_instr(Instruction::SetVar(id), span, func);
                } else {
                    let var_id = self.new_var(&name, scope, false, span);
                    self.compile_expr(value, scope, func)?;
                    self.push_instr(Instruction::SetVar(var_id), span, func);
                }
            }
            Statement::If {
                branches,
                else_branch,
            } => {
                let mut skips = vec![];
                for (cond, stmts) in branches {
                    self.compile_expr(cond, scope, func)?;
                    let jump = self.push_instr(Instruction::JumpIfFalse(InstrNum(0)), span, func);
                    let derived = self.derive_scope(scope);
                    self.compile_stmts(stmts, derived, func)?;
                    let skip = self.push_instr(Instruction::Jump(InstrNum(0)), span, func);
                    skips.push(skip);
                    self.get_instr(jump).modify_num((skip.idx + 1) as u16);
                }
                if let Some(stmts) = else_branch {
                    let derived = self.derive_scope(scope);
                    self.compile_stmts(stmts, derived, func)?;
                }
                let skip_to = self.func_len(func);
                for i in skips {
                    self.get_instr(i).modify_num(skip_to as u16)
                }
            }
            Statement::TryCatch {
                try_branch,
                catch,
                catch_var,
            } => todo!(),
            Statement::While { cond, code } => {
                let (cond_pos, _) = self.compile_expr(cond, scope, func)?;
                let jump = self.push_instr(Instruction::JumpIfFalse(InstrNum(0)), span, func);

                let derived = self.derive_scope(scope);
                self.compile_stmts(code, derived, func)?;
                let back =
                    self.push_instr(Instruction::Jump(InstrNum(cond_pos as u16)), span, func);

                self.get_instr(jump).modify_num((back.idx + 1) as u16);
            }
            Statement::For {
                var,
                iterator,
                code,
            } => {
                self.compile_expr(iterator, scope, func)?;
                self.push_instr(Instruction::ToIter, span, func);
                let iter_pos = self.push_instr(Instruction::IterNext(InstrNum(0)), span, func);

                let derived = self.derive_scope(scope);

                let var_id = self.new_var(&var, derived, false, span);
                self.push_instr(Instruction::SetVar(var_id), span, func);
                self.compile_stmts(code, derived, func)?;
                let back =
                    self.push_instr(Instruction::Jump(InstrNum(iter_pos.idx as u16)), span, func);

                self.get_instr(iter_pos).modify_num((back.idx + 1) as u16);
            }
            Statement::Return(a) => {
                if let Some(e) = a {
                    self.compile_expr(e, scope, func)?;
                } else {
                    self.push_instr(Instruction::PushEmpty, span, func);
                }
                self.push_instr(Instruction::Return, span, func);
            }
            Statement::Break => todo!(),
            Statement::Continue => todo!(),
            Statement::TypeDef(_) => todo!(),
            Statement::Impl(typ, dict) => {
                let keys = self
                    .code
                    .keys_register
                    .insert(dict.iter().map(|(s, _)| s.clone()).rev().collect());
                let key_spans = self.ast_data.impl_spans[stmt_key].clone();
                for ((name, v), span) in dict.into_iter().zip(key_spans) {
                    match v {
                        Some(e) => {
                            self.compile_expr(e, scope, func)?;
                        }
                        None => match self.get_var(&name, scope) {
                            Some(data) => {
                                self.push_instr(Instruction::LoadVar(data.id), span, func);
                            }
                            None => {
                                return Err(CompilerError::NonexistentVariable {
                                    name,
                                    area: self.ast_data.source.area(span),
                                })
                            }
                        },
                    }
                }
                self.compile_expr(typ, scope, func)?;
                self.push_instr(Instruction::Impl(KeysID(keys as u16)), span, func);
            }
            Statement::Print(v) => {
                self.compile_expr(v, scope, func)?;
                self.push_instr(Instruction::Print, span, func);
            }
            Statement::Add(_) => todo!(),
        }
        Ok((start_idx, self.func_len(func)))
    }

    pub fn compile_stmts(
        &mut self,
        stmts: Vec<StmtKey>,
        scope: ScopeKey,
        func: usize,
    ) -> Result<(usize, usize), CompilerError> {
        let start_idx = self.func_len(func);
        for i in stmts {
            self.compile_stmt(i, scope, func)?;
        }
        Ok((start_idx, self.func_len(func)))
    }
    pub fn start_compile(&mut self, stmts: Vec<StmtKey>) -> Result<(), CompilerError> {
        let base_scope = self.scopes.insert(Scope {
            vars: HashMap::new(),
            parent: None,
        });
        self.code.funcs.push(BytecodeFunc {
            instructions: vec![],
            arg_ids: vec![],
            capture_ids: vec![],
        });
        self.compile_stmts(stmts, base_scope, 0)?;
        Ok(())
    }
}

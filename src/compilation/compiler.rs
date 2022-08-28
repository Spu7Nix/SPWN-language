use std::fs;
use std::{collections::HashMap, path::PathBuf};

use ahash::AHashMap;
use slotmap::{new_key_type, SlotMap};

use super::{
    code::{
        BytecodeFunc, Code, ConstID, InstrNum, InstrPos, Instruction, KeysID, MacroBuildID,
        MemberID, VarID,
    },
    error::CompilerError,
};
use crate::{
    leveldata::object_data::ObjectMode,
    parsing::{
        ast::{ASTData, ASTInsert, ExprKey, Expression, MacroCode, Statement, StmtKey},
        lexer::Token,
    },
    sources::{CodeSpan, SpwnSource},
    vm::{interpreter::TypeKey, types::CustomType, value::Value},
};
use crate::{parsing::ast::ImportType, vm::value::ValueType};

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Type(ValueType),
}

impl Constant {
    pub fn to_value(&self) -> Value {
        match self {
            Constant::Int(v) => Value::Int(*v),
            Constant::Float(v) => Value::Float(*v),
            Constant::Bool(v) => Value::Bool(*v),
            Constant::String(v) => Value::String(v.clone()),
            Constant::Type(k) => Value::Type(*k),
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
    pub children: Vec<ScopeKey>,
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
    pub types: SlotMap<TypeKey, CustomType>,
    pub type_keys: AHashMap<String, TypeKey>,
}

impl Compiler {
    pub fn new(ast_data: ASTData, source: SpwnSource) -> Self {
        Self {
            ast_data,
            code: Code::new(source.clone()),
            scopes: SlotMap::default(),
            types: SlotMap::default(),
            type_keys: AHashMap::default(),
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

    pub fn get_inner_vars(&self, scope: ScopeKey) -> Vec<VarID> {
        let mut vars = vec![];
        fn inner(compiler: &Compiler, scope: ScopeKey, vars: &mut Vec<VarID>) {
            for (_, VarData { id, .. }) in &compiler.scopes[scope].vars {
                vars.push(*id);
            }
            for i in &compiler.scopes[scope].children {
                inner(compiler, *i, vars);
            }
        }
        inner(self, scope, &mut vars);
        vars
    }

    pub fn derive_scope(&mut self, scope: ScopeKey) -> ScopeKey {
        let child = self.scopes.insert(Scope {
            vars: HashMap::new(),
            parent: Some(scope),
            children: vec![],
        });
        self.scopes[scope].children.push(child);
        child
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

        match expr.t {
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
            Expression::Type(name) => {
                if let Some(key) = self.type_keys.get(&name) {
                    let id = self
                        .code
                        .const_register
                        .insert(Constant::Type(ValueType::Custom(*key)));
                    self.push_instr(Instruction::LoadConst(ConstID(id as u16)), span, func);
                } else {
                    return Err(CompilerError::UndefinedType {
                        name,
                        area: self.ast_data.source.area(span),
                    });
                }
            }
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
                    .insert(dict.iter().map(|s| s.0.clone()).rev().collect());

                for kvs in dict.into_iter() {
                    match kvs.1 {
                        Some(e) => {
                            self.compile_expr(e, scope, func)?;
                        }
                        None => match self.get_var(&kvs.0, scope) {
                            Some(data) => {
                                self.push_instr(Instruction::LoadVar(data.id), span, func);
                            }
                            None => {
                                return Err(CompilerError::NonexistentVariable {
                                    name: kvs.0.clone(),
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
            Expression::Macro {
                args,
                ret_type,
                code,
            } => {
                self.code.funcs.push(BytecodeFunc {
                    instructions: vec![],
                    arg_ids: vec![],
                    capture_ids: self.get_accessible_vars(scope),
                    inner_ids: vec![],
                });

                let func_id = self.code.funcs.len() - 1;

                let info = self.code.macro_build_register.insert((
                    func_id,
                    args.iter()
                        .map(|a| (a.0.clone(), a.1.is_some(), a.2.is_some()))
                        .collect(),
                ));

                let derived = self.derive_scope(scope);

                let mut arg_ids = vec![];
                for arg in args.into_iter() {
                    let arg_id = self.new_var(&arg.0, derived, false, arg.span);
                    arg_ids.push(arg_id);

                    if let Some(t) = arg.1 {
                        self.compile_expr(t, scope, func)?;
                    }

                    if let Some(d) = arg.2 {
                        self.compile_expr(d, scope, func)?;
                    }
                }

                self.code.funcs[func_id].arg_ids = arg_ids;

                if let Some(ret) = ret_type {
                    self.compile_expr(ret, scope, func)?;
                } else {
                    self.push_instr(Instruction::PushAnyPattern, span, func);
                }

                match code {
                    MacroCode::Normal(stmts) => {
                        self.compile_stmts(stmts, derived, func_id, false)?;
                    }
                    MacroCode::Lambda(expr) => {
                        self.compile_expr(expr, derived, func_id)?;
                        self.push_instr(Instruction::Return, span, func_id);
                    }
                }
                self.code.funcs[func_id].inner_ids = self.get_inner_vars(derived);

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

            Expression::Member { base, name } => {
                self.compile_expr(base, scope, func)?;
                let id = self.code.member_register.insert(name);
                self.push_instr(Instruction::Member(MemberID(id as u16)), span, func);
            }

            Expression::TypeOf { base } => {
                self.compile_expr(base, scope, func)?;
                self.push_instr(Instruction::TypeOf, span, func);
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
                let enter =
                    self.push_instr(Instruction::EnterTriggerFunction(InstrNum(0)), span, func);

                let derived = self.derive_scope(scope);
                self.compile_stmts(stmts, derived, func, false)?;

                let to = self.push_instr(Instruction::YeetContext, span, func);
                self.get_instr(enter).modify_num((to.idx + 1) as u16);
            }
            Expression::Instance(t, fields) => {
                self.compile_expr(fields, scope, func)?;
                self.compile_expr(t, scope, func)?;

                self.push_instr(Instruction::BuildInstance, span, func);
            }
            Expression::Split(..) => todo!(),
            Expression::Obj(mode, vals) => {
                let l = InstrNum(vals.len() as u16);
                for val in vals.into_iter() {
                    self.compile_expr(val.0, scope, func)?;
                    self.compile_expr(val.1, scope, func)?;
                }
                self.push_instr(
                    match mode {
                        ObjectMode::Object => Instruction::BuildObject(l),
                        ObjectMode::Trigger => Instruction::BuildTrigger(l),
                    },
                    span,
                    func,
                );
            }
            Expression::Builtins => {
                self.push_instr(Instruction::PushBuiltins, span, func);
            }
            Expression::Import(ImportType::Module(m)) => {
                // parse & compile the module
                // put it into a function??
                // call the function???
                // profit?????

                let path = PathBuf::from(m);
                let content = fs::read_to_string(&path).unwrap();

                // parse content
                let source = SpwnSource::File(path);
                let mut parser = crate::parsing::parser::Parser::new(&content, source.clone());
                let mut ast_data = ASTData::new(source);
                let statements = match parser.parse(&mut ast_data) {
                    Ok(a) => a,
                    Err(e) => todo!(),
                };

                todo!()
            }

            Expression::Import(ImportType::Library(_)) => todo!(),
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

        let has_arrow = self.ast_data.stmt_arrows[stmt_key];

        let arrow_jump = if has_arrow {
            Some(self.push_instr(Instruction::EnterArrowStatement(InstrNum(0)), span, func))
        } else {
            None
        };

        match stmt.t {
            Statement::Expr(e) => {
                self.compile_expr(e, scope, func)?;
                self.push_instr(Instruction::PopTop, span, func);
            }
            Statement::Let(name, value) => {
                let var_id = self.new_var(&name, scope, true, span);
                self.compile_expr(value, scope, func)?;
                self.push_instr(Instruction::CreateVar(var_id), span, func);
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
                    self.push_instr(Instruction::CreateVar(var_id), span, func);
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
                    self.compile_stmts(stmts, derived, func, false)?;
                    let skip = self.push_instr(Instruction::Jump(InstrNum(0)), span, func);
                    skips.push(skip);
                    self.get_instr(jump).modify_num((skip.idx + 1) as u16);
                }
                if let Some(stmts) = else_branch {
                    let derived = self.derive_scope(scope);
                    self.compile_stmts(stmts, derived, func, false)?;
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
                self.compile_stmts(code, derived, func, false)?;
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
                self.compile_stmts(code, derived, func, false)?;
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
            Statement::TypeDef(_) => {
                //already done
            }
            Statement::Impl(typ, dict) => {
                self.compile_expr(dict, scope, func)?;
                self.compile_expr(typ, scope, func)?;

                self.push_instr(Instruction::Impl, span, func);
            }
            Statement::Print(v) => {
                self.compile_expr(v, scope, func)?;
                self.push_instr(Instruction::Print, span, func);
            }
            Statement::Add(v) => {
                self.compile_expr(v, scope, func)?;
                self.push_instr(Instruction::AddObject, span, func);
            }
        }

        if let Some(arrow_jump) = arrow_jump {
            let to = self.push_instr(Instruction::YeetContext, span, func);
            self.get_instr(arrow_jump).modify_num((to.idx + 1) as u16);
        }

        Ok((start_idx, self.func_len(func)))
    }

    pub fn compile_stmts(
        &mut self,
        stmts: Vec<StmtKey>,
        scope: ScopeKey,
        func: usize,
        allow_type_def: bool,
    ) -> Result<(usize, usize), CompilerError> {
        let start_idx = self.func_len(func);

        for i in stmts.iter() {
            if let Statement::TypeDef(name) = self.ast_data.get(*i).t {
                if allow_type_def {
                    let k = self.types.insert(CustomType {
                        name: name.clone(),
                        //members: AHashMap::default(),
                    });
                    self.type_keys.insert(name, k);
                } else {
                    let span = self.ast_data.span(*i);
                    return Err(CompilerError::LowerLevelTypeDef {
                        name,
                        area: self.code.source.area(span),
                    });
                }
            }
        }

        for i in stmts {
            self.compile_stmt(i, scope, func)?;
        }
        Ok((start_idx, self.func_len(func)))
    }

    pub fn start_compile(&mut self, stmts: Vec<StmtKey>) -> Result<(), CompilerError> {
        let base_scope = self.scopes.insert(Scope {
            vars: HashMap::new(),
            parent: None,
            children: vec![],
        });
        self.code.funcs.push(BytecodeFunc {
            instructions: vec![],
            arg_ids: vec![],
            capture_ids: vec![],
            inner_ids: vec![],
        });
        self.compile_stmts(stmts, base_scope, 0, true)?;
        Ok(())
    }
}

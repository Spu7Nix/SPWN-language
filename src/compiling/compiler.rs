use std::{path::PathBuf, rc::Rc};

use ahash::AHashMap;
use lasso::{Rodeo, Spur};
use slotmap::{new_key_type, SlotMap};

use crate::{
    parsing::{
        ast::{ExprNode, Expression, ImportType, Spanned, Statement, StmtNode},
        parser::Parser,
        utils::operators::{BinOp, UnaryOp},
    },
    sources::{BytecodeMap, CodeArea, CodeSpan, SpwnSource},
    util::{Interner, RandomState},
};

use super::{
    bytecode::{Bytecode, BytecodeBuilder, FuncBuilder, Function},
    error::CompilerError,
};

pub type CompileResult<T> = Result<T, CompilerError>;

new_key_type! {
    pub struct ScopeKey;
}

#[derive(Clone, Copy, Debug)]
pub struct Variable {
    mutable: bool,
    def_span: CodeSpan,
    reg: usize,
}

#[derive(Clone)]
pub enum ScopeType {
    Global,
    Loop(Vec<usize>),
    MacroBody,
}

pub struct Scope {
    parent: Option<ScopeKey>,
    variables: AHashMap<Spur, Variable>,
    typ: Option<ScopeType>,
    // captures: Vec<Spur>,
}

pub struct Compiler<'a> {
    interner: Interner,
    scopes: SlotMap<ScopeKey, Scope>,
    pub src: SpwnSource,

    pub map: &'a mut BytecodeMap,

    global_return: Option<(Vec<Spanned<Spur>>, CodeSpan)>,
}

macro_rules! bop {
    ($left:ident $f:ident $right:ident ($scope:ident, $builder:ident, $out_reg:ident, $self:ident)) => {{
        let a = $self.compile_expr(&$left, $scope, $builder)?;
        let b = $self.compile_expr(&$right, $scope, $builder)?;
        $builder.$f(a, b, $out_reg)
    }};
}

impl<'a> Compiler<'a> {
    pub fn new(interner: Interner, src: SpwnSource, map: &'a mut BytecodeMap) -> Self {
        Self {
            interner,
            scopes: SlotMap::default(),
            src,
            map,
            global_return: None,
        }
    }
    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: self.src.clone(),
        }
    }

    pub fn get_var(&self, var: Spur, scope: ScopeKey) -> Option<Variable> {
        let scope = &self.scopes[scope];
        match scope.variables.get(&var) {
            Some(v) => Some(*v),
            None => match scope.parent {
                Some(k) => self.get_var(var, k),
                None => None,
            },
        }
    }
    pub fn redef_var(&mut self, var: Spur, span: CodeSpan, scope: ScopeKey) {
        let scope = &mut self.scopes[scope];
        match scope.variables.get_mut(&var) {
            Some(v) => v.def_span = span,
            None => {
                if let Some(k) = scope.parent {
                    self.redef_var(var, span, k)
                }
            }
        }
    }
    pub fn new_var(&mut self, var: Spur, data: Variable, scope: ScopeKey) {
        self.scopes[scope].variables.insert(var, data);
    }

    pub fn derive_scope(&mut self, scope: ScopeKey, typ: Option<ScopeType>) -> ScopeKey {
        self.scopes.insert(Scope {
            parent: Some(scope),
            variables: AHashMap::new(),
            typ,
        })
    }

    pub fn find_scope_type(&self, scope: ScopeKey) -> Option<&ScopeType> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => Some(t),
            None => match scope.parent {
                Some(k) => self.find_scope_type(k),
                None => None,
            },
        }
    }

    pub fn compile(&mut self, stmts: Vec<StmtNode>) -> CompileResult<()> {
        let mut builder = BytecodeBuilder::new();

        // func 0 ("global")
        builder.new_func(|f| {
            let base_scope = self.scopes.insert(Scope {
                parent: None,
                variables: AHashMap::new(),
                typ: Some(ScopeType::Global),
            });

            self.compile_stmts(&stmts, base_scope, f)
        })?;
        let (consts, unopt_funcs) = builder.build();

        let functions = unopt_funcs
            .into_iter()
            .map(|v| {
                let opcodes = v
                    .into_iter()
                    .map(|opcode| opcode.try_into().expect("404, sex not found , yo're missing the sex... Where is it? You find it? You do finding its now? Where art it hbe it has gone? Forgotten by societty , , sex walked across acres of empty land....   Almost as if ...,, he did... whereever he may be.... He will stay it will remain..... It told me this it would be like this it will. Be like this it will...."))
                    .collect();
                Function { opcodes }
            })
            .collect();

        // Ok(Bytecode { consts, functions })

        self.map
            .map
            .insert(self.src.clone(), Bytecode { consts, functions });

        Ok(())

        // Ok(builder.build())
    }

    pub fn compile_import(
        &mut self,
        typ: ImportType,
        scope: ScopeKey,
        span: CodeSpan,
    ) -> CompileResult<()> {
        let (src, name) = match typ {
            ImportType::Module(s) => (
                SpwnSource::File(PathBuf::from(self.interner.resolve(&s))),
                self.interner.resolve(&s),
            ),
            ImportType::Library(name) => (
                SpwnSource::File(PathBuf::from(format!(
                    "libraries/{}/lib.spwn",
                    self.interner.resolve(&name)
                ))),
                self.interner.resolve(&name),
            ),
        };
        let is_module = matches!(typ, ImportType::Module(_));

        // let bytecode

        let code = match src.read() {
            Some(s) => s,
            None => {
                return Err(CompilerError::NonexistentImport {
                    is_module,
                    name: name.into(),
                    area: self.make_area(span),
                })
            }
        };

        let interner: Interner = Rodeo::with_hasher(RandomState::new());
        let mut parser = Parser::new(code.trim_end(), src, interner);

        match parser.parse() {
            Ok(ast) => {
                let interner = Rc::try_unwrap(parser.interner)
                    .expect("multiple references still held (how??????????????????)")
                    .into_inner();
                let mut compiler = Compiler::new(interner, parser.src, self.map);

                match compiler.compile(ast.statements) {
                    Ok(bytecode) => {}
                    Err(err) => return Err(err),
                }
            }
            Err(err) => {
                return Err(CompilerError::ImportSyntaxError {
                    is_module,
                    err,
                    area: self.make_area(span),
                })
            }
        }

        Ok(())
    }

    pub fn compile_stmts(
        &mut self,
        stmts: &Vec<StmtNode>,
        scope: ScopeKey,
        builder: &mut FuncBuilder,
    ) -> CompileResult<()> {
        for stmt in stmts {
            self.compile_stmt(stmt, scope, builder)?;
        }
        Ok(())
    }

    pub fn compile_stmt(
        &mut self,
        stmt: &StmtNode,
        scope: ScopeKey,
        builder: &mut FuncBuilder,
    ) -> CompileResult<()> {
        match &*stmt.stmt {
            Statement::Expr(e) => {
                self.compile_expr(e, scope, builder)?;
            }
            Statement::Let(var, expr) => match *var.expr {
                Expression::Var(s) => {
                    let var_reg = builder.next_reg();
                    self.new_var(
                        s,
                        Variable {
                            mutable: true,
                            def_span: stmt.span,
                            reg: var_reg,
                        },
                        scope,
                    );
                    let expr_reg = self.compile_expr(expr, scope, builder)?;

                    builder.copy(expr_reg, var_reg);
                }
                _ => todo!("haha 😂😂😂😂"),
            },

            Statement::While { cond, code } => {
                builder.block(|b| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::Loop(b.block_path.clone())));

                    let cond_reg = self.compile_expr(cond, scope, b)?;
                    b.exit_if_false(cond_reg);

                    self.compile_stmts(code, inner_scope, b)?;

                    b.repeat_block();
                    Ok(())
                })?;
            }

            Statement::If {
                branches,
                else_branch,
            } => {
                builder.block(|outer_b| {
                    for (cond, code) in branches {
                        let outer_path = outer_b.block_path.clone();
                        let inner_scope = self.derive_scope(scope, None);
                        // let fuck = outer_b.test();
                        outer_b.block(|b| {
                            let cond_reg = self.compile_expr(cond, scope, b)?;
                            b.exit_if_false(cond_reg);
                            self.compile_stmts(code, inner_scope, b)?;

                            b.exit_other_block(outer_path);

                            Ok(())
                        })?;
                        // outer_b.exit_block();
                    }

                    if let Some(code) = else_branch {
                        let inner_scope = self.derive_scope(scope, None);
                        self.compile_stmts(code, inner_scope, outer_b)?;
                    }

                    Ok(())
                })?;
            }
            Statement::Break => match self.find_scope_type(scope) {
                Some(ScopeType::Loop(path)) => builder.exit_other_block(path.clone()),
                _ => {
                    return Err(CompilerError::BreakOutsideLoop {
                        area: self.make_area(stmt.span),
                    })
                }
            },
            Statement::Continue => match self.find_scope_type(scope) {
                Some(ScopeType::Loop(path)) => builder.exit_other_block(path.clone()),
                _ => {
                    return Err(CompilerError::ContinueOutsideLoop {
                        area: self.make_area(stmt.span),
                    })
                }
            },
            Statement::For {
                iter,
                iterator,
                code,
            } => todo!(),
            Statement::TryCatch {
                try_code,
                error_var,
                catch_code,
            } => todo!(),
            Statement::Arrow(statement) => {
                builder.block(|b| {
                    b.enter_arrow();
                    self.compile_stmt(statement, scope, b)?;
                    b.yeet_context();
                    Ok(())
                })?;
            }
            Statement::Return(value) => {
                if matches!(self.scopes[scope].typ, Some(ScopeType::Global)) {
                    match value {
                        Some(node) => match &*node.expr {
                            Expression::Dict(items) => {
                                if let Some((_, prev_span)) = self.global_return {
                                    return Err(CompilerError::DuplicateModuleReturn {
                                        area: self.make_area(stmt.span),
                                        prev_area: self.make_area(prev_span),
                                    });
                                }

                                let ret_reg = self.compile_expr(node, scope, builder)?;
                                self.global_return =
                                    Some((items.iter().map(|i| i.0).collect(), stmt.span));
                                builder.export(ret_reg);
                            }
                            _ => {
                                return Err(CompilerError::InvalidModuleReturn {
                                    area: self.make_area(stmt.span),
                                })
                            }
                        },
                        None => {
                            return Err(CompilerError::InvalidModuleReturn {
                                area: self.make_area(stmt.span),
                            })
                        }
                    }
                } else {
                    match self.find_scope_type(scope) {
                        Some(ScopeType::MacroBody) => {
                            let out_reg = builder.next_reg();
                            match value {
                                None => {
                                    builder.load_empty(out_reg);
                                    builder.ret(out_reg)
                                }
                                Some(expr) => {
                                    self.compile_expr(expr, scope, builder)?;
                                    builder.ret(out_reg)
                                }
                            }
                        }
                        _ => {
                            return Err(CompilerError::ReturnOutsideMacro {
                                area: self.make_area(stmt.span),
                            })
                        }
                    }
                }
            }
            Statement::TypeDef(_) => todo!(),
            Statement::Impl { typ, items } => todo!(),
            Statement::ExtractImport(_) => todo!(),
        }
        Ok(())
    }

    pub fn compile_expr(
        &mut self,
        expr: &ExprNode,
        scope: ScopeKey,
        builder: &mut FuncBuilder,
    ) -> CompileResult<usize> {
        let out_reg = builder.next_reg();

        match &*expr.expr {
            Expression::Int(v) => builder.load_int(*v, out_reg),
            Expression::Float(v) => builder.load_float(*v, out_reg),
            Expression::Bool(v) => builder.load_bool(*v, out_reg),
            Expression::String(v) => {
                builder.load_string(self.interner.resolve(v).to_string(), out_reg)
            }
            Expression::Id(class, value) => builder.load_id(*value, *class, out_reg),
            Expression::Op(left, op, right) => match op {
                BinOp::Plus => bop!(left add right (scope, builder, out_reg, self)),
                BinOp::Minus => bop!(left sub right (scope, builder, out_reg, self)),
                BinOp::Mult => bop!(left mult right (scope, builder, out_reg, self)),
                BinOp::Div => bop!(left div right (scope, builder, out_reg, self)),
                BinOp::Mod => bop!(left modulo right (scope, builder, out_reg, self)),
                BinOp::Pow => bop!(left pow right (scope, builder, out_reg, self)),
                BinOp::ShiftLeft => bop!(left shl right (scope, builder, out_reg, self)),
                BinOp::ShiftRight => bop!(left shr right (scope, builder, out_reg, self)),
                BinOp::BinAnd => bop!(left bin_and right (scope, builder, out_reg, self)),
                BinOp::BinOr => bop!(left bin_or right (scope, builder, out_reg, self)),
                // specky can do the rest :)))
                BinOp::PlusEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.add_eq(a, b)
                }
                BinOp::MinusEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.sub_eq(a, b)
                }
                BinOp::MultEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.mult_eq(a, b)
                }
                BinOp::DivEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.div_eq(a, b)
                }
                BinOp::ModEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.modulo_eq(a, b)
                }
                BinOp::PowEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.pow_eq(a, b)
                }
                BinOp::ShiftLeftEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.shl_eq(a, b)
                }
                BinOp::ShiftRightEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.shr_eq(a, b)
                }
                BinOp::BinAndEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.bin_and_eq(a, b)
                }
                BinOp::BinOrEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.bin_or_eq(a, b)
                }
                BinOp::BinNotEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.bin_not_eq(a, b)
                }
                BinOp::Eq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.eq(a, b, out_reg)
                }
                BinOp::Neq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.neq(a, b, out_reg)
                }
                BinOp::Gt => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.gt(a, b, out_reg)
                }
                BinOp::Gte => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.gte(a, b, out_reg)
                }
                BinOp::Lt => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.lt(a, b, out_reg)
                }
                BinOp::Lte => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.lte(a, b, out_reg)
                }
                BinOp::DotDot => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.range(a, b, out_reg)
                }
                BinOp::In => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.in_op(a, b, out_reg)
                }
                BinOp::As => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.as_op(a, b, out_reg)
                }
                BinOp::Is => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.is_op(a, b, out_reg)
                }

                BinOp::Assign => match *left.expr {
                    Expression::Var(s) => match self.get_var(s, scope) {
                        Some(var) => {
                            if var.mutable {
                                self.redef_var(s, expr.span, scope);
                                let expr_reg = self.compile_expr(right, scope, builder)?;
                                builder.copy(expr_reg, var.reg);
                            } else {
                                return Err(CompilerError::ImmutableAssign {
                                    area: self.make_area(expr.span),
                                    def_area: self.make_area(var.def_span),
                                    var: self.interner.resolve(&s).into(),
                                });
                            }
                        }
                        None => {
                            let var_reg = builder.next_reg();
                            self.new_var(
                                s,
                                Variable {
                                    mutable: false,
                                    def_span: expr.span,
                                    reg: var_reg,
                                },
                                scope,
                            );
                            let expr_reg = self.compile_expr(right, scope, builder)?;

                            builder.copy(expr_reg, var_reg);
                        }
                    },
                    _ => todo!("haha 😂😂😂😂"),
                },
            },
            Expression::Unary(op, value) => {
                let v = self.compile_expr(value, scope, builder)?;
                match op {
                    UnaryOp::BinNot => builder.unary_bin_not(v, out_reg),
                    UnaryOp::ExclMark => builder.unary_not(v, out_reg),
                    UnaryOp::Minus => builder.unary_negate(v, out_reg),
                }
            }
            Expression::Var(name) => match self.get_var(*name, scope) {
                Some(data) => return Ok(data.reg),
                None => {
                    return Err(CompilerError::NonexistentVariable {
                        area: self.make_area(expr.span),
                        var: self.interner.resolve(name).into(),
                    })
                }
            },
            Expression::Type(_) => todo!(),
            Expression::Array(items) => {
                builder.new_array(items.len() as u16, out_reg, |builder, elems| {
                    for item in items {
                        elems.push(self.compile_expr(item, scope, builder)?);
                    }
                    Ok(())
                })?;
            }
            Expression::Dict(items) => {
                builder.new_dict(items.len() as u16, out_reg, |builder, elems| {
                    for (key, item) in items {
                        let value_reg = match item {
                            Some(e) => self.compile_expr(e, scope, builder)?,
                            None => match self.get_var(key.value, scope) {
                                Some(data) => data.reg,
                                None => {
                                    return Err(CompilerError::NonexistentVariable {
                                        area: self.make_area(expr.span),
                                        var: self.interner.resolve(&key.value).into(),
                                    })
                                }
                            },
                        };

                        elems.push((self.interner.resolve(&key.value).into(), value_reg));
                    }
                    Ok(())
                })?;
            }
            Expression::Maybe(e) => match e {
                Some(e) => {
                    let value = self.compile_expr(e, scope, builder)?;
                    builder.wrap_maybe(value, out_reg)
                }
                None => builder.load_none(out_reg),
            },
            Expression::Index { base, index } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                let index_reg = self.compile_expr(index, scope, builder)?;
                builder.index(base_reg, out_reg, index_reg)
            }
            Expression::Member { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                builder.member(base_reg, out_reg, self.interner.resolve(&name).into())
            }
            Expression::Associated { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                builder.associated(base_reg, out_reg, self.interner.resolve(&name).into())
            }
            Expression::Call {
                base,
                params,
                named_params,
            } => todo!(),
            Expression::Macro {
                args,
                ret_type,
                code,
            } => todo!(),
            Expression::MacroPattern { args, ret_type } => todo!(),
            Expression::TriggerFunc { attributes, code } => todo!(),
            Expression::TriggerFuncCall(_) => todo!(),
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => {
                let cond_reg = self.compile_expr(cond, scope, builder)?;
                builder.block(|outer_b| {
                    let outer_path = outer_b.block_path.clone();

                    outer_b.block(|b| {
                        b.exit_if_false(cond_reg);
                        let reg = self.compile_expr(if_true, scope, b)?;
                        b.copy(reg, out_reg);

                        b.exit_other_block(outer_path);
                        Ok(())
                    })?;

                    let reg = self.compile_expr(if_false, scope, outer_b)?;
                    outer_b.copy(reg, out_reg);

                    Ok(())
                })?;
            }
            Expression::Typeof(_) => todo!(),
            Expression::Builtins => {
                builder.load_builtins(out_reg);
            }
            Expression::Empty => {
                builder.load_empty(out_reg);
            }
            Expression::Import(t) => {
                self.compile_import(*t, scope, expr.span)?;
            }
            Expression::Instance { base, items } => todo!(),
        }

        Ok(out_reg)
    }
}

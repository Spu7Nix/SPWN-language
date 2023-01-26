use std::cell::RefCell;
use std::rc::Rc;

use ahash::AHashMap;
use lasso::Spur;
use slotmap::{new_key_type, SlotMap};

use super::bytecode::{Bytecode, BytecodeBuilder, FuncBuilder, Function};
use super::error::CompilerError;
use crate::cli::FileSettings;
use crate::parsing::ast::{
    ExprNode, Expression, ImportType, MacroCode, Spannable, Spanned, Statement, StmtNode,
};
use crate::parsing::parser::Parser;
use crate::parsing::utils::operators::{AssignOp, BinOp, UnaryOp};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, SpwnSource};
use crate::util::Interner;
use crate::vm::opcodes::Register;

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
}

pub struct Compiler<'a> {
    interner: Rc<RefCell<Interner>>,
    scopes: SlotMap<ScopeKey, Scope>,
    pub src: SpwnSource,

    global_return: Option<(Vec<Spanned<Spur>>, CodeSpan)>,

    file_attrs: &'a FileSettings,

    pub map: &'a mut BytecodeMap,
}

impl<'a> Compiler<'a> {
    pub fn new(
        interner: Rc<RefCell<Interner>>,
        src: SpwnSource,
        file_attrs: &'a FileSettings,
        map: &'a mut BytecodeMap,
    ) -> Self {
        Self {
            interner,
            scopes: SlotMap::default(),
            src,
            global_return: None,
            file_attrs,
            map,
        }
    }

    fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: self.src.clone(),
        }
    }

    fn resolve(&self, spur: &Spur) -> String {
        self.interner.borrow().resolve(spur).to_string()
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

    pub fn get_accessible_vars(&self, scope: ScopeKey) -> Vec<(Spur, Variable)> {
        let mut vars = vec![];
        fn inner(compiler: &Compiler, scope: ScopeKey, vars: &mut Vec<(Spur, Variable)>) {
            let scope = &compiler.scopes[scope];
            for (name, data) in &scope.variables {
                vars.push((*name, *data))
            }
            if let Some(p) = scope.parent {
                inner(compiler, p, vars)
            }
        }
        inner(self, scope, &mut vars);
        vars
    }

    pub fn compile(&mut self, stmts: Vec<StmtNode>) -> CompileResult<&Bytecode<Register>> {
        let mut builder = BytecodeBuilder::new();

        // func 0 ("global")
        builder.new_func(
            |f| {
                let base_scope = self.scopes.insert(Scope {
                    parent: None,
                    variables: AHashMap::new(),
                    typ: Some(ScopeType::Global),
                });

                self.compile_stmts(&stmts, base_scope, f)?;

                Ok(vec![])
            },
            0,
        )?;

        let unopt_code = builder.build(
            &self.src,
            match &self.global_return {
                Some(v) => {
                    v.0.iter()
                        .map(|s| self.interner.borrow().resolve(&s.value).into())
                        .collect()
                }
                None => vec![],
            },
        );

        let functions = unopt_code
            .functions
            .into_iter()
            .map(|f| {
                // let v = if true { // TODO: change this to a debug flag or #[no_bytecode_optimization] attribute
                //     optimize_function(&v)
                // } else {
                //     v
                // };

                let opcodes = f
                    .opcodes
                    .into_iter()
                    .map(|opcode| opcode.try_into().expect("usize too big for u8"))
                    .collect();
                Function {
                    opcodes,
                    regs_used: f.regs_used,
                    arg_amount: f.arg_amount,
                    capture_regs: f
                        .capture_regs
                        .iter()
                        .map(|(from, to)| (*from as Register, *to as Register))
                        .collect(),
                }
            })
            .collect();

        self.map.map.insert(
            self.src.clone(),
            Bytecode {
                import_paths: unopt_code.import_paths,
                source_hash: unopt_code.source_hash,
                consts: unopt_code.consts,
                functions,
                opcode_span_map: unopt_code.opcode_span_map,
                export_names: unopt_code.export_names,
            },
        );

        Ok(&self.map.map[&self.src])
    }

    pub fn compile_import(
        &mut self,
        typ: &ImportType,
        span: CodeSpan,
        importer_src: SpwnSource,
    ) -> CompileResult<()> {
        let base_dir = match &importer_src {
            SpwnSource::File(path) => path.parent().unwrap(),
        };

        let (name, rel_path) = typ.to_path_name();
        let import_path = base_dir.join(rel_path);

        let import_src = SpwnSource::File(import_path.clone());
        let is_module = matches!(typ, ImportType::Module(_));

        let code = match import_src.read() {
            Some(s) => s,
            None => {
                return Err(CompilerError::NonexistentImport {
                    is_module,
                    name,
                    area: self.make_area(span),
                })
            }
        };
        let import_base = import_path.parent().unwrap();

        let hash = md5::compute(&code);

        let spwnc_path = import_base.join(format!(".spwnc/{name}.spwnc"));

        'from_cache: {
            if spwnc_path.is_file() {
                let source = std::fs::read(&spwnc_path).unwrap();

                let bytecode: Bytecode<Register> = match bincode::deserialize(&source) {
                    Ok(b) => b,
                    Err(_) => {
                        break 'from_cache;
                    }
                };

                if bytecode.source_hash == hash.into() {
                    for import in &bytecode.import_paths {
                        self.compile_import(&import.value, import.span, import_src.clone())?;
                    }

                    self.map.map.insert(import_src, bytecode);
                    return Ok(());
                    // break 'bytecode &self.map.map[&import_src];
                }
            }
        }

        let mut parser = Parser::new(code.trim_end(), import_src, Rc::clone(&self.interner));

        match parser.parse() {
            Ok(ast) => {
                let mut file_settings = FileSettings::default();
                file_settings.apply_attributes(&ast.file_attributes);

                let mut compiler = Compiler::new(
                    Rc::clone(&self.interner),
                    parser.src,
                    &file_settings,
                    self.map,
                );

                match compiler.compile(ast.statements) {
                    Ok(bytecode) => {
                        let bytes = bincode::serialize(&bytecode).unwrap();

                        // dont write bytecode if caching is disabled
                        if !self.file_attrs.no_bytecode_cache {
                            let _ = std::fs::create_dir(import_base.join(".spwnc"));
                            std::fs::write(&spwnc_path, bytes).unwrap();
                        }
                    }
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
        };

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

                    builder.copy(expr_reg, var_reg, stmt.span);
                }
                _ => todo!("haha 😂😂😂😂"),
            },
            Statement::AssignOp(left, op, right) => match op {
                AssignOp::Assign => match *left.expr {
                    Expression::Var(s) => match self.get_var(s, scope) {
                        Some(var) => {
                            if var.mutable {
                                self.redef_var(s, stmt.span, scope);
                                let expr_reg = self.compile_expr(right, scope, builder)?;

                                builder.copy(expr_reg, var.reg, stmt.span);
                            } else {
                                return Err(CompilerError::ImmutableAssign {
                                    area: self.make_area(stmt.span),
                                    def_area: self.make_area(var.def_span),
                                    var: self.resolve(&s),
                                });
                            }
                        }
                        None => {
                            let var_reg = builder.next_reg();

                            self.new_var(
                                s,
                                Variable {
                                    mutable: false,
                                    def_span: stmt.span,
                                    reg: var_reg,
                                },
                                scope,
                            );
                            let expr_reg = self.compile_expr(right, scope, builder)?;

                            builder.copy(expr_reg, var_reg, stmt.span);
                        }
                    },
                    _ => todo!("haha 😂😂😂😂"),
                },
                AssignOp::PlusEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.add_eq(a, b, stmt.span)
                }
                AssignOp::MinusEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.sub_eq(a, b, stmt.span)
                }
                AssignOp::MultEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.mult_eq(a, b, stmt.span)
                }
                AssignOp::DivEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.div_eq(a, b, stmt.span)
                }
                AssignOp::ModEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.modulo_eq(a, b, stmt.span)
                }
                AssignOp::PowEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.pow_eq(a, b, stmt.span)
                }
                AssignOp::ShiftLeftEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.shl_eq(a, b, stmt.span)
                }
                AssignOp::ShiftRightEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.shr_eq(a, b, stmt.span)
                }
                AssignOp::BinAndEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.bin_and_eq(a, b, stmt.span)
                }
                AssignOp::BinOrEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.bin_or_eq(a, b, stmt.span)
                }
                AssignOp::BinNotEq => {
                    let a = self.compile_expr(left, scope, builder)?;
                    let b = self.compile_expr(right, scope, builder)?;
                    builder.bin_not_eq(a, b, stmt.span)
                }
            },

            Statement::While { cond, code } => {
                builder.block(|b| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::Loop(b.block_path.clone())));

                    let cond_reg = self.compile_expr(cond, scope, b)?;
                    b.exit_if_false(cond_reg, cond.span);

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
                            b.exit_if_false(cond_reg, cond.span);
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
                Some(ScopeType::Loop(path)) => builder.repeat_other_block(path.clone()),
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
                                builder.ret(ret_reg);
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
                        Some(ScopeType::MacroBody) => match value {
                            None => {
                                let out_reg = builder.next_reg();
                                builder.load_empty(out_reg, stmt.span);
                                builder.ret(out_reg)
                            }
                            Some(expr) => {
                                let ret_reg = self.compile_expr(expr, scope, builder)?;
                                builder.ret(ret_reg)
                            }
                        },
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
            Statement::Print(v) => {
                let v_reg = self.compile_expr(v, scope, builder)?;
                builder.print(v_reg);
            }
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

        macro_rules! bin_op {
            ($left:ident $fn:ident $right:ident) => {{
                let a = self.compile_expr(&$left, scope, builder)?;
                let b = self.compile_expr(&$right, scope, builder)?;
                builder.$fn(a, b, out_reg, expr.span)
            }};
        }

        match &*expr.expr {
            Expression::Int(v) => builder.load_int(*v, out_reg, expr.span),
            Expression::Float(v) => builder.load_float(*v, out_reg, expr.span),
            Expression::Bool(v) => builder.load_bool(*v, out_reg, expr.span),
            Expression::String(v) => builder.load_string(self.resolve(v), out_reg, expr.span),
            Expression::Id(class, value) => builder.load_id(*value, *class, out_reg, expr.span),
            Expression::Op(left, op, right) => match op {
                BinOp::Plus => bin_op!(left add right),
                BinOp::Minus => bin_op!(left sub right),
                BinOp::Mult => bin_op!(left mult right),
                BinOp::Div => bin_op!(left div right),
                BinOp::Mod => bin_op!(left modulo right),
                BinOp::Pow => bin_op!(left pow right),
                BinOp::ShiftLeft => {
                    bin_op!(left shl right)
                }
                BinOp::ShiftRight => {
                    bin_op!(left shr right)
                }
                BinOp::BinAnd => {
                    bin_op!(left bin_and right)
                }
                BinOp::BinOr => {
                    bin_op!(left bin_or right)
                }
                BinOp::Eq => {
                    bin_op!(left eq right)
                }
                BinOp::Neq => {
                    bin_op!(left neq right)
                }
                BinOp::Gt => {
                    bin_op!(left gt right)
                }
                BinOp::Gte => {
                    bin_op!(left gte right)
                }
                BinOp::Lt => {
                    bin_op!(left lt right)
                }
                BinOp::Lte => {
                    bin_op!(left lte right)
                }
                BinOp::Range => {
                    bin_op!(left range right)
                }
                BinOp::In => {
                    bin_op!(left in_op right)
                }
                BinOp::As => {
                    bin_op!(left as_op right)
                }
                BinOp::Is => {
                    bin_op!(left is_op right)
                }
                BinOp::Or => todo!(),
                BinOp::And => todo!(),
            },
            Expression::Unary(op, value) => {
                let v = self.compile_expr(value, scope, builder)?;
                match op {
                    UnaryOp::BinNot => builder.unary_bin_not(v, out_reg, expr.span),
                    UnaryOp::ExclMark => builder.unary_not(v, out_reg, expr.span),
                    UnaryOp::Minus => builder.unary_negate(v, out_reg, expr.span),
                }
            }
            Expression::Var(name) => match self.get_var(*name, scope) {
                Some(data) => return Ok(data.reg),
                None => {
                    return Err(CompilerError::NonexistentVariable {
                        area: self.make_area(expr.span),
                        var: self.resolve(name),
                    })
                }
            },
            Expression::Type(_) => todo!(),
            Expression::Array(items) => {
                builder.new_array(
                    items.len() as u16,
                    out_reg,
                    |builder, elems| {
                        for item in items {
                            elems.push(self.compile_expr(item, scope, builder)?);
                        }
                        Ok(())
                    },
                    expr.span,
                )?;
            }
            Expression::Dict(items) => {
                builder.new_dict(
                    items.len() as u16,
                    out_reg,
                    |builder, elems| {
                        for (key, item) in items {
                            let value_reg = match item {
                                Some(e) => self.compile_expr(e, scope, builder)?,
                                None => match self.get_var(key.value, scope) {
                                    Some(data) => data.reg,
                                    None => {
                                        return Err(CompilerError::NonexistentVariable {
                                            area: self.make_area(key.span),
                                            var: self.resolve(&key.value),
                                        })
                                    }
                                },
                            };

                            elems.push((self.resolve(&key.value).spanned(key.span), value_reg));
                        }
                        Ok(())
                    },
                    expr.span,
                )?;
            }
            Expression::Maybe(e) => match e {
                Some(e) => {
                    let value = self.compile_expr(e, scope, builder)?;
                    builder.wrap_maybe(value, out_reg, expr.span)
                }
                None => builder.load_none(out_reg, expr.span),
            },
            Expression::Index { base, index } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                let index_reg = self.compile_expr(index, scope, builder)?;
                builder.index(base_reg, out_reg, index_reg, expr.span)
            }
            Expression::Member { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                builder.member(
                    base_reg,
                    out_reg,
                    self.resolve(&name.value).spanned(name.span),
                    expr.span,
                )
            }
            Expression::Associated { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                builder.associated(
                    base_reg,
                    out_reg,
                    self.resolve(&name.value).spanned(name.span),
                    expr.span,
                )
            }
            Expression::Macro {
                args,
                ret_type,
                code,
            } => {
                let func_id = builder.new_func(
                    |f| {
                        let mut variables = AHashMap::new();

                        for (name, ..) in args {
                            variables.insert(
                                name.value,
                                Variable {
                                    mutable: false,
                                    def_span: name.span,
                                    reg: f.next_reg(),
                                },
                            );
                        }
                        let to_capture = self.get_accessible_vars(scope);
                        let mut capture_regs = vec![];

                        for (name, data) in to_capture {
                            let reg = f.next_reg();
                            capture_regs.push((data.reg, reg));
                            variables.insert(name, Variable { reg, ..data });
                        }

                        let base_scope = self.scopes.insert(Scope {
                            parent: None,
                            variables,
                            typ: Some(ScopeType::MacroBody),
                        });

                        match code {
                            MacroCode::Normal(stmts) => self.compile_stmts(stmts, base_scope, f)?,
                            MacroCode::Lambda(expr) => {
                                let ret_reg = self.compile_expr(expr, base_scope, f)?;
                                f.ret(ret_reg);
                            }
                        }

                        Ok(capture_regs)
                    },
                    args.len(),
                )?;

                builder.new_macro(
                    func_id,
                    out_reg,
                    |builder, elems| {
                        for (name, pat, def) in args {
                            let n = self.resolve(&name.value).spanned(name.span);

                            let p = if let Some(p) = pat {
                                Some(self.compile_expr(p, scope, builder)?)
                            } else {
                                None
                            };
                            let d = if let Some(d) = def {
                                Some(self.compile_expr(d, scope, builder)?)
                            } else {
                                None
                            };

                            elems.push((n, p, d))
                        }

                        Ok(())
                    },
                    expr.span,
                )?;
            }
            Expression::Call {
                base,
                params,
                named_params,
            } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                let args_reg = builder.next_reg();
                builder.new_array(
                    2,
                    args_reg,
                    |builder, elems| {
                        let params_reg = builder.next_reg();
                        builder.new_array(
                            params.len() as u16,
                            params_reg,
                            |builder, elems| {
                                for i in params {
                                    elems.push(self.compile_expr(i, scope, builder)?);
                                }
                                Ok(())
                            },
                            expr.span,
                        )?;

                        let named_params_reg = builder.next_reg();
                        builder.new_dict(
                            named_params.len() as u16,
                            named_params_reg,
                            |builder, elems| {
                                for (name, param) in named_params {
                                    let value_reg = self.compile_expr(param, scope, builder)?;

                                    elems.push((
                                        self.resolve(&name.value).spanned(name.span),
                                        value_reg,
                                    ));
                                }
                                Ok(())
                            },
                            expr.span,
                        )?;

                        elems.push(params_reg);
                        elems.push(named_params_reg);

                        Ok(())
                    },
                    expr.span,
                )?;
                builder.call(base_reg, out_reg, args_reg, expr.span);
            }
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
                        b.exit_if_false(cond_reg, cond.span);
                        let reg = self.compile_expr(if_true, scope, b)?;
                        b.copy(reg, out_reg, expr.span);

                        b.exit_other_block(outer_path);
                        Ok(())
                    })?;

                    let reg = self.compile_expr(if_false, scope, outer_b)?;
                    outer_b.copy(reg, out_reg, expr.span);

                    Ok(())
                })?;
            }
            Expression::Typeof(_) => todo!(),
            Expression::Builtins => {
                builder.load_builtins(out_reg, expr.span);
            }
            Expression::Empty => {
                builder.load_empty(out_reg, expr.span);
            }
            Expression::Import(t) => {
                self.compile_import(t, expr.span, self.src.clone())?;
                builder.import(out_reg, t.clone().spanned(expr.span), expr.span)
            }
            Expression::Instance { base, items } => todo!(),
        }

        Ok(out_reg)
    }
}

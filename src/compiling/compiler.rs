use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use ahash::AHashMap;
use delve::VariantNames;
use lasso::Spur;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

use super::bytecode::{Bytecode, BytecodeBuilder, Constant, FuncBuilder, Function};
use super::error::CompilerError;
use crate::cli::Settings;
use crate::gd::object_keys::{ObjectKeyValueType, OBJECT_KEYS};
use crate::parsing::ast::{
    DictItems, ExprNode, Expression, ImportType, MacroArg, MacroCode, ModuleImport, ObjKeyType,
    Pattern, PatternNode, Spannable, Spanned, Statement, StmtNode, StringContent,
};
use crate::parsing::attributes::ScriptAttribute;
use crate::parsing::parser::Parser;
use crate::parsing::utils::operators::{AssignOp, BinOp, Operator, UnaryOp};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, SpwnSource};
use crate::util::{Interner, BUILTIN_DIR};
use crate::vm::opcodes::{Register, UnoptRegister};
use crate::vm::pattern::ConstPattern;
use crate::vm::value::ValueType;

pub type CompileResult<T> = Result<T, CompilerError>;

new_key_type! {
    pub struct ScopeKey;
    pub struct CustomTypeKey;
}

#[derive(Clone, Copy, Debug)]
pub struct Variable {
    mutable: bool,
    def_span: CodeSpan,
    reg: usize,
}

#[derive(Clone, Debug)]
pub enum ScopeType {
    Global,
    Loop(Vec<usize>),
    MacroBody,
    TriggerFunc(CodeArea),
    ArrowStmt(CodeArea),
}

pub struct Scope {
    parent: Option<ScopeKey>,
    variables: AHashMap<Spur, Variable>,
    typ: Option<ScopeType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDef {
    pub def_src: SpwnSource,
    pub name: Spur,
    pub private: bool,
}

pub type TypeDefMap = AHashMap<TypeDef, Spanned<CustomTypeKey>>;

pub struct Compiler<'a> {
    interner: Rc<RefCell<Interner>>,
    scopes: SlotMap<ScopeKey, Scope>,
    pub src: SpwnSource,

    global_return: Option<(Vec<Spanned<Spur>>, CodeSpan)>,

    settings: &'a Settings,

    pub map: &'a mut BytecodeMap,

    pub custom_type_defs: &'a mut TypeDefMap,
    available_custom_types: AHashMap<Spur, CustomTypeKey>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        interner: Rc<RefCell<Interner>>,
        src: SpwnSource,
        settings: &'a Settings,
        map: &'a mut BytecodeMap,
        type_defs: &'a mut TypeDefMap,
    ) -> Self {
        Self {
            interner,
            scopes: SlotMap::default(),
            src,
            global_return: None,
            settings,
            map,
            custom_type_defs: type_defs,
            available_custom_types: AHashMap::new(),
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

    fn intern(&self, s: &str) -> Spur {
        self.interner.borrow_mut().get_or_intern(s)
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

    // pub fn redef_var(&mut self, var: Spur, span: CodeSpan, scope: ScopeKey) {
    //     let scope = &mut self.scopes[scope];
    //     match scope.variables.get_mut(&var) {
    //         Some(v) => v.def_span = span,
    //         None => {
    //             if let Some(k) = scope.parent {
    //                 self.redef_var(var, span, k)
    //             }
    //         }
    //     }
    // }

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

    // pub fn find_scope_type(&self, scope: ScopeKey) -> Option<&ScopeType> {
    //     let scope = &self.scopes[scope];
    //     match &scope.typ {
    //         Some(t) => Some(t),
    //         None => match scope.parent {
    //             Some(k) => self.find_scope_type(k),
    //             None => None,
    //         },
    //     }
    // }

    fn is_inside_loop(&self, scope: ScopeKey) -> Option<&Vec<usize>> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(ScopeType::ArrowStmt(_) | ScopeType::TriggerFunc(_)) | None => {
                match scope.parent {
                    Some(k) => self.is_inside_loop(k),
                    None => None,
                }
            },
            Some(t) => match t {
                ScopeType::Loop(v) => Some(v),
                _ => None,
            },
        }
    }

    fn is_inside_macro(&self, scope: ScopeKey) -> bool {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => match t {
                ScopeType::MacroBody => return true,
                ScopeType::Global => return false,
                _ => (),
            },
            None => (),
        }
        match scope.parent {
            Some(k) => self.is_inside_macro(k),
            None => false,
        }
    }

    pub fn assert_can_return(&self, scope: ScopeKey, area: CodeArea) -> CompileResult<()> {
        fn can_return_d(slf: &Compiler, scope: ScopeKey, area: CodeArea) -> CompileResult<()> {
            let scope = &slf.scopes[scope];
            match &scope.typ {
                Some(t) => match t {
                    ScopeType::MacroBody => return Ok(()),
                    ScopeType::TriggerFunc(def) => {
                        return Err(CompilerError::BreakInTriggerFuncScope {
                            area,
                            def: def.clone(),
                        })
                    },
                    ScopeType::ArrowStmt(def) => {
                        return Err(CompilerError::BreakInArrowStmtScope {
                            area,
                            def: def.clone(),
                        })
                    },
                    _ => (),
                },
                None => (),
            }
            match scope.parent {
                Some(k) => can_return_d(slf, k, area),
                None => unreachable!(),
            }
        }

        if let Some(ScopeType::ArrowStmt(_)) = self.scopes[scope].typ {
            return Ok(()); // -> return
        }

        can_return_d(self, scope, area)
    }

    pub fn assert_can_break_loop(&self, scope: ScopeKey, area: CodeArea) -> CompileResult<()> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => match t {
                ScopeType::Loop(_) => return Ok(()),
                ScopeType::TriggerFunc(def) => {
                    return Err(CompilerError::BreakInTriggerFuncScope {
                        area,
                        def: def.clone(),
                    })
                },
                ScopeType::ArrowStmt(def) => {
                    return Err(CompilerError::BreakInArrowStmtScope {
                        area,
                        def: def.clone(),
                    })
                },

                _ => (),
            },
            None => (),
        }
        match scope.parent {
            Some(k) => self.assert_can_break_loop(k, area),
            None => unreachable!(), // should only be called after is_inside_loop
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

    pub fn compile(
        &mut self,
        stmts: Vec<StmtNode>,
        entry: Option<Vec<ScriptAttribute>>,
        span: CodeSpan,
    ) -> CompileResult<&Bytecode<Register>> {
        let mut builder = BytecodeBuilder::new();

        // func 0 ("global")
        builder.new_func(
            |f| {
                let base_scope = self.scopes.insert(Scope {
                    parent: None,
                    variables: AHashMap::new(),
                    typ: Some(ScopeType::Global),
                });

                let builtin_import_reg = f.next_reg();

                if entry.is_some() {
                    let import_type =
                        ImportType::Module(BUILTIN_DIR.join("core/lib.spwn"), ModuleImport::Core);
                    self.compile_import(&import_type, CodeSpan::internal(), self.src.clone())?;
                    f.import(
                        builtin_import_reg,
                        import_type.spanned(CodeSpan::internal()),
                        CodeSpan::internal(),
                    )
                }
                if let Some(attrs) = entry {
                    if !attrs.iter().any(|a| *a == ScriptAttribute::NoStd) {
                        let import_type =
                            ImportType::Module(BUILTIN_DIR.join("std/lib.spwn"), ModuleImport::Std);
                        self.compile_import(&import_type, CodeSpan::internal(), self.src.clone())?;
                        f.import(
                            builtin_import_reg,
                            import_type.spanned(CodeSpan::internal()),
                            CodeSpan::internal(),
                        )
                    }
                }

                self.compile_stmts(&stmts, base_scope, f)?;

                let final_ret = f.next_reg();

                f.load_empty_dict(
                    final_ret,
                    CodeSpan {
                        start: usize::MAX,
                        end: usize::MAX,
                    },
                );
                f.ret(final_ret, true, span);

                // f.load_empty(reg, span)

                Ok((vec![], vec![]))
            },
            0,
            span,
        )?;

        let unopt_code = builder.build(
            &self.src,
            match &self.global_return {
                Some(v) => {
                    v.0.iter()
                        .map(|s| self.interner.borrow().resolve(&s.value).into())
                        .collect()
                },
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
                    .map(|opcode| {
                        opcode
                            .try_into()
                            .expect("usize too big for u8 (too many registers used)")
                    })
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
                    ref_arg_regs: f.ref_arg_regs.iter().map(|r| *r as Register).collect(),
                    span: f.span,
                }
            })
            .collect();

        self.map.map.insert(
            self.src.clone(),
            Bytecode {
                import_paths: unopt_code.import_paths,
                spwn_ver: unopt_code.spwn_ver,
                custom_types: unopt_code.custom_types,
                source_hash: unopt_code.source_hash,
                consts: unopt_code.consts,
                const_patterns: unopt_code.const_patterns,
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
        let base_dir = importer_src.path().parent().unwrap();

        let (name, import_path) = (typ.name(), typ.full_path(base_dir));

        let import_src =
            importer_src.change_path_conditional(import_path.clone(), typ.module_import_type());

        let is_module = matches!(typ, ImportType::Module { .. });

        let code = match import_src.read() {
            Some(s) => s,
            None => {
                return Err(CompilerError::NonexistentImport {
                    is_module,
                    name,
                    area: self.make_area(span),
                });
            },
        };
        let import_base = import_path.parent().unwrap();

        let hash = md5::compute(&code);

        let spwnc_path = import_base.join(format!(".spwnc/{name}.spwnc"));

        'from_cache: {
            // if spwnc_path.is_file() {
            //     let source_bytes = std::fs::read(&spwnc_path).unwrap();

            //     let bytecode: Bytecode<Register> = match bincode::deserialize(&source_bytes) {
            //         Ok(b) => b,
            //         Err(_) => {
            //             break 'from_cache;
            //         },
            //     };

            //     if bytecode.source_hash == hash.into()
            //         && bytecode.spwn_ver == env!("CARGO_PKG_VERSION")
            //     {
            //         for import in &bytecode.import_paths {
            //             self.compile_import(&import.value, import.span, import_src.clone())?;
            //         }
            //         for (k, (name, private)) in &bytecode.custom_types {
            //             self.custom_type_defs.insert(
            //                 TypeDef {
            //                     def_src: import_src.clone(),
            //                     name: self.intern(&name.value),
            //                     private: *private,
            //                 },
            //                 k.spanned(name.span),
            //             );
            //         }

            //         self.map.map.insert(import_src, bytecode);
            //         return Ok(());
            //     }
            // }
        }

        let mut parser = Parser::new(&code, import_src, Rc::clone(&self.interner));

        match parser.parse() {
            Ok(ast) => {
                let mut compiler = Compiler::new(
                    Rc::clone(&self.interner),
                    parser.src,
                    self.settings,
                    self.map,
                    self.custom_type_defs,
                );

                match compiler.compile(
                    ast.statements,
                    None,
                    CodeSpan {
                        start: 0,
                        end: code.len(),
                    },
                ) {
                    Ok(bytecode) => {
                        let bytes = bincode::serialize(&bytecode).unwrap();

                        // dont write bytecode if caching is disabled
                        if !self.settings.no_bytecode_cache {
                            let _ = std::fs::create_dir(import_base.join(".spwnc"));
                            std::fs::write(&spwnc_path, bytes).unwrap();
                        }
                    },
                    Err(err) => return Err(err),
                }
            },
            Err(err) => {
                return Err(CompilerError::ImportSyntaxError {
                    is_module,
                    err,
                    area: self.make_area(span),
                })
            },
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
                self.compile_expr(e, scope, builder, ExprType::Normal)?;
            },
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
                    let expr_reg = self.compile_expr(expr, scope, builder, ExprType::Normal)?;

                    builder.copy(expr_reg, var_reg, stmt.span);
                },
                _ => todo!("haha 😂😂😂😂"),
            },
            Statement::AssignOp(left, op, right) => match op {
                AssignOp::Assign => {
                    if let Expression::Var(s) = *left.expr {
                        if self.get_var(s, scope).is_none() {
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
                            let expr_reg =
                                self.compile_expr(right, scope, builder, ExprType::Normal)?;

                            builder.copy(expr_reg, var_reg, stmt.span);

                            return Ok(());
                        }
                    }

                    let into_reg =
                        self.compile_expr(left, scope, builder, ExprType::Assign(stmt.span))?;

                    let expr_reg = self.compile_expr(right, scope, builder, ExprType::Normal)?;

                    builder.copy(expr_reg, into_reg, stmt.span);
                },
                AssignOp::PlusEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.add_eq(a, b, stmt.span)
                },
                AssignOp::MinusEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.sub_eq(a, b, stmt.span)
                },
                AssignOp::MultEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.mult_eq(a, b, stmt.span)
                },
                AssignOp::DivEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.div_eq(a, b, stmt.span)
                },
                AssignOp::ModEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.modulo_eq(a, b, stmt.span)
                },
                AssignOp::PowEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.pow_eq(a, b, stmt.span)
                },
                AssignOp::ShiftLeftEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.shl_eq(a, b, stmt.span)
                },
                AssignOp::ShiftRightEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.shr_eq(a, b, stmt.span)
                },
                AssignOp::BinAndEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.bin_and_eq(a, b, stmt.span)
                },
                AssignOp::BinOrEq => {
                    let a = self.compile_expr(left, scope, builder, ExprType::Normal)?;
                    let b = self.compile_expr(right, scope, builder, ExprType::Normal)?;
                    builder.bin_or_eq(a, b, stmt.span)
                },
            },

            Statement::While { cond, code } => {
                builder.block(|b| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::Loop(b.block_path.clone())));

                    let cond_reg = self.compile_expr(cond, scope, b, ExprType::Normal)?;
                    b.exit_if_false(cond_reg, cond.span);

                    self.compile_stmts(code, inner_scope, b)?;

                    b.repeat_block();
                    Ok(())
                })?;
            },

            Statement::For {
                iter_var,
                iterator,
                code,
            } => {
                let iter_exec = self.compile_expr(iterator, scope, builder, ExprType::Normal)?;
                let iter_reg = builder.next_reg();
                builder.wrap_iterator(iter_exec, iter_reg, iterator.span);

                builder.block(|b| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::Loop(b.block_path.clone())));

                    let next_reg = b.next_reg();
                    b.iter_next(iter_reg, next_reg, iter_var.span);

                    b.unwrap_or_exit(next_reg, iterator.span);

                    match &*iter_var.expr {
                        Expression::Var(s) => {
                            let var_reg = b.next_reg();
                            self.new_var(
                                *s,
                                Variable {
                                    mutable: false,
                                    def_span: iter_var.span,
                                    reg: var_reg,
                                },
                                inner_scope,
                            );
                            b.copy(next_reg, var_reg, iter_var.span);
                        },
                        _ => todo!("haha 😂😂😂😂"),
                    }

                    self.compile_stmts(code, inner_scope, b)?;

                    b.repeat_block();
                    Ok(())
                })?;
            },

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
                            let cond_reg = self.compile_expr(cond, scope, b, ExprType::Normal)?;
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
            },
            Statement::Break => match self.is_inside_loop(scope) {
                Some(path) => {
                    self.assert_can_break_loop(scope, self.make_area(stmt.span))?;
                    builder.exit_other_block(path.clone())
                },
                _ => {
                    return Err(CompilerError::BreakOutsideLoop {
                        area: self.make_area(stmt.span),
                    })
                },
            },
            Statement::Continue => match self.is_inside_loop(scope) {
                Some(path) => {
                    self.assert_can_break_loop(scope, self.make_area(stmt.span))?;
                    builder.repeat_other_block(path.clone())
                },
                _ => {
                    return Err(CompilerError::ContinueOutsideLoop {
                        area: self.make_area(stmt.span),
                    })
                },
            },
            Statement::TryCatch { try_code, branches } => {
                /*

                #[
                    Message: "Type mismatch", Note: None;
                    Labels: [
                        area => "Expected {}, found {}": expected.runtime_display(vm), v.0.runtime_display(vm);
                        v.1 => "Value defined as {} here": v.0.runtime_display(vm);
                    ]
                ]
                TypeMismatch {
                    v: (ValueType, CodeArea),
                    area: CodeArea,
                    expected: ValueType,
                    [call_stack]
                },

                class NameError(Exception)

                type @error

                impl @error {
                    TYPE_MISMATCH: 0
                }

                try {

                } catch @error::TYPE_MISMATCH {

                } catch

                .to_error() -> Value

                Error(@error::TYPE_MISMATCH)


                try {

                    type @error

                    impl @error {
                        TYPE_MISMATCH: @error::new()

                        code:
                        message:
                        description:
                        ...

                        new: () {}
                    }

                    <!!! 😡 TYPE_MISMATCH 😡 !!!>

                } catch is @error::TYPE_MISMATCH {
                    @error >SEX< >.< OwO :() :)))))))))))))))) :( :) )))) ) :(  : ) P : : )) )))) ) ) )
                }

                */
            },
            Statement::Arrow(statement) => {
                builder.block(|b| {
                    let inner_scope = self
                        .derive_scope(scope, Some(ScopeType::ArrowStmt(self.make_area(stmt.span)))); // variables made in arrow statements shouldnt be allowed anyways
                    b.enter_arrow();
                    self.compile_stmt(statement, inner_scope, b)?;
                    b.yeet_context();
                    Ok(())
                })?;
            },
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

                                let ret_reg =
                                    self.compile_expr(node, scope, builder, ExprType::Normal)?;
                                self.global_return =
                                    Some((items.iter().map(|i| i.0).collect(), stmt.span));
                                builder.ret(ret_reg, true, stmt.span);
                            },
                            _ => {
                                return Err(CompilerError::InvalidModuleReturn {
                                    area: self.make_area(stmt.span),
                                })
                            },
                        },
                        None => {
                            return Err(CompilerError::InvalidModuleReturn {
                                area: self.make_area(stmt.span),
                            })
                        },
                    }
                } else if self.is_inside_macro(scope) {
                    self.assert_can_return(scope, self.make_area(stmt.span))?;
                    match value {
                        None => {
                            let out_reg = builder.next_reg();
                            builder.load_empty(out_reg, stmt.span);
                            builder.ret(out_reg, false, stmt.span)
                        },
                        Some(expr) => {
                            let ret_reg =
                                self.compile_expr(expr, scope, builder, ExprType::Normal)?;
                            builder.ret(ret_reg, false, stmt.span)
                        },
                    }
                } else {
                    return Err(CompilerError::ReturnOutsideMacro {
                        area: self.make_area(stmt.span),
                    });
                }
            },
            Statement::TypeDef { name, private } => {
                if !matches!(self.scopes[scope].typ, Some(ScopeType::Global)) {
                    return Err(CompilerError::TypeDefNotGlobal {
                        area: self.make_area(stmt.span),
                    });
                }

                let info = TypeDef {
                    def_src: self.src.clone(),
                    name: *name,
                    private: *private,
                };

                if ValueType::VARIANT_NAMES.contains(&self.resolve(name).as_str()) {
                    return Err(CompilerError::BuiltinTypeOverride {
                        area: self.make_area(stmt.span),
                    });
                } else if self.custom_type_defs.contains_key(&info) {
                    return Err(CompilerError::DuplicateTypeDef {
                        area: self.make_area(stmt.span),
                        prev_area: self.make_area(self.custom_type_defs[&info].span),
                    });
                } else if self.available_custom_types.contains_key(name) {
                    // TODO test
                    return Err(CompilerError::DuplicateImportedType {
                        area: self.make_area(stmt.span),
                    });
                }

                let k = builder.create_type(self.resolve(name), *private, stmt.span);

                self.custom_type_defs.insert(info, k.spanned(stmt.span));
                self.available_custom_types.insert(*name, k);
            },
            Statement::Impl { base, items } => {
                let dict_reg = builder.next_reg();
                self.build_dict(builder, items, dict_reg, scope, stmt.span)?;

                let base_reg = self.compile_expr(base, scope, builder, ExprType::Normal)?;

                builder.do_impl(base_reg, dict_reg, stmt.span);
            },
            Statement::Overload { op, macros } => {
                let array_reg = builder.next_reg();

                builder.new_array(
                    macros.len() as u16,
                    array_reg,
                    |builder, elems| {
                        for e in macros {
                            match &*e.expr {
                                Expression::Macro { args, .. } => {
                                    match op {
                                        Operator::Bin(_) | Operator::Assign(_) => {
                                            if args.len() != 2 {
                                                return Err(CompilerError::InvalidOverload {
                                                    expected: "macro with 2 arguments".into(),
                                                    area: self.make_area(e.span),
                                                });
                                            }
                                        },
                                        Operator::Unary(_) => {
                                            if args.len() != 1 {
                                                return Err(CompilerError::InvalidOverload {
                                                    expected: "macro with 1 argument".into(),
                                                    area: self.make_area(e.span),
                                                });
                                            }
                                        },
                                    }
                                    for arg in args {
                                        let err = match arg {
                                            MacroArg::Spread { .. } => {
                                                Some("macro with no spreads")
                                            },
                                            MacroArg::Single {
                                                default: Some(_), ..
                                            } => Some("macro with no defaults"),
                                            MacroArg::Single { pattern: None, .. } => {
                                                Some("macro with explicit patterns")
                                            },
                                            _ => None,
                                        };
                                        if let Some(msg) = err {
                                            return Err(CompilerError::InvalidOverload {
                                                expected: msg.into(),
                                                area: self.make_area(e.span),
                                            });
                                        }
                                    }
                                },
                                _ => {
                                    return Err(CompilerError::InvalidOverload {
                                        expected: "macro expression".into(),
                                        area: self.make_area(e.span),
                                    })
                                },
                            }

                            elems.push((
                                self.compile_expr(e, scope, builder, ExprType::Normal)?,
                                false,
                            ));
                        }

                        Ok(())
                    },
                    stmt.span,
                )?;

                builder.do_overload(array_reg, *op, stmt.span);
            },
            Statement::ExtractImport(_) => todo!(),
            Statement::Dbg(v) => {
                let v = self.compile_expr(v, scope, builder, ExprType::Normal)?;
                builder.dbg(v);
            },
            Statement::Throw(err) => {
                let out_reg = builder.next_reg();

                builder.load_string(self.resolve(err), out_reg, stmt.span);
                builder.throw(out_reg, stmt.span);
            },
        }
        Ok(())
    }

    pub fn compile_pattern_check(
        &mut self,
        expr_reg: UnoptRegister,
        pattern: &PatternNode,
        scope: ScopeKey,
        builder: &mut FuncBuilder,
    ) -> CompileResult<UnoptRegister> {
        let out_reg = builder.next_reg();

        match &*pattern.pat {
            Pattern::Any => builder.load_bool(true, out_reg, pattern.span),
            Pattern::Type(t) => {
                let t_reg = self.compile_expr(
                    &ExprNode {
                        expr: Box::new(Expression::Type(*t)),
                        attributes: vec![],
                        span: pattern.span,
                    },
                    scope,
                    builder,
                    ExprType::Normal,
                )?;
                let typeof_reg = builder.next_reg();
                builder.type_of(expr_reg, typeof_reg, pattern.span);

                builder.eq(t_reg, typeof_reg, out_reg, pattern.span);
            },
            Pattern::Either(a, b) => {
                let left = self.compile_pattern_check(expr_reg, a, scope, builder)?;
                let right = self.compile_pattern_check(expr_reg, b, scope, builder)?;

                builder.or(left, right, out_reg, pattern.span);
            },
            Pattern::Both(a, b) => {
                let left = self.compile_pattern_check(expr_reg, a, scope, builder)?;
                let right = self.compile_pattern_check(expr_reg, b, scope, builder)?;

                builder.and(left, right, out_reg, pattern.span);
            },
            Pattern::Eq(val) => {
                let val = self.compile_expr(val, scope, builder, ExprType::Normal)?;
                builder.eq(expr_reg, val, out_reg, pattern.span);
            },
            Pattern::Neq(val) => {
                let val = self.compile_expr(val, scope, builder, ExprType::Normal)?;
                builder.neq(expr_reg, val, out_reg, pattern.span);
            },
            Pattern::Lt(val) => {
                let val = self.compile_expr(val, scope, builder, ExprType::Normal)?;
                builder.lt(expr_reg, val, out_reg, pattern.span);
            },
            Pattern::Lte(val) => {
                let val = self.compile_expr(val, scope, builder, ExprType::Normal)?;
                builder.lte(expr_reg, val, out_reg, pattern.span);
            },
            Pattern::Gt(val) => {
                let val = self.compile_expr(val, scope, builder, ExprType::Normal)?;
                builder.gt(expr_reg, val, out_reg, pattern.span);
            },
            Pattern::Gte(val) => {
                let val = self.compile_expr(val, scope, builder, ExprType::Normal)?;
                builder.gte(expr_reg, val, out_reg, pattern.span);
            },
            Pattern::MacroPattern { args, ret_type } => {
                // for pattern in args {
                // let p = self.compile_pattern_check(expr_reg, pattern, scope, builder)?;
                todo!()
                // }

                // let ret = self.compile_pattern_check(expr_reg, ret_type, scope, builder)?;
            },
        };

        Ok(out_reg)
    }

    pub fn convert_const_expr(&mut self, expr: &ExprNode) -> CompileResult<Constant> {
        Ok(match &*expr.expr {
            Expression::Int(v) => Constant::Int(*v),
            Expression::Float(v) => Constant::Float(*v),
            Expression::String(v) => {
                let content = match v.s {
                    StringContent::Normal(s) => self.resolve(&s),
                };
                if v.bytes {
                    Constant::Array(content.bytes().map(|b| Constant::Int(b as i64)).collect())
                } else {
                    Constant::String(content)
                }
            },
            Expression::Bool(v) => Constant::Bool(*v),
            Expression::Id(class, Some(id)) => Constant::Id(*class, *id),
            Expression::Type(t) => {
                let name = self.resolve(t);
                if ValueType::VARIANT_NAMES.contains(&name.as_str()) {
                    Constant::Type(ValueType::from_str(&name).unwrap())
                } else {
                    match self.available_custom_types.get(t) {
                        Some(k) => Constant::Type(ValueType::Custom(*k)),
                        None => {
                            return Err(CompilerError::NonexistentType {
                                area: self.make_area(expr.span),
                                type_name: name,
                            })
                        },
                    }
                }
            },
            Expression::Array(arr) => {
                let mut v = vec![];
                for i in arr {
                    v.push(self.convert_const_expr(i)?)
                }
                Constant::Array(v)
            },
            Expression::Dict(map) if map.iter().all(|(_, v, _)| v.is_some()) => {
                let mut m = AHashMap::new();
                for (k, v, _) in map {
                    m.insert(
                        self.resolve(&k.value),
                        self.convert_const_expr(&v.clone().unwrap())?,
                    );
                }
                Constant::Dict(m)
            },
            Expression::Maybe(o) => Constant::Maybe(match o {
                Some(e) => Some(Box::new(self.convert_const_expr(e)?)),
                None => None,
            }),
            Expression::Builtins => Constant::Builtins,
            Expression::Empty => Constant::Empty,
            Expression::Instance { base, items } => {
                let t = self.convert_const_expr(base)?;
                let t = match t {
                    Constant::Type(t) => t,
                    _ => todo!(),
                };
                let k = if let ValueType::Custom(k) = t {
                    k
                } else {
                    todo!()
                };

                let mut m = AHashMap::new();
                for (k, v, _) in items {
                    m.insert(
                        self.resolve(&k.value),
                        Box::new(self.convert_const_expr(&v.clone().unwrap())?),
                    );
                }
                Constant::Instance(k, m)
            },
            Expression::Obj(..) => todo!(),
            _ => {
                return Err(CompilerError::ExpectedConstantExpr {
                    area: self.make_area(expr.span),
                })
            },
        })
    }

    pub fn convert_const_pattern(&mut self, pat: &PatternNode) -> CompileResult<ConstPattern> {
        Ok(ConstPattern {
            pat: match &*pat.pat {
                Pattern::Any => Pattern::Any,
                Pattern::Type(t) => {
                    let name = self.resolve(t);
                    let t = if ValueType::VARIANT_NAMES.contains(&name.as_str()) {
                        ValueType::from_str(&name).unwrap()
                    } else {
                        match self.available_custom_types.get(t) {
                            Some(k) => ValueType::Custom(*k),
                            None => {
                                return Err(CompilerError::NonexistentType {
                                    area: self.make_area(pat.span),
                                    type_name: name,
                                })
                            },
                        }
                    };
                    Pattern::Type(t)
                },
                Pattern::Either(a, b) => Pattern::Either(
                    Box::new(self.convert_const_pattern(a)?),
                    Box::new(self.convert_const_pattern(b)?),
                ),
                Pattern::Both(a, b) => Pattern::Both(
                    Box::new(self.convert_const_pattern(a)?),
                    Box::new(self.convert_const_pattern(b)?),
                ),
                Pattern::Eq(v) => Pattern::Eq(self.convert_const_expr(v)?.spanned(pat.span)),
                Pattern::Neq(v) => Pattern::Neq(self.convert_const_expr(v)?.spanned(pat.span)),
                Pattern::Lt(v) => Pattern::Lt(self.convert_const_expr(v)?.spanned(pat.span)),
                Pattern::Lte(v) => Pattern::Lte(self.convert_const_expr(v)?.spanned(pat.span)),
                Pattern::Gt(v) => Pattern::Gt(self.convert_const_expr(v)?.spanned(pat.span)),
                Pattern::Gte(v) => Pattern::Gte(self.convert_const_expr(v)?.spanned(pat.span)),
                Pattern::MacroPattern { args, ret_type } => todo!(),
            },
        })
    }

    pub fn compile_expr(
        &mut self,
        expr: &ExprNode,
        scope: ScopeKey,
        builder: &mut FuncBuilder,
        expr_type: ExprType,
    ) -> CompileResult<UnoptRegister> {
        let out_reg = builder.next_reg();

        macro_rules! bin_op {
            ($left:ident $fn:ident $right:ident) => {{
                let a = self.compile_expr(&$left, scope, builder, ExprType::Normal)?;
                let b = self.compile_expr(&$right, scope, builder, ExprType::Normal)?;
                builder.$fn(a, b, out_reg, expr.span)
            }};
        }

        match &*expr.expr {
            Expression::Int(v) => builder.load_int(*v, out_reg, expr.span),
            Expression::Float(v) => builder.load_float(*v, out_reg, expr.span),
            Expression::Bool(v) => builder.load_bool(*v, out_reg, expr.span),
            Expression::String(v) => {
                match v.s {
                    StringContent::Normal(s) => {
                        builder.load_string(self.resolve(&s), out_reg, expr.span);
                    },
                }
                if v.bytes {
                    builder.make_byte_array(out_reg, expr.span);
                }
            },
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
                },
                BinOp::ShiftRight => {
                    bin_op!(left shr right)
                },
                BinOp::BinAnd => {
                    bin_op!(left bin_and right)
                },
                BinOp::BinOr => {
                    bin_op!(left bin_or right)
                },
                BinOp::Eq => {
                    bin_op!(left eq right)
                },
                BinOp::Neq => {
                    bin_op!(left neq right)
                },
                BinOp::Gt => {
                    bin_op!(left gt right)
                },
                BinOp::Gte => {
                    bin_op!(left gte right)
                },
                BinOp::Lt => {
                    bin_op!(left lt right)
                },
                BinOp::Lte => {
                    bin_op!(left lte right)
                },
                BinOp::Range => {
                    bin_op!(left range right)
                },
                BinOp::In => {
                    bin_op!(left in_op right)
                },
                BinOp::As => {
                    bin_op!(left as_op right)
                },
                BinOp::Or => bin_op!(left or right),
                BinOp::And => bin_op!(left and right),
            },
            Expression::Unary(op, value) => {
                let v = self.compile_expr(value, scope, builder, ExprType::Normal)?;
                match op {
                    UnaryOp::BinNot => builder.unary_bin_not(v, out_reg, expr.span),
                    UnaryOp::ExclMark => builder.unary_not(v, out_reg, expr.span),
                    UnaryOp::Minus => builder.unary_negate(v, out_reg, expr.span),
                    // UnaryOp::Eq => builder.unary_pat_eq(v, out_reg, expr.span),
                    // UnaryOp::Neq => builder.unary_pat_neq(v, out_reg, expr.span),
                    // UnaryOp::Gt => builder.unary_pat_gt(v, out_reg, expr.span),
                    // UnaryOp::Gte => builder.unary_pat_gte(v, out_reg, expr.span),
                    // UnaryOp::Lt => builder.unary_pat_lt(v, out_reg, expr.span),
                    // UnaryOp::Lte => builder.unary_pat_lte(v, out_reg, expr.span),
                }
            },
            Expression::Var(name) => match self.get_var(*name, scope) {
                Some(data) => {
                    if let ExprType::Assign(stmt_span) = expr_type {
                        if !data.mutable {
                            return Err(CompilerError::ImmutableAssign {
                                area: self.make_area(stmt_span),
                                def_area: self.make_area(data.def_span),
                                var: self.resolve(name),
                            });
                        }
                    }

                    return Ok(data.reg);
                },
                None => {
                    return Err(CompilerError::NonexistentVariable {
                        area: self.make_area(expr.span),
                        var: self.resolve(name),
                    })
                },
            },
            Expression::Type(t) => {
                let name = self.resolve(t);
                if ValueType::VARIANT_NAMES.contains(&name.as_str()) {
                    builder.load_builtin_type(&name, out_reg, expr.span)
                } else {
                    match self.available_custom_types.get(t) {
                        Some(k) => builder.load_custom_type(*k, out_reg, expr.span),
                        None => {
                            return Err(CompilerError::NonexistentType {
                                area: self.make_area(expr.span),
                                type_name: name,
                            })
                        },
                    }
                }
            },
            Expression::Array(items) => {
                builder.new_array(
                    items.len() as u16,
                    out_reg,
                    |builder, elems| {
                        for item in items {
                            elems.push((
                                self.compile_expr(item, scope, builder, ExprType::Normal)?,
                                false,
                            ));
                        }
                        Ok(())
                    },
                    expr.span,
                )?;
            },
            Expression::Dict(items) => {
                self.build_dict(builder, items, out_reg, scope, expr.span)?;
            },
            Expression::Instance { base, items } => {
                let dict_reg = builder.next_reg();
                self.build_dict(builder, items, dict_reg, scope, expr.span)?;

                let base_reg = self.compile_expr(base, scope, builder, expr_type)?;

                builder.create_instance(base_reg, dict_reg, out_reg, expr.span);
            },
            Expression::Maybe(e) => match e {
                Some(e) => {
                    let value = self.compile_expr(e, scope, builder, ExprType::Normal)?;
                    builder.wrap_maybe(value, out_reg, expr.span)
                },
                None => builder.load_none(out_reg, expr.span),
            },
            Expression::Index { base, index } => {
                let base_reg = self.compile_expr(base, scope, builder, expr_type)?;
                let index_reg = self.compile_expr(index, scope, builder, ExprType::Normal)?;
                builder.index(base_reg, out_reg, index_reg, expr.span)
            },
            Expression::Member { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder, expr_type)?;
                builder.member(
                    base_reg,
                    out_reg,
                    self.resolve(&name.value).spanned(name.span),
                    expr.span,
                )
            },
            Expression::TypeMember { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder, expr_type)?;
                builder.type_member(
                    base_reg,
                    out_reg,
                    self.resolve(&name.value).spanned(name.span),
                    expr.span,
                )
            },
            Expression::Associated { base, name } => {
                let base_reg = self.compile_expr(base, scope, builder, expr_type)?;
                builder.associated(
                    base_reg,
                    out_reg,
                    self.resolve(&name.value).spanned(name.span),
                    expr.span,
                )
            },
            Expression::Is(val, pat) => {
                let reg = self.compile_expr(val, scope, builder, expr_type)?;
                let out = self.compile_pattern_check(reg, pat, scope, builder)?;
                builder.copy(out, out_reg, expr.span);
            },
            Expression::Macro {
                args,
                ret_type,
                code,
            } => {
                let func_id = builder.new_func(
                    |f| {
                        let mut variables = AHashMap::new();

                        let mut ref_arg_regs = vec![];

                        for a in args {
                            let reg = f.next_reg();
                            let name = a.name().value;

                            let is_by_ref = name == self.intern("self")
                                || matches!(a, MacroArg::Single { is_ref: true, .. });

                            variables.insert(
                                name,
                                Variable {
                                    mutable: is_by_ref,
                                    def_span: a.name().span,
                                    reg,
                                },
                            );
                            if is_by_ref {
                                ref_arg_regs.push(reg)
                            }
                        }

                        let to_capture = self.get_accessible_vars(scope);
                        let mut capture_regs = vec![];

                        for (name, data) in to_capture {
                            if !variables.contains_key(&name) {
                                let reg = f.next_reg();
                                capture_regs.push((data.reg, reg));
                                variables.insert(name, Variable { reg, ..data });
                            }
                        }

                        let base_scope = self.scopes.insert(Scope {
                            parent: None,
                            variables,
                            typ: Some(ScopeType::MacroBody),
                        });

                        match code {
                            MacroCode::Normal(stmts) => self.compile_stmts(stmts, base_scope, f)?,
                            MacroCode::Lambda(expr) => {
                                let ret_reg =
                                    self.compile_expr(expr, base_scope, f, ExprType::Normal)?;
                                f.ret(ret_reg, false, expr.span);
                            },
                        }

                        Ok((capture_regs, ref_arg_regs))
                    },
                    args.len(),
                    expr.span,
                )?;

                builder.new_macro(
                    func_id,
                    out_reg,
                    |builder, elems| {
                        for arg in args {
                            match arg {
                                MacroArg::Single {
                                    name,
                                    pattern,
                                    default,
                                    is_ref,
                                } => {
                                    let n = self.resolve(&name.value).spanned(name.span);

                                    let p = if let Some(p) = pattern {
                                        Some(self.convert_const_pattern(p)?.spanned(p.span))
                                    } else {
                                        None
                                    };
                                    let d = if let Some(d) = default {
                                        Some(self.compile_expr(
                                            d,
                                            scope,
                                            builder,
                                            ExprType::Normal,
                                        )?)
                                    } else {
                                        None
                                    };

                                    elems.push(MacroArg::Single {
                                        name: n,
                                        pattern: p,
                                        default: d,
                                        is_ref: *is_ref,
                                    })
                                },
                                MacroArg::Spread { name, pattern } => {
                                    let n = self.resolve(&name.value).spanned(name.span);

                                    let p = if let Some(p) = pattern {
                                        Some(self.convert_const_pattern(p)?.spanned(p.span))
                                    } else {
                                        None
                                    };

                                    elems.push(MacroArg::Spread {
                                        name: n,
                                        pattern: p,
                                    })
                                },
                            }
                        }
                        Ok(())
                    },
                    expr.span,
                )?;
            },
            Expression::Call {
                base,
                params,
                named_params,
            } => {
                let base_reg = self.compile_expr(base, scope, builder, ExprType::Normal)?;
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
                                    elems.push((
                                        self.compile_expr(i, scope, builder, ExprType::Normal)?,
                                        true,
                                    ));
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
                                    let value_reg =
                                        self.compile_expr(param, scope, builder, ExprType::Normal)?;

                                    elems.push((
                                        self.resolve(&name.value).spanned(name.span),
                                        value_reg,
                                        true,
                                        false,
                                    ));
                                }
                                Ok(())
                            },
                            expr.span,
                        )?;

                        elems.push((params_reg, true));
                        elems.push((named_params_reg, true));

                        Ok(())
                    },
                    expr.span,
                )?;
                builder.call(base_reg, out_reg, args_reg, expr.span);
            },
            // Expression::MacroPattern { args, ret_type } => todo!(),
            Expression::TriggerFunc { attributes, code } => {
                use crate::gd::ids::IDClass::Group;
                let group_reg = builder.next_reg();
                builder.load_id(None, Group, group_reg, expr.span);
                builder.make_trigger_function(group_reg, out_reg, expr.span);

                builder.push_context_group(group_reg, expr.span);
                builder.block(|b| {
                    let inner_scope = self.derive_scope(
                        scope,
                        Some(ScopeType::TriggerFunc(self.make_area(expr.span))),
                    );
                    self.compile_stmts(code, inner_scope, b)
                })?;
                builder.pop_context_group(out_reg, expr.span);
            },
            Expression::TriggerFuncCall(e) => {
                let reg = self.compile_expr(e, scope, builder, ExprType::Normal)?;
                builder.call_trigger_func(reg);
            },
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => {
                let cond_reg = self.compile_expr(cond, scope, builder, ExprType::Normal)?;
                builder.block(|outer_b| {
                    let outer_path = outer_b.block_path.clone();

                    outer_b.block(|b| {
                        b.exit_if_false(cond_reg, cond.span);
                        let reg = self.compile_expr(if_true, scope, b, ExprType::Normal)?;
                        b.copy(reg, out_reg, expr.span);

                        b.exit_other_block(outer_path);
                        Ok(())
                    })?;

                    let reg = self.compile_expr(if_false, scope, outer_b, ExprType::Normal)?;
                    outer_b.copy(reg, out_reg, expr.span);

                    Ok(())
                })?;
            },
            Expression::Typeof(e) => {
                let reg = self.compile_expr(e, scope, builder, ExprType::Normal)?;
                builder.type_of(reg, out_reg, expr.span);
            },
            Expression::Builtins => {
                builder.load_builtins(out_reg, expr.span);
            },
            Expression::Empty => {
                builder.load_empty(out_reg, expr.span);
            },
            Expression::Epsilon => {
                builder.load_epsilon(out_reg, expr.span);
            },
            // Expression::AnyPattern => {
            //     builder.load_any(out_reg, expr.span);
            // }
            Expression::Import(t) => {
                self.compile_import(t, expr.span, self.src.clone())?;
                builder.import(out_reg, t.clone().spanned(expr.span), expr.span)
            },

            Expression::Obj(typ, items) => {
                builder.new_object(
                    items.len() as u16,
                    out_reg,
                    |builder, elems| {
                        for (key, expr) in items {
                            let value_reg =
                                self.compile_expr(expr, scope, builder, ExprType::Normal)?;

                            elems.push((*key, value_reg));
                        }

                        Ok(())
                    },
                    expr.span,
                    *typ,
                )?;
            },
        }

        Ok(out_reg)
    }

    fn build_dict(
        &mut self,
        builder: &mut FuncBuilder,
        items: &DictItems,
        out_reg: usize,
        scope: ScopeKey,
        span: CodeSpan,
    ) -> Result<(), CompilerError> {
        builder.new_dict(
            items.len() as u16,
            out_reg,
            |builder, elems| {
                for (key, item, private) in items {
                    let value_reg = match item {
                        Some(e) => self.compile_expr(e, scope, builder, ExprType::Normal)?,
                        None => match self.get_var(key.value, scope) {
                            Some(data) => data.reg,
                            None => {
                                return Err(CompilerError::NonexistentVariable {
                                    area: self.make_area(key.span),
                                    var: self.resolve(&key.value),
                                })
                            },
                        },
                    };

                    elems.push((
                        self.resolve(&key.value).spanned(key.span),
                        value_reg,
                        false,
                        *private,
                    ));
                }
                Ok(())
            },
            span,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Normal,
    Assign(CodeSpan),
    // Match,
}

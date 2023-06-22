use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::str::FromStr;

use ahash::{AHashMap, AHasher};
use delve::VariantNames;
use itertools::Itertools;
use lasso::Spur;
use serde::{Deserialize, Serialize};

use super::builder::{BlockID, CodeBuilder, JumpType};
use super::bytecode::{Bytecode, UnoptRegister};
use super::error::CompileError;
use crate::cli::Settings;
use crate::compiling::builder::ProtoBytecode;
use crate::compiling::opcodes::Opcode;
use crate::interpreting::value::ValueType;
use crate::new_id_wrapper;
use crate::parsing::ast::{
    Ast, DictItem, ExprNode, Expression, Import, ImportType, MatchBranch, Statement, StmtNode, Vis,
    VisTrait,
};
use crate::parsing::parser::Parser;
use crate::parsing::utils::operators::{AssignOp, BinOp, UnaryOp};
use crate::sources::{BytecodeMap, CodeArea, CodeSpan, Spannable, Spanned, SpwnSource};
use crate::util::{ImmutCloneStr, ImmutCloneVec, ImmutStr, ImmutVec, Interner, SlabMap};

pub type CompileResult<T> = Result<T, CompileError>;

new_id_wrapper! {
    ScopeID: u16;
    LocalTypeID: u16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CustomTypeID {
    pub local: LocalTypeID,
    pub source_hash: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct VarData {
    mutable: bool,
    def_span: CodeSpan,
    reg: UnoptRegister,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeType {
    Global,
    Loop(BlockID),
    MacroBody,
    TriggerFunc(CodeSpan),
    ArrowStmt(CodeSpan),
}

#[derive(Debug, Clone)]
pub struct Scope {
    vars: AHashMap<Spur, VarData>,
    parent: Option<ScopeID>,
    typ: Option<ScopeType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignType {
    Normal { is_let: bool },
    Match,
}

impl AssignType {
    fn is_declare(&self) -> bool {
        matches!(
            self,
            AssignType::Match | AssignType::Normal { is_let: true }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeDef {
    pub def_span: CodeSpan,
    pub name: Spur,
}

pub struct Compiler<'a> {
    src: Rc<SpwnSource>,
    interner: Rc<RefCell<Interner>>,
    scopes: SlabMap<ScopeID, Scope>,
    pub global_return: Option<Spanned<ImmutVec<Spanned<Spur>>>>,

    settings: &'a Settings,
    bytecode_map: &'a mut BytecodeMap,

    pub custom_type_defs: SlabMap<LocalTypeID, Vis<TypeDef>>,
    available_custom_types: AHashMap<Spur, CustomTypeID>,
}

impl<'a> Compiler<'a> {
    pub fn new(
        src: Rc<SpwnSource>,
        settings: &'a Settings,
        bytecode_map: &'a mut BytecodeMap,
        interner: Rc<RefCell<Interner>>,
    ) -> Self {
        Self {
            src,
            interner,
            scopes: SlabMap::new(),
            global_return: None,
            custom_type_defs: SlabMap::new(),
            available_custom_types: AHashMap::new(),
            settings,
            bytecode_map,
        }
    }
}

impl Compiler<'_> {
    pub fn make_area(&self, span: CodeSpan) -> CodeArea {
        CodeArea {
            span,
            src: Rc::clone(&self.src),
        }
    }

    fn intern(&self, s: &str) -> Spur {
        self.interner.borrow_mut().get_or_intern(s)
    }

    pub fn src_hash(&self) -> u32 {
        let mut hasher = DefaultHasher::default();
        self.src.hash(&mut hasher);
        let h = hasher.finish();
        (h % (u32::MAX as u64)) as u32
    }

    pub fn resolve(&self, s: &Spur) -> ImmutStr {
        self.interner.borrow().resolve(s).into()
    }

    fn resolve_arr(&self, s: &Spur) -> ImmutVec<char> {
        self.interner
            .borrow()
            .resolve(s)
            .chars()
            .collect_vec()
            .into()
    }

    pub fn derive_scope(&mut self, scope: ScopeID, typ: Option<ScopeType>) -> ScopeID {
        let scope = Scope {
            vars: AHashMap::new(),
            parent: Some(scope),
            typ,
        };
        self.scopes.insert(scope)
    }

    pub fn get_var(&self, var: Spur, scope: ScopeID) -> Option<VarData> {
        match self.scopes[scope].vars.get(&var) {
            Some(v) => Some(*v),
            None => match self.scopes[scope].parent {
                Some(p) => self.get_var(var, p),
                None => None,
            },
        }
    }

    fn is_inside_macro(&self, scope: ScopeID) -> bool {
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

    fn is_inside_loop(&self, scope: ScopeID) -> Option<BlockID> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(ScopeType::ArrowStmt(_) | ScopeType::TriggerFunc(_)) | None => {
                match scope.parent {
                    Some(k) => self.is_inside_loop(k),
                    None => None,
                }
            },
            Some(t) => match t {
                ScopeType::Loop(v) => Some(*v),
                _ => None,
            },
        }
    }

    pub fn assert_can_return(&self, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
        fn can_return_d(slf: &Compiler, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
            let scope = &slf.scopes[scope];
            match &scope.typ {
                Some(ScopeType::MacroBody) => return Ok(()),
                Some(ScopeType::TriggerFunc(def)) => {
                    return Err(CompileError::BreakInTriggerFuncScope {
                        area: slf.make_area(span),
                        def: slf.make_area(*def),
                    })
                },
                Some(ScopeType::ArrowStmt(def)) => {
                    return Err(CompileError::BreakInArrowStmtScope {
                        area: slf.make_area(span),
                        def: slf.make_area(*def),
                    })
                },
                Some(_) => (),
                None => (),
            }
            match scope.parent {
                Some(k) => can_return_d(slf, k, span),
                None => unreachable!(),
            }
        }

        if let Some(ScopeType::ArrowStmt(_)) = self.scopes[scope].typ {
            return Ok(()); // -> return
        }

        can_return_d(self, scope, span)
    }

    pub fn assert_can_break_loop(&self, scope: ScopeID, span: CodeSpan) -> CompileResult<()> {
        let scope = &self.scopes[scope];
        match &scope.typ {
            Some(t) => match t {
                ScopeType::Loop(_) => return Ok(()),
                ScopeType::TriggerFunc(def) => {
                    return Err(CompileError::BreakInTriggerFuncScope {
                        area: self.make_area(span),
                        def: self.make_area(*def),
                    })
                },
                ScopeType::ArrowStmt(def) => {
                    return Err(CompileError::BreakInArrowStmtScope {
                        area: self.make_area(span),
                        def: self.make_area(*def),
                    })
                },

                _ => (),
            },
            None => (),
        }
        match scope.parent {
            Some(k) => self.assert_can_break_loop(k, span),
            None => unreachable!(), // should only be called after is_inside_loop
        }
    }

    pub fn compile_import(
        &mut self,
        import: &Import,
        span: CodeSpan,
        importer_src: Rc<SpwnSource>,
    ) -> CompileResult<(
        ImmutVec<ImmutStr>,
        SpwnSource,
        ImmutVec<(CustomTypeID, Spur)>,
    )> {
        let base_dir = importer_src.path().parent().unwrap();
        let mut path = base_dir.to_path_buf();

        match import.settings.typ {
            ImportType::File => path.push(&import.path),
            ImportType::Library => {
                path.push("libraries");
                path.push(&import.path);
                path.push("lib.spwn");
            },
        };

        let is_file = matches!(import.settings.typ, ImportType::File);

        let new_src = Rc::new(SpwnSource::File(path.clone()));

        let import_name = path.file_name().unwrap().to_str().unwrap();
        let import_base = path.parent().unwrap();
        let spwnc_path = import_base.join(format!(".spwnc/{import_name}.spwnc"));

        let code = match new_src.read() {
            Some(c) => c,
            None => {
                return Err(CompileError::NonexistentImport {
                    is_file,
                    name: import.path.to_str().unwrap().into(),
                    area: self.make_area(span),
                })
            },
        };

        let mut parser: Parser<'_> =
            Parser::new(&code, Rc::clone(&new_src), Rc::clone(&self.interner));

        let ast = parser
            .parse()
            .map_err(|e| CompileError::ImportSyntaxError {
                is_file,
                err: e,
                area: self.make_area(span),
            })?;

        let mut compiler = Compiler::new(
            Rc::clone(&new_src),
            self.settings,
            self.bytecode_map,
            Rc::clone(&self.interner),
        );

        compiler.compile(&ast, (0..code.len()).into())?;

        let export_names = compiler.bytecode_map[&new_src].export_names.clone();
        let custom_types = compiler.bytecode_map[&new_src]
            .custom_types
            .iter()
            .filter(|(_, v)| v.is_pub())
            .map(|(id, s)| (*id, compiler.intern(&s.value().value)))
            .collect();

        let bytes = bincode::serialize(&self.bytecode_map[&new_src]).unwrap();

        // dont write bytecode if caching is disabled
        if !self.settings.no_bytecode_cache {
            let _ = std::fs::create_dir(import_base.join(".spwnc"));
            std::fs::write(spwnc_path, bytes).unwrap();
        }

        Ok((export_names, (*new_src).clone(), custom_types))

        // todo: caching
        // 'from_cache: {
        //     if spwnc_path.is_file() {
        //         let source_bytes = std::fs::read(&spwnc_path).unwrap();

        //         let bytecode: Bytecode<Register> = match bincode::deserialize(&source_bytes) {
        //             Ok(b) => b,
        //             Err(_) => {
        //                 break 'from_cache;
        //             },
        //         };

        //         if bytecode.source_hash == hash.into()
        //             && bytecode.spwn_ver == env!("CARGO_PKG_VERSION")
        //         {
        //             for import in &bytecode.import_paths {
        //                 self.compile_import(&import.value, import.span, import_src.clone())?;
        //             }
        //             for (k, (name, private)) in &bytecode.custom_types {
        //                 self.custom_type_defs.insert(
        //                     TypeDef {
        //                         def_src: import_src.clone(),
        //                         name: self.intern(&name.value),
        //                         private: *private,
        //                     },
        //                     k.spanned(name.span),
        //                 );
        //             }

        //             self.map.map.insert(import_src, bytecode);
        //             return Ok(());
        //         }
        //     }
        // }
    }

    pub fn do_assign(
        &mut self,
        left: &ExprNode,
        right: UnoptRegister,
        scope: ScopeID,
        builder: &mut CodeBuilder,
        typ: AssignType,
    ) -> CompileResult<()> {
        macro_rules! destructure_dict {
            ($v:expr) => {
                for e in $v {
                    let item = e.value();

                    // avoid uneccessary allocation
                    let default = if item.value.is_none() {
                        Some(ExprNode {
                            expr: Box::new(Expression::Var(item.name.value)),
                            attributes: vec![],
                            span: item.name.span,
                        })
                    } else {
                        None
                    };

                    let key = match &item.value {
                        Some(e) => e,
                        None => default.as_ref().unwrap(), // use the key as the value ( `{a}` -> `{a: a}` )
                    };

                    let elem_reg = builder.next_reg();

                    if let AssignType::Match = typ {
                        builder.match_catch(None, true, left.span);
                    }
                    builder.member(
                        right,
                        elem_reg,
                        item.name.map(|v| self.resolve_arr(&v)),
                        left.span,
                    );
                    self.do_assign(key, elem_reg, scope, builder, typ)?;
                }
            };
        }

        match &*left.expr {
            Expression::Var(n) => {
                match typ {
                    AssignType::Normal { is_let: false } => match self.get_var(*n, scope) {
                        Some(data) if data.mutable => {
                            builder.copy_deep(right, data.reg, left.span);
                        },
                        Some(data) => {
                            return Err(CompileError::ImmutableAssign {
                                area: self.make_area(left.span),
                                def_area: self.make_area(data.def_span),
                                var: self.resolve(n),
                            })
                        },
                        None => {
                            let reg = builder.next_reg();
                            self.scopes[scope].vars.insert(
                                *n,
                                VarData {
                                    mutable: false,
                                    def_span: left.span,
                                    reg,
                                },
                            );
                            builder.copy_deep(right, reg, left.span);
                        },
                    },
                    _ => {
                        let reg = builder.next_reg();
                        self.scopes[scope].vars.insert(
                            *n,
                            VarData {
                                mutable: true,
                                def_span: left.span,
                                reg,
                            },
                        );
                        builder.copy_deep(right, reg, left.span);
                    },
                }
                Ok(())
            },
            Expression::Array(v) => {
                for (i, e) in v.iter().enumerate() {
                    let elem_reg = builder.next_reg();
                    let idx_reg = builder.next_reg();
                    builder.load_const(i as i64, idx_reg, left.span);
                    if let AssignType::Match = typ {
                        builder.match_catch(None, true, left.span);
                    }
                    builder.index(right, elem_reg, idx_reg, left.span);
                    self.do_assign(e, elem_reg, scope, builder, typ)?;
                }
                Ok(())
            },
            Expression::Dict(v) => {
                destructure_dict!(v);
                Ok(())
            },
            Expression::Instance { base, items } => {
                let base_reg = self.compile_expr(base, scope, builder)?;
                let type_reg = builder.next_reg();
                builder.type_of(right, type_reg, left.span);

                let eq_reg = builder.next_reg();

                builder.push_raw_opcode(
                    Opcode::Eq {
                        a: base_reg,
                        b: type_reg,
                        to: eq_reg,
                    },
                    left.span,
                );
                if let AssignType::Match = typ {
                    builder.match_catch(None, true, left.span);
                }
                builder.assert(eq_reg, left.span);

                destructure_dict!(items);
                Ok(())
            },
            Expression::Maybe(v) => todo!(),
            Expression::Index { .. } => {
                if let AssignType::Normal { is_let: true } = typ {
                    return Err(CompileError::IllegalExpressionInAssigment {
                        area: self.make_area(left.span),
                        is_let: true,
                    });
                }
                self.compile_mem_expr(left, scope, builder)?;
                builder.write_mem(right, left.span);
                Ok(())
            },
            Expression::Member { base, name } => {
                if let AssignType::Normal { is_let: true } = typ {
                    return Err(CompileError::IllegalExpressionInAssigment {
                        area: self.make_area(left.span),
                        is_let: true,
                    });
                }
                self.compile_mem_expr(left, scope, builder)?;
                builder.write_mem(right, left.span);
                Ok(())
            },
            Expression::Associated { base, name } => todo!(),
            _ => match typ {
                AssignType::Normal { .. } => Err(CompileError::IllegalExpressionInAssigment {
                    area: self.make_area(left.span),
                    is_let: false,
                }),
                AssignType::Match => {
                    let pat = self.compile_expr(left, scope, builder)?;
                    if let AssignType::Match = typ {
                        builder.match_catch(None, true, left.span);
                    }
                    builder.assert_matches(right, pat, left.span);
                    Ok(())
                },
            },
        }
    }

    pub fn compile_mem_expr(
        &mut self,
        expr: &ExprNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<()> {
        match &*expr.expr {
            Expression::Var(v) => match self.get_var(*v, scope) {
                Some(v) => {
                    builder.change_mem(v.reg, expr.span);
                },
                None => {
                    return Err(CompileError::NonexistentVariable {
                        area: self.make_area(expr.span),
                        var: self.resolve(v),
                    })
                },
            },
            Expression::Index { base, index } => {
                self.compile_mem_expr(base, scope, builder)?;
                let index = self.compile_expr(index, scope, builder)?;
                builder.index_set_mem(index, expr.span);
            },
            Expression::Member { base, name } => {
                self.compile_mem_expr(base, scope, builder)?;
                builder.member_set_mem(name.map(|v| self.resolve_arr(&v)), expr.span);
            },
            _ => {
                return Err(CompileError::IllegalExpressionInAssigment {
                    area: self.make_area(expr.span),
                    is_let: false,
                })
            },
        }
        Ok(())
    }

    pub fn compile_expr(
        &mut self,
        expr: &ExprNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<UnoptRegister> {
        match &*expr.expr {
            Expression::Int(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::Float(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::String(v) => {
                todo!()
                // let reg = builder.next_reg();
                // builder.load_const(*v, reg, expr.span);
                // Ok(reg)
            },
            Expression::Bool(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::Op(left, op, right) => {
                let left = self.compile_expr(left, scope, builder)?;
                let right = self.compile_expr(right, scope, builder)?;

                let dest = builder.next_reg();

                macro_rules! bin_op {
                    ($opcode_name:ident) => {
                        builder.push_raw_opcode(
                            Opcode::$opcode_name {
                                a: left,
                                b: right,
                                to: dest,
                            },
                            expr.span,
                        )
                    };
                }

                match op {
                    BinOp::Plus => bin_op!(Plus),
                    BinOp::Minus => bin_op!(Minus),
                    BinOp::Mult => bin_op!(Mult),
                    BinOp::Div => bin_op!(Div),
                    BinOp::Mod => bin_op!(Mod),
                    BinOp::Pow => bin_op!(Pow),
                    BinOp::Eq => bin_op!(Eq),
                    BinOp::Neq => bin_op!(Neq),
                    BinOp::Gt => bin_op!(Gt),
                    BinOp::Gte => bin_op!(Gte),
                    BinOp::Lt => bin_op!(Lt),
                    BinOp::Lte => bin_op!(Lte),
                    BinOp::BinOr => bin_op!(BinOr),
                    BinOp::BinAnd => bin_op!(BinAnd),
                    BinOp::Range => bin_op!(Range),
                    BinOp::In => bin_op!(In),
                    BinOp::ShiftLeft => bin_op!(ShiftLeft),
                    BinOp::ShiftRight => bin_op!(ShiftRight),
                    BinOp::As => bin_op!(As),
                    BinOp::Or => todo!(),
                    BinOp::And => todo!(),
                }
                Ok(dest)
            },
            Expression::Unary(op, value) => {
                let value = self.compile_expr(value, scope, builder)?;
                let dest = builder.next_reg();

                macro_rules! unary_op {
                    ($opcode_name:ident) => {
                        builder
                            .push_raw_opcode(Opcode::$opcode_name { v: value, to: dest }, expr.span)
                    };
                }

                match op {
                    UnaryOp::ExclMark => unary_op!(Not),
                    UnaryOp::Minus => unary_op!(Negate),
                }

                Ok(dest)
            },
            Expression::Var(v) => match self.get_var(*v, scope) {
                Some(v) => Ok(v.reg),
                None => Err(CompileError::NonexistentVariable {
                    area: self.make_area(expr.span),
                    var: self.resolve(v),
                }),
            },
            Expression::Type(v) => {
                let reg = builder.next_reg();
                let name = self.resolve(v);

                match ValueType::from_str(&name) {
                    Ok(v) => {
                        builder.load_const(v, reg, expr.span);
                    },
                    Err(_) => match self.available_custom_types.get(v) {
                        Some(k) => builder.load_const(ValueType::Custom(*k), reg, expr.span),
                        None => {
                            return Err(CompileError::NonexistentType {
                                area: self.make_area(expr.span),
                                type_name: name.into(),
                            })
                        },
                    },
                }
                Ok(reg)
            },
            Expression::Array(v) => {
                let dest = builder.next_reg();

                builder.alloc_array(dest, v.len() as u16, expr.span);
                for e in v {
                    let r = self.compile_expr(e, scope, builder)?;
                    builder.push_array_elem(r, dest, expr.span);
                }

                Ok(dest)
            },
            Expression::Dict(v) => {
                let dest = builder.next_reg();

                builder.alloc_dict(dest, v.len() as u16, expr.span);
                for item in v {
                    let r = match &item.value().value {
                        Some(e) => self.compile_expr(e, scope, builder)?,
                        None => match self.get_var(item.value().name.value, scope) {
                            Some(v) => v.reg,
                            None => {
                                return Err(CompileError::NonexistentVariable {
                                    area: self.make_area(expr.span),
                                    var: self.resolve(&item.value().name),
                                })
                            },
                        },
                    };
                    let k = builder.next_reg();

                    let chars = self.resolve_arr(&item.value().name.value);
                    builder.load_const::<ImmutVec<char>>(chars, k, item.value().name.span);

                    builder.insert_dict_elem(r, dest, k, expr.span, item.is_priv())
                }

                Ok(dest)
            },
            Expression::Maybe(_) => todo!(),
            Expression::Is(..) => todo!(),
            Expression::Index { base, index } => {
                let base = self.compile_expr(base, scope, builder)?;
                let index = self.compile_expr(index, scope, builder)?;
                let out = builder.next_reg();
                builder.index(base, out, index, expr.span);
                Ok(out)
            },
            Expression::Member { base, name } => {
                let base = self.compile_expr(base, scope, builder)?;
                let out = builder.next_reg();
                builder.member(base, out, name.map(|v| self.resolve_arr(&v)), expr.span);
                Ok(out)
            },
            Expression::TypeMember { base, name } => todo!(),
            Expression::Associated { base, name } => todo!(),
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
            Expression::TriggerFunc { attributes, code } => todo!(),
            Expression::TriggerFuncCall(_) => todo!(),
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => todo!(),
            Expression::Typeof(_) => todo!(),
            Expression::Builtins => todo!(),
            Expression::Empty => todo!(),
            Expression::Epsilon => todo!(),
            Expression::Import(import) => {
                let out = builder.next_reg();
                let (_, s, _) = self.compile_import(import, expr.span, Rc::clone(&self.src))?;
                builder.import(out, s, expr.span);

                Ok(out)
            },
            Expression::Instance { base, items } => todo!(),
            Expression::Match { value, branches } => {
                let value_reg = self.compile_expr(value, scope, builder)?;

                let out_reg = builder.next_reg();

                builder.new_block(|b| {
                    let outer = b.block;

                    for (pattern, branch) in branches {
                        b.new_block(|b| {
                            let derived = self.derive_scope(scope, None);
                            self.do_assign(pattern, value_reg, derived, b, AssignType::Match)?;

                            match branch {
                                MatchBranch::Expr(e) => {
                                    let e = self.compile_expr(e, derived, b)?;
                                    b.copy_deep(e, out_reg, expr.span);
                                    b.jump(Some(outer), JumpType::End, expr.span);
                                },
                                MatchBranch::Block(stmts) => {
                                    b.load_empty(out_reg, expr.span);
                                    for s in stmts {
                                        self.compile_stmt(s, derived, b)?;
                                    }
                                    b.jump(Some(outer), JumpType::End, expr.span);
                                },
                            }

                            Ok(())
                        })?;
                    }

                    Ok(())
                })?;

                Ok(out_reg)
            },
        }
    }

    pub fn compile_stmt(
        &mut self,
        stmt: &StmtNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<()> {
        match &*stmt.stmt {
            Statement::Expr(e) => {
                self.compile_expr(e, scope, builder)?;
            },
            Statement::Let(left, right) => {
                let right = self.compile_expr(right, scope, builder)?;
                self.do_assign(
                    left,
                    right,
                    scope,
                    builder,
                    AssignType::Normal { is_let: true },
                )?;
            },
            Statement::AssignOp(left, op, right) => {
                macro_rules! assign_op {
                    ($opcode_name:ident) => {{
                        let var = match &*left.expr {
                            Expression::Var(v) => *v,
                            _ => {
                                return Err(CompileError::IllegalExpressionForAugmentedAssignment {
                                    area: self.make_area(left.span),
                                })
                            },
                        };
                        let right_reg = self.compile_expr(right, scope, builder)?;
                        match self.get_var(var, scope) {
                            Some(data) if data.mutable => builder.push_raw_opcode(
                                Opcode::$opcode_name {
                                    a: data.reg,
                                    b: right_reg,
                                },
                                stmt.span,
                            ),
                            Some(data) => {
                                return Err(CompileError::ImmutableAssign {
                                    area: self.make_area(stmt.span),
                                    def_area: self.make_area(data.def_span),
                                    var: self.resolve(&var),
                                })
                            },
                            None => {
                                return Err(CompileError::NonexistentVariable {
                                    area: self.make_area(left.span),
                                    var: self.resolve(&var),
                                })
                            },
                        }
                    }};
                }

                match op {
                    AssignOp::Assign => {
                        let right = self.compile_expr(right, scope, builder)?;
                        self.do_assign(
                            left,
                            right,
                            scope,
                            builder,
                            AssignType::Normal { is_let: false },
                        )?;
                    },
                    AssignOp::PlusEq => assign_op!(PlusEq),
                    AssignOp::MinusEq => assign_op!(MinusEq),
                    AssignOp::MultEq => assign_op!(MultEq),
                    AssignOp::DivEq => assign_op!(DivEq),
                    AssignOp::PowEq => assign_op!(PowEq),
                    AssignOp::ModEq => assign_op!(ModEq),
                    AssignOp::BinAndEq => assign_op!(BinAndEq),
                    AssignOp::BinOrEq => assign_op!(BinOrEq),
                    AssignOp::ShiftLeftEq => assign_op!(ShiftLeftEq),
                    AssignOp::ShiftRightEq => assign_op!(ShiftRightEq),
                }
            },
            Statement::If {
                branches,
                else_branch,
            } => {
                builder.new_block(|b| {
                    let outer = b.block;

                    for (cond, code) in branches {
                        b.new_block(|b| {
                            let cond_reg = self.compile_expr(cond, scope, b)?;
                            b.jump(None, JumpType::EndIfFalse(cond_reg), cond.span);

                            let derived = self.derive_scope(scope, None);

                            for s in code {
                                self.compile_stmt(s, derived, b)?;
                            }
                            b.jump(Some(outer), JumpType::End, stmt.span);
                            Ok(())
                        })?;
                    }

                    if let Some(code) = else_branch {
                        let derived = self.derive_scope(scope, None);

                        for s in code {
                            self.compile_stmt(s, derived, b)?;
                        }
                    }

                    Ok(())
                })?;
            },
            Statement::While { cond, code } => {
                builder.new_block(|b| {
                    let cond_reg = self.compile_expr(cond, scope, b)?;
                    b.jump(None, JumpType::EndIfFalse(cond_reg), cond.span);

                    let derived = self.derive_scope(scope, Some(ScopeType::Loop(b.block)));

                    for s in code {
                        self.compile_stmt(s, derived, b)?;
                    }
                    b.jump(None, JumpType::Start, stmt.span);

                    Ok(())
                })?;
            },
            Statement::For {
                iter_var,
                iterator,
                code,
            } => {
                let iter_exec = self.compile_expr(iterator, scope, builder)?;
                let iter_reg = builder.next_reg();
                builder.push_raw_opcode(
                    Opcode::WrapIterator {
                        src: iter_exec,
                        dest: iter_reg,
                    },
                    stmt.span,
                );

                builder.new_block(|b| {
                    let next_reg = b.next_reg();
                    b.iter_next(iter_reg, next_reg, iter_var.span);

                    b.jump(None, JumpType::UnwrapOrEnd(next_reg), iterator.span);

                    let derived = self.derive_scope(scope, Some(ScopeType::Loop(b.block)));

                    match &*iter_var.expr {
                        Expression::Var(v) => {
                            let var_reg = b.next_reg();
                            self.scopes[derived].vars.insert(
                                *v,
                                VarData {
                                    mutable: false,
                                    def_span: iter_var.span,
                                    reg: var_reg,
                                },
                            );
                            b.copy_deep(next_reg, var_reg, iter_var.span);
                        },
                        _ => todo!("destruction !!!!!!!!!!!!!!!!!!!!!!!!"),
                    };

                    for s in code {
                        self.compile_stmt(s, derived, b)?;
                    }
                    b.jump(None, JumpType::Start, stmt.span);

                    Ok(())
                })?;
            },
            Statement::TryCatch {
                try_code,
                branches,
                catch_all,
            } => {
                /*





                */

                builder.new_block(|b| {
                    //df d
                    Ok(())
                })?;
            },
            Statement::Arrow(s) => {
                builder.new_block(|b| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::ArrowStmt(stmt.span))); // variables made in arrow statements shouldnt be allowed anyways
                    b.enter_arrow(stmt.span);
                    self.compile_stmt(s, inner_scope, b)?;
                    b.yeet_context(stmt.span);
                    Ok(())
                })?;
            },
            Statement::Return(v) => match self.scopes[scope].typ {
                Some(ScopeType::Global) => match v {
                    Some(e) => match &*e.expr {
                        Expression::Dict(items) => {
                            if let Some(gr) = &self.global_return {
                                return Err(CompileError::DuplicateModuleReturn {
                                    area: self.make_area(stmt.span),
                                    prev_area: self.make_area(gr.span),
                                });
                            }

                            let ret_reg = self.compile_expr(e, scope, builder)?;
                            self.global_return = Some(
                                items
                                    .iter()
                                    .map(|i| i.value().name)
                                    .collect_vec()
                                    .into_boxed_slice()
                                    .spanned(stmt.span),
                            );
                            builder.ret(ret_reg, true, stmt.span);
                        },
                        _ => {
                            return Err(CompileError::InvalidModuleReturn {
                                area: self.make_area(stmt.span),
                            })
                        },
                    },
                    None => {
                        return Err(CompileError::InvalidModuleReturn {
                            area: self.make_area(stmt.span),
                        })
                    },
                },
                _ => {
                    if self.is_inside_macro(scope) {
                        self.assert_can_return(scope, stmt.span)?;
                        match v {
                            None => {
                                let out_reg = builder.next_reg();
                                builder.load_empty(out_reg, stmt.span);
                                builder.ret(out_reg, false, stmt.span)
                            },
                            Some(expr) => {
                                let ret_reg = self.compile_expr(expr, scope, builder)?;
                                builder.ret(ret_reg, false, stmt.span)
                            },
                        }
                    } else {
                        return Err(CompileError::ReturnOutsideMacro {
                            area: self.make_area(stmt.span),
                        });
                    }
                },
            },
            Statement::Break => match self.is_inside_loop(scope) {
                Some(path) => {
                    self.assert_can_break_loop(scope, stmt.span)?;
                    builder.jump(Some(path), JumpType::End, stmt.span)
                },
                _ => {
                    return Err(CompileError::BreakOutsideLoop {
                        area: self.make_area(stmt.span),
                    })
                },
            },
            Statement::Continue => match self.is_inside_loop(scope) {
                Some(path) => {
                    self.assert_can_break_loop(scope, stmt.span)?;
                    builder.jump(Some(path), JumpType::Start, stmt.span)
                },
                _ => {
                    return Err(CompileError::ContinueOutsideLoop {
                        area: self.make_area(stmt.span),
                    })
                },
            },
            Statement::TypeDef(name) => {
                if !matches!(self.scopes[scope].typ, Some(ScopeType::Global)) {
                    return Err(CompileError::TypeDefNotGlobal {
                        area: self.make_area(stmt.span),
                    });
                }

                if ValueType::VARIANT_NAMES.contains(&&*self.resolve(name.value())) {
                    return Err(CompileError::BuiltinTypeOverride {
                        area: self.make_area(stmt.span),
                    });
                } else if let Some((_, def)) = self
                    .custom_type_defs
                    .iter()
                    .find(|(_, v)| v.value().name == *name.value())
                {
                    return Err(CompileError::DuplicateTypeDef {
                        area: self.make_area(stmt.span),
                        prev_area: self.make_area(def.value().def_span),
                    });
                } else if self.available_custom_types.contains_key(name.value()) {
                    return Err(CompileError::DuplicateImportedType {
                        area: self.make_area(stmt.span),
                    });
                };

                let def = TypeDef {
                    def_span: stmt.span,
                    name: *name.value(),
                };

                let id = self.custom_type_defs.insert(name.map(|_| def));
                self.available_custom_types.insert(
                    *name.value(),
                    CustomTypeID {
                        local: id,
                        source_hash: self.src_hash(),
                    },
                );
            },
            Statement::ExtractImport(import) => {
                let import_reg = builder.next_reg();
                let (names, s, types) =
                    self.compile_import(import, stmt.span, Rc::clone(&self.src))?;
                builder.import(import_reg, s, stmt.span);

                for name in &*names {
                    let var_reg = builder.next_reg();
                    let spur = self.intern(name);

                    self.scopes[scope].vars.insert(
                        spur,
                        VarData {
                            mutable: false,
                            def_span: stmt.span,
                            reg: var_reg,
                        },
                    );

                    builder.member(
                        import_reg,
                        var_reg,
                        self.resolve_arr(&spur).spanned(stmt.span),
                        stmt.span,
                    )
                }

                for (id, name) in types.iter() {
                    self.available_custom_types.insert(*name, *id);
                }
            },
            Statement::Impl { base, items } => todo!(),
            Statement::Overload { op, macros } => todo!(),
            Statement::Dbg(v) => {
                let v = self.compile_expr(v, scope, builder)?;
                builder.dbg(v, stmt.span);
            },
            Statement::Throw(v) => {
                let v = self.compile_expr(v, scope, builder)?;
                builder.throw(v, stmt.span);
            },
        }
        Ok(())
    }

    pub fn compile(&mut self, ast: &Ast, span: CodeSpan) -> CompileResult<()> {
        let mut code = ProtoBytecode::new();
        code.new_func(
            |b| {
                let base_scope = self.scopes.insert(Scope {
                    vars: Default::default(),
                    parent: None,
                    typ: Some(ScopeType::Global),
                });
                for stmt in &ast.statements {
                    self.compile_stmt(stmt, base_scope, b)?;
                }
                Ok(())
            },
            span,
        )?;
        let code = code.build(&self.src, self).unwrap();

        self.bytecode_map.insert((*self.src).clone(), code);

        Ok(())
    }
}

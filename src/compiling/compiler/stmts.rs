use std::rc::Rc;

use delve::VariantNames;
use itertools::Itertools;

use super::{CompileResult, Compiler, CustomTypeID, ScopeID, ScopeType, TypeDef, VarData};
use crate::compiling::builder::{CodeBuilder, JumpType};
use crate::compiling::error::CompileError;
use crate::compiling::opcodes::Opcode;
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{Expression, Statement, StmtNode, VisTrait};
use crate::parsing::operators::operators::AssignOp;
use crate::sources::Spannable;

impl Compiler<'_> {
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
            Statement::Assign(left, right) => {
                let right_reg = self.compile_expr(right, scope, builder)?;
                let match_reg = self.compile_pattern_check(right_reg, left, scope, builder)?;
                builder.mismatch_throw_if_false(match_reg, left.span);
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
                iter,
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
                    b.iter_next(iter_reg, next_reg, iter.span);

                    b.jump(None, JumpType::UnwrapOrEnd(next_reg), iterator.span);

                    let derived = self.derive_scope(scope, Some(ScopeType::Loop(b.block)));

                    let match_reg = self.compile_pattern_check(next_reg, iter, derived, b)?;
                    b.mismatch_throw_if_false(match_reg, iter.span);

                    for s in code {
                        self.compile_stmt(s, derived, b)?;
                    }
                    b.jump(None, JumpType::Start, stmt.span);

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
            Statement::TryCatch {
                try_code,
                catch_pat,
                catch_code,
            } => {
                let err_reg = builder.next_reg();

                builder.new_block(|builder| {
                    let outer = builder.block;

                    builder.new_block(|builder| {
                        builder.jump(None, JumpType::PushTryCatchEnd(err_reg), stmt.span);
                        let derived = self.derive_scope(scope, None);

                        for s in try_code {
                            self.compile_stmt(s, derived, builder)?;
                        }

                        builder.jump(Some(outer), JumpType::End, stmt.span);

                        Ok(())
                    })?;
                    let derived = self.derive_scope(scope, None);

                    if let Some(catch_pat) = catch_pat {
                        let matches_reg =
                            self.compile_pattern_check(err_reg, catch_pat, derived, builder)?;
                        builder.mismatch_throw_if_false(matches_reg, catch_pat.span);
                    }

                    Ok(())
                })?;
            },
        }
        Ok(())
    }
}

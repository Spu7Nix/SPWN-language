use std::rc::Rc;

use delve::VariantNames;
use itertools::Itertools;
use lasso::Spur;

use super::{CompileResult, Compiler, CustomTypeID, ScopeID, ScopeType, TypeDef, VarData};
use crate::compiling::builder::{CodeBuilder, JumpType};
use crate::compiling::bytecode::UnoptRegister;
use crate::compiling::error::CompileError;
use crate::compiling::opcodes::{FuncID, Opcode};
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{Expression, Pattern, Statement, Statements, StmtNode, Vis, VisTrait};
use crate::parsing::operators::operators::{AssignOp, Operator};
use crate::sources::{CodeSpan, Spannable, SpwnSource};
use crate::util::{ImmutStr, ImmutVec};

impl<'a> Compiler<'a> {
    pub fn compile_stmt(
        &mut self,
        stmt: &StmtNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<()> {
        self.deferred_trigger_func_stack.push(vec![]);
        match &*stmt.stmt {
            Statement::Expr(e) => {
                self.compile_expr(e, scope, builder)?;
            },
            Statement::Assign(left, right) => {
                // match (&*left.pat, &*right.expr) {
                //     (Pattern::)
                // }

                // let right_reg = self.compile_expr(right, scope, builder)?;
                // let match_reg =
                //     self.compile_pattern_check(right_reg, left, false, scope, builder)?;

                let temp_reg = builder.next_reg();
                let marker = builder.mark_insert();

                let match_reg =
                    self.compile_pattern_check(temp_reg, left, false, scope, builder)?;

                let right_reg = self.compile_expr(right, scope, &mut marker.with(builder))?;
                marker
                    .with(builder)
                    .copy_ref(right_reg, temp_reg, right.span);

                builder.mismatch_throw_if_false(match_reg, right_reg, left.span);
            },
            Statement::AssignOp(left, op, right) => {
                macro_rules! assign_op {
                    ($opcode_name:ident) => {{
                        let (var, _) = match &*left.pat {
                            Pattern::Path {
                                var,
                                path,
                                is_ref: false,
                            } => self.get_path_reg(*var, false, path, scope, builder, stmt.span)?,
                            _ => {
                                return Err(CompileError::IllegalAugmentedAssign {
                                    area: self.make_area(left.span),
                                })
                            },
                        };
                        let right_reg = self.compile_expr(right, scope, builder)?;

                        builder.push_raw_opcode(
                            Opcode::$opcode_name {
                                a: var,
                                b: right_reg,
                            },
                            stmt.span,
                        )
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
                            let derived = self.derive_scope(scope, None);

                            let cond_reg = self.compile_expr(cond, derived, b)?;
                            b.jump(None, JumpType::EndIfFalse(cond_reg), cond.span);

                            self.compile_stmts(code, derived, b)?;
                            b.jump(Some(outer), JumpType::End, stmt.span);
                            Ok(())
                        })?;
                    }

                    if let Some(code) = else_branch {
                        let derived = self.derive_scope(scope, None);

                        self.compile_stmts(code, derived, b)?;
                    }

                    Ok(())
                })?;
            },
            Statement::While { cond, code } => {
                let derived = self.derive_scope(scope, Some(ScopeType::Loop(builder.block)));

                builder.new_block(|builder| {
                    let cond_reg = self.compile_expr(cond, derived, builder)?;
                    builder.jump(None, JumpType::EndIfFalse(cond_reg), cond.span);

                    self.compile_stmts(code, derived, builder)?;
                    builder.jump(None, JumpType::Start, stmt.span);

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
                    Opcode::IntoIterator {
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

                    let match_reg = self.compile_pattern_check(next_reg, iter, true, derived, b)?;
                    b.mismatch_throw_if_false(match_reg, next_reg, iter.span);

                    self.compile_stmts(code, derived, b)?;
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
                    if let Some(ret_pat) = self.is_inside_macro(scope) {
                        self.assert_can_return(scope, stmt.span)?;
                        match v {
                            None => {
                                let out_reg = builder.next_reg();
                                builder.load_empty(out_reg, stmt.span);
                                self.compile_return(
                                    out_reg,
                                    ret_pat.as_deref(),
                                    false,
                                    scope,
                                    stmt.span,
                                    builder,
                                )?;
                            },
                            Some(expr) => {
                                let ret_reg = self.compile_expr(expr, scope, builder)?;
                                self.compile_return(
                                    ret_reg,
                                    ret_pat.as_deref(),
                                    false,
                                    scope,
                                    stmt.span,
                                    builder,
                                )?;
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
                    .local_type_defs
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
                    src: Rc::clone(&self.src),
                    def_span: stmt.span,
                    name: *name.value(),
                };

                let id = self.local_type_defs.insert(name.map(|_| def));

                let custom_id = CustomTypeID {
                    local: id,
                    source_hash: self.src_hash(),
                };
                let name_str = self.resolve_32(name.value());
                self.type_def_map.insert(
                    custom_id,
                    TypeDef {
                        src: Rc::clone(&self.src),
                        def_span: stmt.span,
                        name: name_str,
                    },
                );
                self.available_custom_types
                    .insert(*name.value(), name.map(|_| custom_id));
            },
            Statement::ExtractImport(import) => {
                let import_reg = builder.next_reg();
                let (names, s, types) = self.compile_import(
                    import,
                    stmt.span,
                    Rc::clone(&self.src),
                    self.src.get_variant(),
                )?;
                builder.import(import_reg, s, stmt.span);

                self.extract_import(names, types, scope, import_reg, builder, stmt.span);
            },
            Statement::Impl { name, items } => {
                let mut new_items = items.clone();

                for item in items {
                    if let Some(alias) = self.find_alias_attr(&item.value().attributes)? {
                        let mut aliased = item.clone();
                        aliased.value_mut().name = self.intern(&alias.name).spanned(alias.span);
                        new_items.push(aliased);
                    }
                }

                let dict_reg = self.compile_dictlike(&new_items, scope, stmt.span, builder)?;
                let typ_reg = builder.next_reg();

                self.load_type(&name.value, typ_reg, name.span, builder)?;

                builder.push_raw_opcode(
                    Opcode::Impl {
                        base: typ_reg,
                        dict: dict_reg,
                    },
                    stmt.span,
                );
            },
            Statement::Overload { op, macros } => {
                for v in macros {
                    if matches!(&*v.value().expr, Expression::Macro { .. }) {
                        let macro_reg = self.compile_expr(v.value(), scope, builder)?;

                        builder.push_raw_opcode(
                            Opcode::AddOperatorOverload {
                                from: macro_reg,
                                op: *op,
                            },
                            stmt.span,
                        );
                    } else {
                        return Err(CompileError::UnexpectedItemInOverload {
                            area: self.make_area(v.value().span),
                        });
                    }
                }
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

                        self.compile_stmts(try_code, derived, builder)?;

                        builder.jump(Some(outer), JumpType::End, stmt.span);

                        Ok(())
                    })?;
                    let derived = self.derive_scope(scope, None);

                    if let Some(catch_pat) = catch_pat {
                        let matches_reg =
                            self.compile_pattern_check(err_reg, catch_pat, true, derived, builder)?;
                        builder.mismatch_throw_if_false(matches_reg, err_reg, catch_pat.span);
                    }

                    self.compile_stmts(catch_code, derived, builder)?;

                    Ok(())
                })?;
            },
        }

        for trigger_fn in self.deferred_trigger_func_stack.pop().unwrap() {
            builder.push_raw_opcode(
                Opcode::SetContextGroup {
                    reg: trigger_fn.group_reg,
                },
                trigger_fn.span,
            );
            builder.new_block(|builder| {
                let inner_scope =
                    self.derive_scope(scope, Some(ScopeType::TriggerFunc(trigger_fn.span)));
                self.compile_stmts(&trigger_fn.stmts, inner_scope, builder)?;
                Ok(())
            })?;
            builder.push_raw_opcode(
                Opcode::SetContextGroup {
                    reg: trigger_fn.fn_reg,
                },
                trigger_fn.span,
            );
        }
        Ok(())
    }

    pub fn compile_stmts(
        &mut self,
        stmts: &Statements,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<()> {
        for s in stmts {
            self.compile_stmt(s, scope, builder)?;
        }
        Ok(())
    }
}

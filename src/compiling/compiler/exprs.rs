use std::rc::Rc;
use std::str::FromStr;

use base64::Engine;
use itertools::{Either, Itertools};

use super::{CompileResult, Compiler, ScopeID};
use crate::compiling::builder::{CodeBuilder, JumpType};
use crate::compiling::bytecode::{CallExpr, Constant, Register, UnoptRegister};
use crate::compiling::compiler::{Scope, ScopeType, VarData};
use crate::compiling::error::CompileError;
use crate::compiling::opcodes::{Opcode, RuntimeStringFlag};
use crate::gd::ids::IDClass;
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{
    ExprNode, Expression, MacroArg, MacroCode, MatchBranch, MatchBranchCode, StringType, VisTrait,
};
use crate::parsing::operators::operators::{BinOp, UnaryOp};
use crate::sources::{CodeSpan, Spannable, SpwnSource, ZEROSPAN};
use crate::util::ImmutVec;

impl Compiler<'_> {
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
            Expression::String(content) => {
                let out_reg = builder.next_reg();

                match &content.s {
                    StringType::Normal(s) => {
                        let mut s = self.resolve(s).to_string();
                        if content.flags.unindent {
                            s = unindent::unindent(&s)
                        }
                        if content.flags.base64 {
                            s = base64::engine::general_purpose::URL_SAFE.encode(s)
                        }
                        builder.load_const(
                            s.chars().collect_vec().into_boxed_slice(),
                            out_reg,
                            expr.span,
                        )
                    },
                    StringType::FString(v) => {
                        builder.load_const(
                            "".chars().collect_vec().into_boxed_slice(),
                            out_reg,
                            expr.span,
                        );
                        for i in v {
                            let s_r = builder.next_reg();
                            match i {
                                Either::Left(s) => {
                                    let s = self.resolve(s);
                                    builder.load_const(
                                        s.chars().collect_vec().into_boxed_slice(),
                                        s_r,
                                        expr.span,
                                    );
                                },
                                Either::Right(e) => {
                                    let r = self.compile_expr(e, scope, builder)?;

                                    builder.push_raw_opcode(
                                        Opcode::ToString { from: r, dest: s_r },
                                        expr.span,
                                    )
                                },
                            }
                            builder
                                .push_raw_opcode(Opcode::PlusEq { a: out_reg, b: s_r }, expr.span)
                        }
                        if content.flags.unindent {
                            builder.push_raw_opcode(
                                Opcode::ApplyStringFlag {
                                    flag: RuntimeStringFlag::Unindent,
                                    reg: out_reg,
                                },
                                expr.span,
                            )
                        }
                        if content.flags.base64 {
                            builder.push_raw_opcode(
                                Opcode::ApplyStringFlag {
                                    flag: RuntimeStringFlag::Base64,
                                    reg: out_reg,
                                },
                                expr.span,
                            )
                        }
                    },
                }
                if content.flags.bytes {
                    builder.push_raw_opcode(
                        Opcode::ApplyStringFlag {
                            flag: RuntimeStringFlag::ByteString,
                            reg: out_reg,
                        },
                        expr.span,
                    )
                }

                Ok(out_reg)
            },
            Expression::Bool(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
                Ok(reg)
            },
            Expression::Id(class, Some(id)) => {
                let reg = builder.next_reg();
                builder.load_const(Constant::Id(*class, *id), reg, expr.span);
                Ok(reg)
            },
            Expression::Id(class, None) => {
                let reg = builder.next_reg();

                builder.push_raw_opcode(
                    Opcode::LoadArbitraryID {
                        class: *class,
                        dest: reg,
                    },
                    expr.span,
                );
                Ok(reg)
            },
            Expression::Op(left, op, right) => {
                let dest = builder.next_reg();

                macro_rules! bin_op {
                    ($opcode_name:ident) => {{
                        let left = self.compile_expr(left, scope, builder)?;
                        let right = self.compile_expr(right, scope, builder)?;

                        builder.push_raw_opcode(
                            Opcode::$opcode_name {
                                a: left,
                                b: right,
                                to: dest,
                            },
                            expr.span,
                        )
                    }};
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
                    BinOp::Or => self.and_op(
                        &[
                            &|compiler, builder| compiler.compile_expr(left, scope, builder),
                            &|compiler, builder| compiler.compile_expr(right, scope, builder),
                        ],
                        dest,
                        expr.span,
                        builder,
                    )?,
                    BinOp::And => self.or_op(
                        &[
                            &|compiler, builder| compiler.compile_expr(left, scope, builder),
                            &|compiler, builder| compiler.compile_expr(right, scope, builder),
                        ],
                        dest,
                        expr.span,
                        builder,
                    )?,
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
                self.load_type(v, reg, expr.span, builder)?;
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
                let dest = self.compile_dictlike(v, scope, expr.span, builder)?;

                Ok(dest)
            },
            Expression::Maybe(Some(e)) => {
                let dest = builder.next_reg();
                let from = self.compile_expr(e, scope, builder)?;
                builder.push_raw_opcode(Opcode::WrapMaybe { from, to: dest }, expr.span);
                Ok(dest)
            },
            Expression::Maybe(None) => {
                let dest = builder.next_reg();
                builder.push_raw_opcode(Opcode::LoadNone { to: dest }, expr.span);
                Ok(dest)
            },
            Expression::Is(e, p) => {
                let reg = self.compile_expr(e, scope, builder)?;
                let matches = self.compile_pattern_check(reg, p, true, scope, builder)?;
                Ok(matches)
            },
            Expression::Index { base, index } => {
                let base = self.compile_expr(base, scope, builder)?;
                let index = self.compile_expr(index, scope, builder)?;
                let out = builder.next_reg();
                builder.index_mem(base, out, index, expr.span);
                Ok(out)
            },
            Expression::Member { base, name } => {
                let base = self.compile_expr(base, scope, builder)?;
                let out = builder.next_reg();
                builder.member(base, out, name.map(|v| self.resolve_arr(&v)), expr.span);
                Ok(out)
            },
            Expression::TypeMember { base, name } => {
                let base = self.compile_expr(base, scope, builder)?;
                let out = builder.next_reg();
                builder.type_member(base, out, name.map(|v| self.resolve_arr(&v)), expr.span);
                Ok(out)
            },
            Expression::Associated { base, name } => {
                let base = self.compile_expr(base, scope, builder)?;
                let out = builder.next_reg();
                builder.associated(base, out, name.map(|v| self.resolve_arr(&v)), expr.span);
                Ok(out)
            },
            Expression::Call {
                base,
                params,
                named_params,
            } => {
                let base_reg = self.compile_expr(base, scope, builder)?;

                let positional = params
                    .iter()
                    .map(|p| self.compile_expr(p, scope, builder))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice();

                let named = named_params
                    .iter()
                    .map(|(s, p)| {
                        self.compile_expr(p, scope, builder)
                            .map(|v| (self.resolve(&s.value), v))
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice();
                let out_reg = builder.next_reg();

                let call_expr = CallExpr {
                    positional,
                    named,
                    dest: Some(out_reg),
                };

                builder.call(base_reg, call_expr, expr.span);

                Ok(out_reg)
            },
            Expression::Macro {
                args,
                ret_pat,
                code,
            } => {
                let mut spread_arg = None;

                let store_args = args
                    .iter()
                    .enumerate()
                    .map(|(i, a)| match a {
                        MacroArg::Single { pattern, default } => {
                            let mut s = pattern.span;
                            if let Some(e) = default {
                                s = s.extend(e.span)
                            }
                            pattern.pat.get_name().map(|s| self.resolve(&s)).spanned(s)
                        },
                        MacroArg::Spread { pattern } => {
                            spread_arg = Some(i as u8);
                            pattern
                                .pat
                                .get_name()
                                .map(|s| self.resolve(&s))
                                .spanned(pattern.span)
                        },
                    })
                    .collect_vec();

                let captured = self
                    .get_accessible_vars(scope)
                    .enumerate()
                    .map(|(i, (_, v))| (v.reg, Register(i + args.len())))
                    .collect_vec();

                let is_builtin = self.find_builtin_attr(&expr.attributes);
                let is_debug = self.find_debug_bytecode_attr(&expr.attributes);

                let func_id = builder.new_func(
                    |builder| {
                        let base_scope = self.scopes.insert(Scope {
                            vars: Default::default(),
                            parent: None,
                            typ: Some(ScopeType::MacroBody(
                                ret_pat.as_ref().map(|p| Rc::new(p.clone())),
                            )),
                        });

                        for (i, (name, data)) in
                            self.get_accessible_vars(scope).enumerate().collect_vec()
                        {
                            self.scopes[base_scope].vars.insert(
                                name,
                                VarData {
                                    reg: Register(i + args.len()),
                                    ..data
                                },
                            );
                        }

                        for (i, g) in args.iter().enumerate() {
                            let arg_reg = Register(i);

                            let pat = g.pattern();

                            let matches_reg = self
                                .compile_pattern_check(arg_reg, pat, true, base_scope, builder)?;
                            builder.mismatch_throw_if_false(matches_reg, arg_reg, pat.span);
                        }

                        if is_builtin {
                            let ret_reg = builder.next_reg();
                            builder.push_raw_opcode(
                                Opcode::RunBuiltin {
                                    args: args.len() as u8,
                                    dest: ret_reg,
                                },
                                expr.span,
                            );
                            self.compile_return(
                                ret_reg,
                                ret_pat.as_ref(),
                                false,
                                base_scope,
                                expr.span,
                                builder,
                            )?;
                            return Ok(());
                        }

                        match code {
                            MacroCode::Normal(stmts) => {
                                for stmt in stmts {
                                    self.compile_stmt(stmt, base_scope, builder)?;
                                }
                                // let ret_reg = builder.next_reg();
                                // builder.load_empty(ret_reg, expr.span);
                                // self.compile_return(
                                //     ret_reg,
                                //     ret_pat.as_ref(),
                                //     false,
                                //     base_scope,
                                //     expr.span,
                                //     builder,
                                // )?;
                            },
                            MacroCode::Lambda(expr) => {
                                let ret_reg = self.compile_expr(expr, base_scope, builder)?;
                                self.compile_return(
                                    ret_reg,
                                    ret_pat.as_ref(),
                                    false,
                                    base_scope,
                                    expr.span,
                                    builder,
                                )?;
                            },
                        }

                        Ok(())
                    },
                    (store_args.into(), spread_arg),
                    captured,
                    expr.span,
                )?;

                if is_debug {
                    builder.mark_func_debug(func_id)
                }

                let macro_reg = builder.next_reg();
                builder.push_raw_opcode(
                    Opcode::CreateMacro {
                        func: func_id,
                        dest: macro_reg,
                    },
                    expr.span,
                );

                for (i, arg) in args.iter().enumerate() {
                    if let MacroArg::Single {
                        default: Some(d), ..
                    } = arg
                    {
                        let r = self.compile_expr(d, scope, builder)?;

                        builder.push_raw_opcode(
                            Opcode::PushMacroDefault {
                                to: macro_reg,
                                from: r,
                                arg: i as u8,
                            },
                            expr.span,
                        );
                    }
                }

                if args
                    .get(0)
                    .is_some_and(|v| v.pattern().pat.is_self(&self.interner))
                {
                    builder.push_raw_opcode(Opcode::MarkMacroMethod { reg: macro_reg }, expr.span);
                }

                Ok(macro_reg)
            },
            Expression::TriggerFunc { code } => {
                let group_reg = builder.next_reg();
                let out_reg = builder.next_reg();

                builder.push_raw_opcode(
                    Opcode::LoadArbitraryID {
                        class: IDClass::Group,
                        dest: group_reg,
                    },
                    expr.span,
                );
                builder.push_raw_opcode(
                    Opcode::MakeTriggerFunc {
                        src: group_reg,
                        dest: out_reg,
                    },
                    expr.span,
                );

                builder.push_raw_opcode(Opcode::SetContextGroup { reg: group_reg }, expr.span);
                builder.new_block(|builder| {
                    let inner_scope =
                        self.derive_scope(scope, Some(ScopeType::TriggerFunc(expr.span)));
                    for s in code {
                        self.compile_stmt(s, inner_scope, builder)?
                    }
                    Ok(())
                })?;
                builder.push_raw_opcode(Opcode::SetContextGroup { reg: out_reg }, expr.span);
                Ok(out_reg)
            },
            Expression::TriggerFuncCall(e) => {
                let reg = self.compile_expr(e, scope, builder)?;
                builder.push_raw_opcode(Opcode::CallTriggerFunc { func: reg }, expr.span);

                Ok(builder.next_reg())
            },
            Expression::Ternary {
                cond,
                if_true,
                if_false,
            } => {
                let dest = builder.next_reg();

                let cond_reg = self.compile_expr(cond, scope, builder)?;
                builder.new_block(|builder| {
                    let outer = builder.block;

                    builder.new_block(|builder| {
                        builder.jump(None, JumpType::EndIfFalse(cond_reg), ZEROSPAN);
                        let r = self.compile_expr(if_true, scope, builder)?;
                        builder.copy_deep(r, dest, if_true.span);
                        builder.jump(Some(outer), JumpType::End, ZEROSPAN);
                        Ok(())
                    })?;
                    let r = self.compile_expr(if_false, scope, builder)?;
                    builder.copy_deep(r, dest, if_false.span);
                    Ok(())
                })?;
                Ok(dest)
            },
            Expression::Typeof(e) => {
                let dest = builder.next_reg();
                let r = self.compile_expr(e, scope, builder)?;
                builder.type_of(r, dest, expr.span);
                Ok(dest)
            },
            Expression::Builtins => {
                let dest = builder.next_reg();
                builder.push_raw_opcode(Opcode::LoadBuiltins { to: dest }, expr.span);
                Ok(dest)
            },
            Expression::Empty => {
                let dest = builder.next_reg();
                builder.push_raw_opcode(Opcode::LoadEmpty { to: dest }, expr.span);
                Ok(dest)
            },
            Expression::Epsilon => {
                let dest = builder.next_reg();
                builder.push_raw_opcode(Opcode::LoadEpsilon { to: dest }, expr.span);
                Ok(dest)
            },
            Expression::Import(import) => {
                let out = builder.next_reg();
                let (_, s, _) = self.compile_import(import, expr.span, Rc::clone(&self.src))?;
                builder.import(out, s, expr.span);

                Ok(out)
            },
            Expression::Instance { base, items } => {
                let dest = builder.next_reg();

                let base = self.compile_expr(base, scope, builder)?;

                let items_reg = self.compile_dictlike(items, scope, expr.span, builder)?;

                builder.push_raw_opcode(
                    Opcode::MakeInstance {
                        base,
                        items: items_reg,
                        dest,
                    },
                    expr.span,
                );

                Ok(dest)
            },
            Expression::Match { value, branches } => {
                let value_reg = self.compile_expr(value, scope, builder)?;

                let out_reg = builder.next_reg();
                builder.load_empty(out_reg, expr.span);

                builder.new_block(|builder| {
                    let outer = builder.block;

                    for branch in branches {
                        builder.new_block(|builder| {
                            let derived = self.derive_scope(scope, None);

                            let matches_reg = self.compile_pattern_check(
                                value_reg,
                                &branch.pattern,
                                true,
                                derived,
                                builder,
                            )?;
                            builder.jump(
                                None,
                                JumpType::EndIfFalse(matches_reg),
                                branch.pattern.span,
                            );

                            match &branch.code {
                                MatchBranchCode::Expr(e) => {
                                    let e = self.compile_expr(e, derived, builder)?;
                                    builder.copy_deep(e, out_reg, expr.span);
                                    builder.jump(Some(outer), JumpType::End, expr.span);
                                },
                                MatchBranchCode::Block(stmts) => {
                                    builder.load_empty(out_reg, expr.span);
                                    for s in stmts {
                                        self.compile_stmt(s, derived, builder)?;
                                    }
                                    builder.jump(Some(outer), JumpType::End, expr.span);
                                },
                            }

                            Ok(())
                        })?;
                    }

                    Ok(())
                })?;

                Ok(out_reg)
            },
            Expression::Dbg(v, show_ptr) => {
                let out = builder.next_reg();
                let v = self.compile_expr(v, scope, builder)?;
                builder.dbg(v, *show_ptr, expr.span);
                builder.load_empty(out, expr.span);
                Ok(out)
            },
            Expression::Obj(typ, items) => {
                todo!();
                // builder.new_object(
                //     items.len() as u16,
                //     out_reg,
                //     |builder, elems| {
                //         for (key, expr) in items {
                //             let value_reg =
                //                 self.compile_expr(expr, scope, builder, ExprType::normal())?;

                //             elems.push((*key, value_reg));
                //         }

                //         Ok(())
                //     },
                //     expr.span,
                //     *typ,
                // )?;
            },
        }
    }
}

use std::rc::Rc;
use std::str::FromStr;

use base64::Engine;
use itertools::Itertools;

use super::{CompileResult, Compiler, ScopeID};
use crate::compiling::builder::{CodeBuilder, JumpType};
use crate::compiling::bytecode::UnoptRegister;
use crate::compiling::error::CompileError;
use crate::compiling::opcodes::Opcode;
use crate::interpreting::value::ValueType;
use crate::parsing::ast::{ExprNode, Expression, StringType, VisTrait};
use crate::parsing::operators::operators::{BinOp, UnaryOp};
use crate::sources::{CodeSpan, ZEROSPAN};
use crate::util::{Either, ImmutVec};

impl Compiler<'_> {
    pub fn compile_expr(
        &mut self,
        expr: &ExprNode,
        scope: ScopeID,
        builder: &mut CodeBuilder,
    ) -> CompileResult<UnoptRegister> {
        macro_rules! dict_compile {
            ($dest:expr, $items:expr) => {
                builder.alloc_dict($dest, $items.len() as u16, expr.span);
                for item in $items {
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

                    builder.insert_dict_elem(r, $dest, k, expr.span, item.is_priv())
                }
            };
        }

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
                let out_reg = builder.next_reg();

                match &v.s {
                    StringType::Normal(s) => {
                        let mut s = self.resolve(s).to_string();
                        if v.unindent {
                            s = unindent::unindent(&s)
                        }
                        if v.base64 {
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
                    },
                }
                if v.bytes {
                    builder.push_raw_opcode(Opcode::MakeByteString { reg: out_reg }, expr.span)
                }

                Ok(out_reg)
            },
            Expression::Bool(v) => {
                let reg = builder.next_reg();
                builder.load_const(*v, reg, expr.span);
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
                let dest = builder.next_reg();

                dict_compile!(dest, v);

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
                let matches = self.compile_pattern_check(reg, p, scope, builder)?;
                Ok(matches)
            },
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
            } => todo!(),
            Expression::Macro {
                args,
                ret_type,
                code,
            } => todo!(),
            Expression::TriggerFunc { code } => todo!(),
            Expression::TriggerFuncCall(_) => todo!(),
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
                        builder.copy(r, dest, if_true.span);
                        builder.jump(Some(outer), JumpType::End, ZEROSPAN);
                        Ok(())
                    })?;
                    let r = self.compile_expr(if_false, scope, builder)?;
                    builder.copy(r, dest, if_false.span);
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
            Expression::Instance { base, items } => todo!(),
            Expression::Match { value, branches } => {
                todo!()
                // let value_reg = self.compile_expr(value, scope, builder)?;

                // let out_reg = builder.next_reg();

                // builder.new_block(|b| {
                //     let outer = b.block;

                //     for (pattern, branch) in branches {
                //         b.new_block(|b| {
                //             let derived = self.derive_scope(scope, None);
                //             self.do_assign(pattern, value_reg, derived, b, AssignType::Match)?;

                //             match branch {
                //                 MatchBranch::Expr(e) => {
                //                     let e = self.compile_expr(e, derived, b)?;
                //                     b.copy_deep(e, out_reg, expr.span);
                //                     b.jump(Some(outer), JumpType::End, expr.span);
                //                 },
                //                 MatchBranch::Block(stmts) => {
                //                     b.load_empty(out_reg, expr.span);
                //                     for s in stmts {
                //                         self.compile_stmt(s, derived, b)?;
                //                     }
                //                     b.jump(Some(outer), JumpType::End, expr.span);
                //                 },
                //             }

                //             Ok(())
                //         })?;
                //     }

                //     Ok(())
                // })?;

                // Ok(out_reg)
            },
            Expression::Id(class, Some(id)) => Constant::Id(*class, *id),
        }
    }
}

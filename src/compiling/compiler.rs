use crate::{
    parsing::{
        ast::{ExprNode, Expression, Statement, StmtNode},
        parser::Interner,
        utils::operators::{BinOp, UnaryOp},
    },
    vm::opcodes::Register,
};

use super::bytecode::{Bytecode, BytecodeBuilder, FuncBuilder};

pub struct Compiler {
    interner: Interner,
}

impl Compiler {
    pub fn new(interner: Interner) -> Self {
        Self { interner }
    }

    pub fn compile(&self, stmts: Vec<StmtNode>) -> Bytecode {
        let mut builder = BytecodeBuilder::new();

        // func 0 ("global")
        builder.new_func(|f| self.compile_stmts(stmts, f));

        builder.build()
    }

    pub fn compile_stmts(&self, stmts: Vec<StmtNode>, builder: &mut FuncBuilder) {
        for stmt in stmts {
            self.compile_stmt(stmt, builder);
        }
    }

    pub fn compile_stmt(&self, stmt: StmtNode, builder: &mut FuncBuilder) {
        match *stmt.stmt {
            Statement::Expr(e) => {
                self.compile_expr(e, builder);
            }
            Statement::Let(var, expr) => todo!(),

            _ => todo!(),
        }
    }

    pub fn compile_expr(&self, expr: ExprNode, builder: &mut FuncBuilder) -> Register {
        let out_reg = builder.next_reg();

        match *expr.expr {
            Expression::Int(v) => builder.load_int(v, out_reg),
            Expression::Float(v) => builder.load_float(v, out_reg),
            Expression::Bool(v) => builder.load_bool(v, out_reg),
            Expression::String(v) => {
                builder.load_string(self.interner.resolve(&v).to_string(), out_reg)
            }
            Expression::Op(left, op, right) => match op {
                BinOp::Plus => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.add(a, b, out_reg)
                }
                BinOp::Minus => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.sub(a, b, out_reg)
                }
                BinOp::Mult => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.mult(a, b, out_reg)
                }
                BinOp::Div => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.div(a, b, out_reg)
                }
                BinOp::Mod => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.modulo(a, b, out_reg)
                }
                BinOp::Pow => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.pow(a, b, out_reg)
                }
                BinOp::ShiftLeft => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.shl(a, b, out_reg)
                }
                BinOp::ShiftRight => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.shr(a, b, out_reg)
                }
                BinOp::BinAnd => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.bin_and(a, b, out_reg)
                }
                BinOp::BinOr => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.bin_or(a, b, out_reg)
                }

                BinOp::PlusEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.add_eq(a, b)
                }
                BinOp::MinusEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.sub_eq(a, b)
                }
                BinOp::MultEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.mult_eq(a, b)
                }
                BinOp::DivEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.div_eq(a, b)
                }
                BinOp::ModEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.modulo_eq(a, b)
                }
                BinOp::PowEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.pow_eq(a, b)
                }
                BinOp::ShiftLeftEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.shl_eq(a, b)
                }
                BinOp::ShiftRightEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.shr_eq(a, b)
                }
                BinOp::BinAndEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.bin_and_eq(a, b)
                }
                BinOp::BinOrEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.bin_or_eq(a, b)
                }
                BinOp::BinNotEq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.bin_not_eq(a, b)
                }

                BinOp::Eq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.eq(a, b, out_reg)
                }
                BinOp::Neq => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.neq(a, b, out_reg)
                }
                BinOp::Gt => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.gt(a, b, out_reg)
                }
                BinOp::Gte => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.gte(a, b, out_reg)
                }
                BinOp::Lt => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.lt(a, b, out_reg)
                }
                BinOp::Lte => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.lte(a, b, out_reg)
                }

                BinOp::DotDot => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.range(a, b, out_reg)
                }
                BinOp::In => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.in_op(a, b, out_reg)
                }
                BinOp::As => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.as_op(a, b, out_reg)
                }
                BinOp::Is => {
                    let a = self.compile_expr(left, builder);
                    let b = self.compile_expr(right, builder);
                    builder.is_op(a, b, out_reg)
                }

                BinOp::Assign => todo!(),
            },
            Expression::Unary(op, value) => {
                let v = self.compile_expr(value, builder);
                match op {
                    UnaryOp::BinNot => builder.unary_bin_not(v, out_reg),
                    UnaryOp::ExclMark => builder.unary_not(v, out_reg),
                    UnaryOp::Minus => builder.unary_negate(v, out_reg),
                }
            }
            Expression::Id(_, _) => todo!(),
            Expression::Var(_) => todo!(),
            Expression::Type(_) => todo!(),
            Expression::Array(items) => {
                builder.new_array(items.len(), out_reg, |builder, elems| {
                    for item in items {
                        elems.push(self.compile_expr(item, builder));
                    }
                })
            }
            Expression::Dict(_) => todo!(),
            Expression::Maybe(_) => todo!(),
            Expression::Index { base, index } => todo!(),
            Expression::Member { base, name } => todo!(),
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
            Expression::MacroPattern { args, ret_type } => todo!(),
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
            Expression::Import(_) => todo!(),
            Expression::Instance { base, items } => todo!(),
            // print(1 + 2)
            // _ => todo!(),
        }
        out_reg
    }
}

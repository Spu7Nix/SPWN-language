use std::rc::Rc;

use itertools::Itertools;

use super::error::RuntimeError;
use super::value::{StoredValue, Value, ValueType};
use super::vm::{Program, RuntimeResult, Vm};
use crate::parsing::ast::VisTrait;
use crate::parsing::operators::operators::{AssignOp, BinOp, UnaryOp};
use crate::sources::CodeSpan;
use crate::util::{Str32, String32};

pub fn to_bool(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<bool> {
    Ok(match &v.value {
        Value::Bool(b) => *b,
        _ => {
            return Err(RuntimeError::TypeMismatch {
                value_type: v.value.get_type(),
                value_area: v.area.clone(),
                expected: &[ValueType::Bool],
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

pub fn equality(a: &Value, b: &Value, vm: &Vm) -> bool {
    match (a, b) {
        (Value::Array(v1), Value::Array(v2)) => {
            if v1.len() != v2.len() {
                false
            } else {
                v1.iter()
                    .zip(v2)
                    .all(|(k1, k2)| equality(&k1.borrow().value, &k2.borrow().value, vm))
            }
        },
        (Value::Dict(v1), Value::Dict(v2)) => {
            if v1.len() != v2.len() {
                false
            } else {
                for (k, k1) in v1 {
                    match v2.get(k) {
                        Some(k2) => {
                            if !equality(&k1.value().borrow().value, &k1.value().borrow().value, vm)
                            {
                                return false;
                            }
                        },
                        None => return false,
                    }
                }
                true
            }
        },
        (Value::Maybe(Some(k1)), Value::Maybe(Some(k2))) => {
            equality(&k1.borrow().value, &k2.borrow().value, vm)
        },
        (Value::Dict { .. }, _) => todo!(),
        (Value::Maybe { .. }, _) => todo!(),
        (Value::Instance { .. }, _) => todo!(),
        (Value::Module { .. }, _) => todo!(),
        // todo: iterator, object
        _ => a == b,
    }
}

fn in_op(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (a, Value::Array(b)) => {
            todo!("context shit overloading boboggl") // context shit overloading boboggl
        },

        (Value::String(s), Value::Dict(d)) => Value::Bool(d.contains_key(s)),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                op: BinOp::In,
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn plus(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a + *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a + *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a + *b as f64),

        (Value::String(a), Value::String(b)) => {
            let v = String32::from_chars(
                a.as_char_slice()
                    .iter()
                    .chain(b.as_char_slice().iter())
                    .copied()
                    .collect_vec(),
            );
            Value::String(v.into())
        },
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Plus,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn minus(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a - *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a - *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 - *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a - *b as f64),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Minus,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn mult(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a * *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a * *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a * *b as f64),

        (Value::Int(n), Value::String(s)) | (Value::String(s), Value::Int(n)) => {
            Value::String(s.as_ref().repeat((*n).max(0) as usize).into())
        },

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Mult,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn div(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                return Err(RuntimeError::DivisionByZero {
                    area: vm.make_area(span, program),
                    call_stack: vm.get_call_stack(),
                });
            }
            Value::Int(*a / *b)
        },
        (Value::Float(a), Value::Float(b)) => Value::Float(*a / *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 / *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a / *b as f64),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Div,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn modulo(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a % *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a % *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 % *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a % *b as f64),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Mod,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn pow(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a.pow(*b as u32)),
        (Value::Float(a), Value::Float(b)) => Value::Float(a.powf(*b)),
        (Value::Int(a), Value::Float(b)) => Value::Float((*a as f64).powf(*b)),
        (Value::Float(a), Value::Int(b)) => Value::Float(a.powi(*b as i32)),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Pow,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn unary_not(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match &v.value {
        Value::Bool(b) => Value::Bool(!b),
        Value::Int(i) => Value::Int(!i),
        Value::Float(f) => Value::Float((!f.to_bits()) as f64),
        _ => {
            return Err(RuntimeError::InvalidUnaryOperand {
                v: (v.value.get_type(), v.area.clone()),
                op: UnaryOp::ExclMark,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn unary_negate(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match &v.value {
        Value::Int(n) => Value::Int(-n),
        Value::Float(n) => Value::Float(-n),
        _ => {
            return Err(RuntimeError::InvalidUnaryOperand {
                v: (v.value.get_type(), v.area.clone()),
                op: UnaryOp::Minus,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn gt(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a > b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a > b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((*a as f64) > *b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(*a > *b as f64),
        (Value::String(a), Value::String(b)) => Value::Bool(a > b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Gt,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn lt(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a < b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a < b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((*a as f64) < *b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(*a < *b as f64),
        (Value::String(a), Value::String(b)) => Value::Bool(a < b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Lt,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn gte(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a >= b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a >= b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((*a as f64) >= *b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(*a >= *b as f64),
        (Value::String(a), Value::String(b)) => Value::Bool(a >= b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Gte,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn lte(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a <= b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a <= b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((*a as f64) <= *b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(*a <= *b as f64),
        (Value::String(a), Value::String(b)) => Value::Bool(a <= b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Lte,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn range(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Range {
            start: *a,
            end: *b,
            step: 1,
        },
        (Value::Range { start, end, step }, Value::Int(b)) => {
            if *step == 1 {
                Value::Range {
                    start: *start,
                    end: *b,
                    step: *end as usize,
                }
            } else {
                todo!()
            }
        },
        // (Value::Int(a), Value::Float(b)) => Value::Range(*a, *b as i64, 1),
        // (Value::Float(a), Value::Int(b)) => Value::Range(*a as i64, *b, 1),
        // (Value::Float(a), Value::Float(b)) => Value::Range(*a as i64, *b as i64, 1),

        // should only allow integer ranges, rounding floats is kinda stinky
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Range,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

pub fn bin_and(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a & *b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a & *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinAnd,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

pub fn bin_or(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a | *b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a | *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn shift_left(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a << *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn shift_right(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a >> *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            })
        },
    })
}

fn eq(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(Value::Bool(equality(&a.value, &b.value, vm)))
}

fn neq(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(Value::Bool(!equality(&a.value, &b.value, vm)))
}

pub type BinOpFn =
    fn(&StoredValue, &StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>;

pub type UnaryOpFn = fn(&StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>;

impl BinOp {
    pub fn get_fn(self) -> BinOpFn {
        match self {
            BinOp::Range => range,
            BinOp::In => in_op,
            BinOp::BinOr => bin_or,
            BinOp::BinAnd => bin_and,
            BinOp::Eq => eq,
            BinOp::Neq => neq,
            BinOp::Gt => gt,
            BinOp::Gte => gte,
            BinOp::Lt => lt,
            BinOp::Lte => lte,
            BinOp::ShiftLeft => shift_left,
            BinOp::ShiftRight => shift_right,
            BinOp::Plus => plus,
            BinOp::Minus => minus,
            BinOp::Mult => mult,
            BinOp::Div => div,
            BinOp::Mod => modulo,
            BinOp::Pow => pow,
            _ => unreachable!(),
        }
    }
}

impl AssignOp {
    pub fn get_fn(self) -> BinOpFn {
        match self {
            AssignOp::PlusEq => plus,
            AssignOp::MinusEq => minus,
            AssignOp::MultEq => mult,
            AssignOp::DivEq => div,
            AssignOp::PowEq => pow,
            AssignOp::ModEq => modulo,
            AssignOp::BinAndEq => bin_and,
            AssignOp::BinOrEq => bin_or,
            AssignOp::ShiftLeftEq => shift_left,
            AssignOp::ShiftRightEq => shift_right,
        }
    }
}

impl UnaryOp {
    pub fn get_fn(self) -> UnaryOpFn {
        match self {
            UnaryOp::ExclMark => unary_not,
            UnaryOp::Minus => unary_negate,
        }
    }
}

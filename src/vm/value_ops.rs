use super::error::RuntimeError;
use super::interpreter::{RuntimeResult, Vm};
use super::value::{StoredValue, Value, ValueType};
use crate::parsing::utils::operators::{BinOp, UnaryOp};
use crate::sources::CodeSpan;

pub fn to_bool(v: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<bool> {
    Ok(match &v.value {
        Value::Bool(b) => *b,
        _ => {
            return Err(RuntimeError::TypeMismatch {
                v: (v.value.get_type(), v.area.clone()),
                expected: ValueType::Bool,
                area: vm.make_area(span),
            })
        }
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
                    .all(|(k1, k2)| equality(&vm.memory[*k1].value, &vm.memory[*k2].value, vm))
            }
        }
        (Value::Dict(v1), Value::Dict(v2)) => {
            if v1.len() != v2.len() {
                false
            } else {
                for (k, k1) in v1 {
                    match v2.get(k) {
                        Some(k2) => {
                            if !equality(&vm.memory[*k1].value, &vm.memory[*k2].value, vm) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
                // v1.iter()
                //     .zip(v2)
                //     .all(|(k1, k2)| equality(&vm.memory[*k1].value, &vm.memory[*k2].value, vm))
            }
        }
        (Value::Maybe(Some(k1)), Value::Maybe(Some(k2))) => {
            equality(&vm.memory[*k1].value, &vm.memory[*k2].value, vm)
        }
        _ => a == b,
    }
}

pub fn add(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a + *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a + *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a + *b as f64),

        (Value::String(a), Value::String(b)) => Value::String(a.clone() + b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Plus,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn sub(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn mult(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a * *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a * *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 * *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a * *b as f64),

        (Value::Int(n), Value::String(s)) | (Value::String(s), Value::Int(n)) => {
            Value::String(s.repeat((*n).max(0) as usize))
        }

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Mult,
                area: vm.make_area(span),
            })
        }
    })
}
pub fn div(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a / *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a / *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 / *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a / *b as f64),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Div,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn modulo(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn pow(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn not(v: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match &v.value {
        Value::Bool(b) => Value::Bool(!b),
        _ => {
            return Err(RuntimeError::InvalidUnaryOperand {
                v: (v.value.get_type(), v.area.clone()),
                op: UnaryOp::ExclMark,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn negate(v: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match &v.value {
        Value::Int(n) => Value::Int(-n),
        Value::Float(n) => Value::Float(-n),
        _ => {
            return Err(RuntimeError::InvalidUnaryOperand {
                v: (v.value.get_type(), v.area.clone()),
                op: UnaryOp::Minus,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn gt(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn lt(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn gte(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn lte(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
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
                area: vm.make_area(span),
            })
        }
    })
}

pub fn and(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && *b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::And,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn or(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || *b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Or,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn range(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Range(*a, *b, 1),
        (Value::Int(a), Value::Float(b)) => Value::Range(*a, *b as i64, 1),
        (Value::Float(a), Value::Int(b)) => Value::Range(*a as i64, *b, 1),
        (Value::Float(a), Value::Float(b)) => Value::Range(*a as i64, *b as i64, 1),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Range,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn bin_and(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a & *b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a & *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinAnd,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn bin_or(a: &StoredValue, b: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a | *b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a | *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn shift_left(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a << *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span),
            })
        }
    })
}

pub fn shift_right(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a >> *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span),
            })
        }
    })
}

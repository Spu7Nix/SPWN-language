use crate::{
    parsing::utils::operators::BinOp,
    sources::{CodeArea, CodeSpan},
};

use super::{
    error::RuntimeError,
    interpreter::{RuntimeResult, Vm},
    value::{StoredValue, Value},
};

// pub fn call_op(a: &StoredValue, b: &StoredValue, op: BinOp, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {

// }

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

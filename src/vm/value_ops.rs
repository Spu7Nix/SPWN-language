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

pub fn not(src: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match &src.value {
        Value::Bool(b) => Value::Bool(!b),
        _ => {
            unimplemented!()
            // return Err(RuntimeError::InvalidOperand {
            //     a: (src.value.get_type(), src.area.clone()),
            //     op: UnOp::Not,
            //     area: arg.make_area(span),
            // })
        }
    })
}

pub fn negate(src: &StoredValue, span: CodeSpan, vm: &Vm) -> RuntimeResult<Value> {
    Ok(match &src.value {
        Value::Int(n) => Value::Int(-n),
        Value::Float(n) => Value::Float(-n),
        _ => {
            unimplemented!()
            // return Err(RuntimeError::InvalidOperand {
            //     a: (src.value.get_type(), src.area.clone()),
            //     op: UnOp::Negate,
            //     area: arg.make_area(span),
            // })
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

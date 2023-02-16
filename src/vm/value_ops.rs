use super::error::RuntimeError;
use super::interpreter::{BytecodeKey, RuntimeResult, Vm};
use super::pattern::Pattern;
use super::value::{StoredValue, Value, ValueType};
use crate::gd::gd_object::ObjParam;
use crate::parsing::utils::operators::{BinOp, UnaryOp};
use crate::sources::CodeSpan;

pub fn to_bool(v: &StoredValue, span: CodeSpan, vm: &Vm, code: BytecodeKey) -> RuntimeResult<bool> {
    Ok(match &v.value {
        Value::Bool(b) => *b,
        _ => {
            return Err(RuntimeError::TypeMismatch {
                v: (v.value.get_type(), v.area.clone()),
                expected: ValueType::Bool,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn to_obj_param(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<ObjParam> {
    let param = 'm: {
        match &v.value {
            Value::Int(n) => Some(ObjParam::Number(*n as f64)),
            Value::Float(n) => Some(ObjParam::Number(*n)),

            Value::Group(id) => Some(ObjParam::Group(*id)),
            Value::Channel(id) => Some(ObjParam::Channel(*id)),
            Value::Block(id) => Some(ObjParam::Block(*id)),
            Value::Item(id) => Some(ObjParam::Item(*id)),

            Value::Bool(b) => Some(ObjParam::Bool(*b)),
            Value::String(s) => Some(ObjParam::Text(s.iter().collect())),

            Value::Epsilon => Some(ObjParam::Epsilon),

            Value::TriggerFunction { group, .. } => Some(ObjParam::Group(*group)),

            Value::Array(v) => {
                let mut arr = vec![];
                for k in v {
                    match &vm.memory[*k].value {
                        Value::Group(g) => arr.push(*g),
                        _ => break 'm None,
                    }
                }
                Some(ObjParam::GroupList(arr))
            }
            // if v.iter().all(|k| {
            //     matches!(&vm.memory[*k].value, Value::Group(_))
            // }) => ObjParam::GroupList(v.iter().map(f))
            // Value::Bool(b) => *b,
            _ => None,
        }
    };

    param.ok_or(RuntimeError::InvalidObjectValue {
        v: (v.value.runtime_display(vm), v.area.clone()),
        area: vm.make_area(span, code),
        call_stack: vm.get_call_stack(),
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
                            if !equality(&vm.memory[k1.0].value, &vm.memory[k2.0].value, vm) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            }
        }
        (Value::Maybe(Some(k1)), Value::Maybe(Some(k2))) => {
            equality(&vm.memory[*k1].value, &vm.memory[*k2].value, vm)
        }
        _ => a == b,
    }
}

pub fn to_pattern(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Pattern> {
    Ok(match &v.value {
        Value::Type(t) => Pattern::Type(*t),
        Value::Pattern(p) => p.clone(),
        // Value::Array(v) => if v.len() != 1 {},
        _ => {
            return Err(RuntimeError::CannotConvertType {
                v: (v.value.get_type(), v.area.clone()),
                to: ValueType::Pattern,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn is_op(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &mut Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    let pat = to_pattern(b, span, vm, code)?;

    Ok(Value::Bool(pat.value_matches(&a.value, vm)?))
}

pub fn as_op(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &mut Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(if let Value::Type(b) = &b.value {
        vm.convert_type(a, *b, span, code)?
    } else {
        return Err(RuntimeError::TypeMismatch {
            v: (b.value.get_type(), b.area.clone()),
            area: vm.make_area(span, code),
            expected: ValueType::Type,
            call_stack: vm.get_call_stack(),
        });
    })
}

pub fn add(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a + *b),
        (Value::Float(a), Value::Float(b)) => Value::Float(*a + *b),
        (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + *b),
        (Value::Float(a), Value::Int(b)) => Value::Float(*a + *b as f64),

        (Value::String(a), Value::String(b)) => Value::String([a.clone(), b.clone()].concat()),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Plus,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn sub(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn mult(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}
pub fn div(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn modulo(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn pow(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn unary_not(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match &v.value {
        Value::Bool(b) => Value::Bool(!b),
        _ => {
            return Err(RuntimeError::InvalidUnaryOperand {
                v: (v.value.get_type(), v.area.clone()),
                op: UnaryOp::ExclMark,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn unary_negate(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match &v.value {
        Value::Int(n) => Value::Int(-n),
        Value::Float(n) => Value::Float(-n),
        _ => {
            return Err(RuntimeError::InvalidUnaryOperand {
                v: (v.value.get_type(), v.area.clone()),
                op: UnaryOp::Minus,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn gt(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn lt(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn gte(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn lte(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
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
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn and(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && *b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::And,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn or(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || *b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Or,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn range(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Range(*a, *b, 1),
        (Value::Range(start, end, step), Value::Int(b)) => {
            if *step == 1 {
                Value::Range(*start, *b, *end as usize)
            } else {
                todo!()
            }
        }
        // (Value::Int(a), Value::Float(b)) => Value::Range(*a, *b as i64, 1),
        // (Value::Float(a), Value::Int(b)) => Value::Range(*a as i64, *b, 1),
        // (Value::Float(a), Value::Float(b)) => Value::Range(*a as i64, *b as i64, 1),

        // should only allow integer ranges, rounding floats is kinda stinky
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::Range,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn bin_and(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a & *b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a & *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinAnd,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn bin_or(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a | *b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a | *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn shift_left(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a << *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn shift_right(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a >> *b),

        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, code),
                call_stack: vm.get_call_stack(),
            })
        }
    })
}

pub fn eq_op(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(Value::Bool(equality(&a.value, &b.value, vm)))
}

pub fn neq_op(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    code: BytecodeKey,
) -> RuntimeResult<Value> {
    Ok(Value::Bool(!equality(&a.value, &b.value, vm)))
}

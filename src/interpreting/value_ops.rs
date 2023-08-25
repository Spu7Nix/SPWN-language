// todo: fix overflows, underflows shit

use std::rc::Rc;

use itertools::{Either, Itertools};

use super::context::Context;
use super::error::RuntimeError;
use super::multi::Multi;
use super::value::{StoredValue, Value, ValueType};
use super::vm::{DeepClone, Program, RuntimeResult, Vm};
use crate::compiling::bytecode::OptRegister;
use crate::gd::gd_object::ObjParam;
use crate::parsing::ast::VisTrait;
use crate::parsing::operators::operators::{AssignOp, BinOp, UnaryOp};
use crate::sources::CodeSpan;
use crate::util::String32;

pub fn to_obj_param(
    v: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
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
            Value::String(s) => Some(ObjParam::Text(s.to_string())),

            Value::Epsilon => Some(ObjParam::Epsilon),

            Value::TriggerFunction { group, .. } => Some(ObjParam::Group(*group)),

            Value::Array(v) => {
                let mut arr = vec![];
                for k in v {
                    match &k.borrow().value {
                        Value::Group(g) => arr.push(*g),
                        _ => break 'm None,
                    }
                }
                Some(ObjParam::GroupList(arr))
            },
            // if v.iter().all(|k| {
            //     matches!(&vm.memory[*k].value, Value::Group(_))
            // }) => ObjParam::GroupList(v.iter().map(f))
            // Value::Bool(b) => *b,
            _ => None,
        }
    };

    param.ok_or(RuntimeError::InvalidObjectValue {
        value_area: v.area.clone(),
        area: vm.make_area(span, program),
        call_stack: vm.get_call_stack(),
    })
}

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

pub fn equality(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Array(v1), Value::Array(v2)) => {
            if v1.len() != v2.len() {
                false
            } else {
                v1.iter()
                    .zip(v2)
                    .all(|(k1, k2)| equality(&k1.borrow().value, &k2.borrow().value))
            }
        },
        (Value::Dict(v1), Value::Dict(v2)) => {
            if v1.len() != v2.len() {
                false
            } else {
                for (k, k1) in v1 {
                    match v2.get(k) {
                        Some(k2) => {
                            if !equality(&k1.value().borrow().value, &k2.value().borrow().value) {
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
            equality(&k1.borrow().value, &k2.borrow().value)
        },
        (
            Value::Instance {
                typ: typ1,
                items: items1,
            },
            Value::Instance {
                typ: typ2,
                items: items2,
            },
        ) => {
            if typ1 != typ2 || items1.len() != items2.len() {
                false
            } else {
                for (k, k1) in items1 {
                    match items2.get(k) {
                        Some(k2) => {
                            if !equality(&k1.value().borrow().value, &k2.value().borrow().value) {
                                return false;
                            }
                        },
                        None => return false,
                    }
                }
                true
            }
        },
        (
            Value::Module {
                exports: exports1,
                types: types1,
            },
            Value::Module {
                exports: exports2,
                types: types2,
            },
        ) => {
            if exports1.len() != exports2.len() || types1 != types2 {
                false
            } else {
                for (k, k1) in exports1 {
                    match exports2.get(k) {
                        Some(k2) => {
                            if !equality(&k1.borrow().value, &k2.borrow().value) {
                                return false;
                            }
                        },
                        None => return false,
                    }
                }
                true
            }
        },
        (
            Value::Object {
                typ: typ1,
                params: params1,
            },
            Value::Object {
                typ: typ2,
                params: params2,
            },
        ) => {
            if typ1 != typ2 || params1.len() != params2.len() {
                false
            } else {
                for (k, k1) in params1 {
                    match params2.get(k) {
                        Some(k2) => {
                            if k1 != k2 {
                                return false;
                            }
                        },
                        None => return false,
                    }
                }
                true
            }
        },
        _ => a == b,
    }
}

fn in_op(
    a: OptRegister,
    b: OptRegister,
    mut ctx: Context,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> Multi<RuntimeResult<Value>> {
    todo!()
    // Ok(match (&a.value, &b.value) {
    //     (a, Value::Array(b)) => {
    //         todo!("context shit overloading boboggl") // context shit overloading boboggl
    //     },

    //     (Value::String(s), Value::Dict(d)) => Value::Bool(d.contains_key(s)),

    //     _ => {
    //         return Err(RuntimeError::InvalidOperands {
    //             op: BinOp::In,
    //             a: (a.value.get_type(), a.area.clone()),
    //             b: (b.value.get_type(), b.area.clone()),
    //             area: vm.make_area(span, program),
    //             call_stack: vm.get_call_stack(),
    //         })
    //     },
    // })
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

        (Value::Array(v1), Value::Array(v2)) => {
            let mut out = vec![];
            for i in v1.iter().chain(v2) {
                out.push(vm.deep_clone(i, false).into());
            }

            Value::Array(out)
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
        (Value::Int(n), Value::Array(v)) | (Value::Array(v), Value::Int(n)) => {
            let mut out = vec![];
            for i in v.iter().cycle().take(*n as usize * v.len()) {
                out.push(vm.deep_clone(i, false).into());
            }

            Value::Array(out)
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

pub fn gte(
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
                    step: *end,
                }
            } else {
                Value::Range {
                    start: *start,
                    end: *b,
                    step: *step,
                }
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
            // Huffing paint thinner makes you invincible.
            // Watch me drive this Toyota 100mph + fucking zoinked out of my gourd.
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinOr,
                area: vm.make_area(span, program),
                call_stack: vm.get_call_stack(),
            });
        },
    })
}

fn bin_xor(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(match (&a.value, &b.value) {
        (Value::Int(a), Value::Int(b)) => Value::Int(*a ^ *b),
        _ => {
            return Err(RuntimeError::InvalidOperands {
                a: (a.value.get_type(), a.area.clone()),
                b: (b.value.get_type(), b.area.clone()),
                op: BinOp::BinXor,
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
    Ok(Value::Bool(equality(&a.value, &b.value)))
}

fn neq(
    a: &StoredValue,
    b: &StoredValue,
    span: CodeSpan,
    vm: &Vm,
    program: &Rc<Program>,
) -> RuntimeResult<Value> {
    Ok(Value::Bool(!equality(&a.value, &b.value)))
}

pub type BinOpFn =
    fn(&StoredValue, &StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>;
pub type BinOpFnSplittable = fn(
    OptRegister,
    OptRegister,
    bulgaria: Context,
    CodeSpan,
    &Vm,
    &Rc<Program>,
) -> Multi<RuntimeResult<Value>>;

pub type UnaryOpFn = fn(&StoredValue, CodeSpan, &Vm, &Rc<Program>) -> RuntimeResult<Value>;

impl BinOp {
    pub fn get_fn(self) -> Either<BinOpFn, BinOpFnSplittable> {
        match self {
            BinOp::Range => Either::Left(range),
            BinOp::BinOr => Either::Left(bin_or),
            BinOp::BinAnd => Either::Left(bin_and),
            BinOp::Eq => Either::Left(eq),
            BinOp::Neq => Either::Left(neq),
            BinOp::Gt => Either::Left(gt),
            BinOp::Gte => Either::Left(gte),
            BinOp::Lt => Either::Left(lt),
            BinOp::Lte => Either::Left(lte),
            BinOp::ShiftLeft => Either::Left(shift_left),
            BinOp::ShiftRight => Either::Left(shift_right),
            BinOp::Plus => Either::Left(plus),
            BinOp::Minus => Either::Left(minus),
            BinOp::Mult => Either::Left(mult),
            BinOp::Div => Either::Left(div),
            BinOp::Mod => Either::Left(modulo),
            BinOp::Pow => Either::Left(pow),
            BinOp::BinXor => Either::Left(bin_xor),
            // BinOp::As => Either::Right(pow),
            BinOp::In => Either::Right(in_op),
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
            AssignOp::BinXorEq => bin_xor,
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

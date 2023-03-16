use serde::{Deserialize, Serialize};

use super::interpreter::{BytecodeKey, RuntimeResult, ValueKey, Vm};
use super::value::{StoredValue, Value, ValueType};
use super::value_ops;
use crate::compiling::bytecode::Constant;
use crate::gd::object_keys::ObjectKey;
use crate::parsing::ast::{Pattern, Spanned};
use crate::sources::CodeSpan;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConstPattern {
    pub pat: Pattern<ValueType, Box<ConstPattern>, Spanned<Constant>>,
}

impl Constant {
    pub fn runtime_display(&self, vm: &Vm) -> String {
        match self {
            Constant::Int(n) => n.to_string(),
            Constant::Float(n) => n.to_string(),
            Constant::Bool(b) => b.to_string(),
            Constant::String(s) => format!("{s:?}"),
            Constant::Id(class, id) => format!("{id}{}", class.letter()),
            Constant::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|c| c.runtime_display(vm))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Constant::Dict(d) => format!(
                "{{ {} }}",
                d.iter()
                    .map(|(s, c)| format!("{}: {}", s, c.runtime_display(vm)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Constant::Builtins => "$".to_string(),
            Constant::Maybe(o) => match o {
                Some(c) => format!("({})?", c.runtime_display(vm)),
                None => "?".into(),
            },
            Constant::Empty => "()".into(),

            Constant::Type(t) => t.runtime_display(vm),
            Constant::Instance(typ, items) => format!(
                "@{}::{{ {} }}",
                vm.resolve(&vm.types[*typ].value.name),
                items
                    .iter()
                    .map(|(s, c)| format!("{}: {}", s, c.runtime_display(vm)))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        }
    }
}

impl ConstPattern {
    pub fn runtime_display(&self, vm: &Vm) -> String {
        match &self.pat {
            Pattern::Type(t) => t.runtime_display(vm),
            Pattern::Either(a, b) => {
                format!("({} | {})", a.runtime_display(vm), b.runtime_display(vm))
            },
            Pattern::Both(a, b) => {
                format!("({} & {})", a.runtime_display(vm), b.runtime_display(vm))
            },
            Pattern::Any => "_".into(),
            Pattern::Eq(c) => format!("=={}", c.value.runtime_display(vm)),
            Pattern::Neq(c) => format!("!={}", c.value.runtime_display(vm)),
            Pattern::Gt(c) => format!(">{}", c.value.runtime_display(vm)),
            Pattern::Gte(c) => format!(">={}", c.value.runtime_display(vm)),
            Pattern::Lt(c) => format!("<{}", c.value.runtime_display(vm)),
            Pattern::Lte(c) => format!("<={}", c.value.runtime_display(vm)),
            _ => todo!(),
        }
    }

    pub fn value_matches(
        &self,
        v: &StoredValue,
        vm: &mut Vm,
        code: BytecodeKey,
    ) -> RuntimeResult<bool> {
        let mut to_stored_value = |c: &Spanned<Constant>| {
            let area = vm.make_area(c.span, code);
            StoredValue {
                value: Value::from_const(&c.value, vm, &area),
                area,
            }
        };

        let unwrap_bool = |v: &Value| {
            if let Value::Bool(b) = &v {
                *b
            } else {
                unreachable!()
            }
        };

        Ok(match &self.pat {
            Pattern::Type(t) => v.value.get_type() == *t,
            Pattern::Either(a, b) => {
                a.value_matches(v, vm, code)? || b.value_matches(v, vm, code)?
            },
            Pattern::Both(a, b) => a.value_matches(v, vm, code)? && b.value_matches(v, vm, code)?,
            Pattern::Any => true,
            Pattern::Eq(n) => unwrap_bool(&value_ops::eq_op(
                v,
                &to_stored_value(n),
                CodeSpan::internal(),
                vm,
                code,
            )?),
            Pattern::Neq(n) => unwrap_bool(&value_ops::neq_op(
                v,
                &to_stored_value(n),
                CodeSpan::internal(),
                vm,
                code,
            )?),
            Pattern::Lt(n) => unwrap_bool(&value_ops::lt(
                v,
                &to_stored_value(n),
                CodeSpan::internal(),
                vm,
                code,
            )?),
            Pattern::Lte(n) => unwrap_bool(&value_ops::lte(
                v,
                &to_stored_value(n),
                CodeSpan::internal(),
                vm,
                code,
            )?),
            Pattern::Gt(n) => unwrap_bool(&value_ops::gt(
                v,
                &to_stored_value(n),
                CodeSpan::internal(),
                vm,
                code,
            )?),
            Pattern::Gte(n) => unwrap_bool(&value_ops::gte(
                v,
                &to_stored_value(n),
                CodeSpan::internal(),
                vm,
                code,
            )?),
            _ => todo!(),
        })
    }
}

impl std::fmt::Debug for ConstPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.pat {
            Pattern::Any => write!(f, "_"),
            Pattern::Type(_) => write!(f, "@<type>"),
            Pattern::Either(a, b) => write!(f, "({a:?} | {b:?})"),
            Pattern::Both(a, b) => write!(f, "({a:?} & {b:?})"),
            Pattern::Eq(c) => write!(f, "=={c:?}"),
            Pattern::Neq(c) => write!(f, "!={c:?}"),
            Pattern::Lt(c) => write!(f, "<{c:?}"),
            Pattern::Lte(c) => write!(f, "<={c:?}"),
            Pattern::Gt(c) => write!(f, ">{c:?}"),
            Pattern::Gte(c) => write!(f, ">={c:?}"),
            Pattern::MacroPattern { args, ret_type } => todo!(),
        }
    }
}

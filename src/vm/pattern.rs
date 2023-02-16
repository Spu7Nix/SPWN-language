use super::interpreter::{RuntimeResult, ValueKey, Vm};
use super::value::{Value, ValueType};
use super::value_ops;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Type(ValueType),

    Either(Box<Pattern>, Box<Pattern>),
    Both(Box<Pattern>, Box<Pattern>),

    Eq(ValueKey),
    Neq(ValueKey),
    Gt(ValueKey),
    Gte(ValueKey),
    Lt(ValueKey),
    Lte(ValueKey),

    Any,
}

impl Pattern {
    pub fn runtime_display(&self, vm: &Vm) -> String {
        match self {
            Pattern::Type(t) => t.runtime_display(vm),
            Pattern::Either(a, b) => {
                format!("({} | {})", a.runtime_display(vm), b.runtime_display(vm))
            }
            Pattern::Both(a, b) => {
                format!("({} & {})", a.runtime_display(vm), b.runtime_display(vm))
            }
            Pattern::Any => "_".into(),
            Pattern::Eq(k) => format!("=={}", vm.memory[*k].value.runtime_display(vm)),
            Pattern::Neq(k) => format!("!={}", vm.memory[*k].value.runtime_display(vm)),
            Pattern::Gt(k) => format!(">{}", vm.memory[*k].value.runtime_display(vm)),
            Pattern::Gte(k) => format!(">={}", vm.memory[*k].value.runtime_display(vm)),
            Pattern::Lt(k) => format!("<{}", vm.memory[*k].value.runtime_display(vm)),
            Pattern::Lte(k) => format!("<={}", vm.memory[*k].value.runtime_display(vm)),
        }
    }

    pub fn value_matches(&self, v: &Value, vm: &mut Vm) -> RuntimeResult<bool> {
        // todo: overaloafdfing?????????
        Ok(match self {
            Pattern::Type(t) => v.get_type() == *t,
            Pattern::Either(a, b) => a.value_matches(v, vm)? || b.value_matches(v, vm)?,
            Pattern::Both(a, b) => a.value_matches(v, vm)? && b.value_matches(v, vm)?,
            Pattern::Any => true,

            _ => todo!(),
        })
    }
}

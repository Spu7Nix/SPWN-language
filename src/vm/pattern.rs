use super::interpreter::Vm;
use super::value::{Value, ValueType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Type(ValueType),

    Either(Box<Pattern>, Box<Pattern>),
    Both(Box<Pattern>, Box<Pattern>),

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
        }
    }

    pub fn value_matches(&self, v: &Value, vm: &Vm) -> bool {
        match self {
            Pattern::Type(t) => v.get_type() == *t,
            Pattern::Either(a, b) => a.value_matches(v, vm) || b.value_matches(v, vm),
            Pattern::Both(a, b) => a.value_matches(v, vm) && b.value_matches(v, vm),
            Pattern::Any => true,
        }
    }
}

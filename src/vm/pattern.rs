use super::interpreter::Vm;
use super::value::ValueType;

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
}

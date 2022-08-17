use super::{
    interpreter::{Globals, ValueKey},
    types::TypeBuilder,
    value::{Value, ValueType},
};
use crate::sources::CodeArea;
use crate::vm::to_value::ToValueResult;
use crate::{attr, method, method_arg};

impl Globals {
    pub fn init_types(&mut self) {
        TypeBuilder::new(ValueType::Array)
            .add_member(self, "length", attr!(g, Value::Array(this) => this.len()))
            .add_method(
                self,
                "push",
                method!(#mut Value::Array(this), #key el => this.push(el)),
            )
            .finish_type(self);
    }
}

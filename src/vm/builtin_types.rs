use super::{
    interpreter::{Globals, ValueKey},
    types::TypeBuilder,
    value::{value_ops, Value, ValueType},
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
                method!(g, #mut Value::Array(this), #key el => this.push(el)),
            )
            .add_method(
                self,
                "reverse",
                method!(g, #mut Value::Array(this) => this.reverse()),
            )
            .finish_type(self);

        TypeBuilder::new(ValueType::Builtins)
            .add_method(
                self,
                "print",
                method!(g, _this, val => println!("{}", val.to_str(g))),
            )
            .finish_type(self);
    }
}

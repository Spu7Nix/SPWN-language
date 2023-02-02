use super::builtin_utils::{BuiltinType, TypeName};
use crate::vm::builtins::builtin_utils::ToValue;
use crate::vm::error::RuntimeError;
use crate::vm::interpreter::Vm;
use crate::vm::value::Value;

impl BuiltinType for String {
    fn invoke_static(mname: &str, vm: &mut Vm) -> Result<Value, RuntimeError> {
        todo!()
    }

    fn invoke_self(&self, mname: &str, vm: &mut Vm) -> Result<Value, RuntimeError> {
        todo!();
        Ok(match mname {
            "length" => self.len().to_value(vm)?,
            "clear" => String::clear.to_value(vm)?,
            _ => todo!(),
        })
    }
}

impl TypeName for String {
    const NAME: &'static str = "string";
}

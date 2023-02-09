use super::interpreter::ValueKey;
use super::value::BuiltinFn;

pub mod builtin_funcs;
pub mod builtin_utils;

use builtin_utils::ToBuiltinFn;

fn test(a: bool) {}

impl builtin_utils::BuiltinType for Vec<ValueKey> {
    fn invoke_static(
        name: &str,
        vm: &mut super::interpreter::Vm,
    ) -> Result<BuiltinFn, super::error::RuntimeError> {
        todo!()
    }

    fn invoke_self(
        &self,
        name: &str,
        vm: &mut super::interpreter::Vm,
    ) -> Result<BuiltinFn, super::error::RuntimeError> {
        Ok(match name {
            "new" => (Self::new as fn() -> _).to_fn(),
            //"clear" => (Self::clear as fn(_) -> _).to_fn(),
            _ => todo!(),
        })
    }
}

//<fn(bool) -> Vec<ValueKey> as ToBuiltinFn<O, A>>::into_fn(`, `)

use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::value::Value;

impl_type! {
    impl Builtins {
        Constants:


        Functions(vm):
        fn print(
            Builtins as self = "$",
            ...args: &String,
            end: TriggerFunction = r#""\n""#,
            sep: String = r#"" ""#,
        ) {
            end.group
            // for arg in args {

            // }
            Value::Empty
        }
    }
}

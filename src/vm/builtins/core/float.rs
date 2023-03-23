use crate::vm::builtins::builtin_utils::impl_type;
use crate::vm::error::RuntimeError;
use crate::vm::value::{StoredValue, Value};
use crate::vm::value_ops;

impl_type! {
    impl Float {
        Constants:

        Functions(vm, call_area):

        fn abs(Float(n) as self) -> Float {
            Value::Float(n.abs())
        }
        fn sign(Float(n) as self) -> Float {
            Value::Float(n.signum())
        }

        fn round(Float(n) as self) -> Float {
            Value::Float(n.round())
        }
        fn ceil(Float(n) as self) -> Float {
            Value::Float(n.ceil())
        }
        fn floor(Float(n) as self) -> Float {
            Value::Float(n.floor())
        }
        fn trunc(Float(n) as self) -> Float {
            Value::Float(n.trunc())
        }

        fn sqrt(Float(n) as self) -> Float {
            Value::Float(n.sqrt())
        }
        fn log(Float(n) as self, Float(base) as base = {2.71828182845904523536028747135266250}) -> Float {
            Value::Float(n.log(base))
        }

        fn lerp(Float(n) as self, Float(a) as a, Float(b) as b) -> Float {
            Value::Float(a + (b - a) * n)
        }

        fn map(Float(n) as self, Float(a) as a, Float(b) as b, Float(c) as c, Float(d) as d) -> Float {
            Value::Float(c + (d - c) * ((n - a) / (b - a)))
        }

        fn clamp(Float(n) as self, Float(min) as min, Float(max) as max) -> Float {
            Value::Float(n.max(min).min(max))
        }
        fn wrap(Float(n) as self, Float(min) as min, Float(max) as max) -> Float {
            Value::Float(((n - min) % (max - min)) + min)
        }
    }
}

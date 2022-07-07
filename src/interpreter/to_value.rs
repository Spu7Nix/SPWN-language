use super::error::RuntimeError;
use super::value::Value;

use super::from_value::Error;

pub trait ToValue {
    fn to_value(self) -> Value;
}

pub trait ToValueResult {
    fn to_value_result(self) -> Result<Value, Error>;
}

impl<R: ToValue> ToValueResult for R {
    fn to_value_result(self) -> Result<Value, Error> {
        Ok(self.to_value())
    }
}

impl ToValue for String {
    fn to_value(self) -> Value {
        Value::String(self)
    }
}

impl ToValue for bool {
    fn to_value(self) -> Value {
        Value::Bool(self)
    }
}

impl ToValue for () {
    fn to_value(self) -> Value {
        Value::Empty
    }
}

// TODO: other values

macro_rules! num_to_value {
    ($($in:ty)*; $($fn:ty)*) => {
        $(
            impl ToValue for $in {
                fn to_value(self) -> Value {
                    Value::Int(self as isize)
                }
            }
        )*
        $(
            impl ToValue for $fn {
                fn to_value(self) -> Value {
                    Value::Float(self.into())
                }
            }
        )*
    };
}
num_to_value! { i8 i16 i32 i64 i128 isize; f32 f64 }

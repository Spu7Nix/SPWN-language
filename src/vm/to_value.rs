use super::error::RuntimeError;
use super::value::Value;

pub trait ToValue {
    fn to_value(self) -> Value;
}

pub trait ToValueResult {
    fn try_to_value(self) -> Result<Value, RuntimeError>;
}

impl<R: ToValue> ToValueResult for R {
    fn try_to_value(self) -> Result<Value, RuntimeError> {
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
        Value::Empty()
    }
}

macro_rules! num_to_value {
    ($($in:ty)*; $($fn:ty)*) => {
        $(
            impl ToValue for $in {
                fn to_value(self) -> Value {
                    Value::Int(self as i64)
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

impl ToValue for Value {
    fn to_value(self) -> Value {
        self
    }
}

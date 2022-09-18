use crate::leveldata::object_data::GdObj;

use super::{error::RuntimeError, interpreter::ValueKey, value::SpwnIterator, value::Value};

pub trait FromValue: Sized {
    fn from_value(val: &Value) -> Result<&Self, RuntimeError>;

    fn from_value_mut(_: &mut Value) -> Result<&mut Self, RuntimeError> {
        unimplemented!()
    }
}

impl FromValue for SpwnIterator {
    fn from_value(val: &Value) -> Result<&Self, RuntimeError> {
        if let Value::Iterator(a) = val {
            Ok(a)
        } else {
            todo!()
        }
    }

    fn from_value_mut(val: &mut Value) -> Result<&mut Self, RuntimeError> {
        if let Value::Iterator(a) = val {
            Ok(a)
        } else {
            todo!("{:?}", val)
        }
    }
}

impl FromValue for Vec<ValueKey> {
    fn from_value(val: &Value) -> Result<&Self, RuntimeError> {
        if let Value::Array(a) = val {
            Ok(a)
        } else {
            // Err(RuntimeError::TypeMismatch {
            //     v: val.into_stored(CodeArea::internal()),
            //     expected: ValueType::Array.into(),
            //     area: CodeArea::internal(),
            // })
            todo!()
        }
    }

    fn from_value_mut(val: &mut Value) -> Result<&mut Self, RuntimeError> {
        if let Value::Array(a) = val {
            Ok(a)
        } else {
            // Err(RuntimeError::TypeMismatch {
            //     v: val.into_stored(CodeArea::internal()),
            //     expected: ValueType::Array.into(),
            //     area: CodeArea::internal(),
            // })
            todo!()
        }
    }
}

impl FromValue for GdObj {
    fn from_value(val: &Value) -> Result<&Self, RuntimeError> {
        if let Value::Object(o) = val {
            Ok(o)
        } else {
            // Err(RuntimeError::TypeMismatch {
            //     v: val.into_stored(CodeArea::internal()),
            //     expected: ValueType::Array.into(),
            //     area: CodeArea::internal(),
            // })
            todo!()
        }
    }
}

// impl<T: 'static> FromValue for *mut T {
//     fn from_value(val: Value) -> Result<Self, RuntimeError> {
//         if let Value::Instance(i) = val {
//             Ok(i.downcast_ref())
//         } else {
//             unreachable!("internal error: downcast type mismatch")
//         }
//     }
// }

use super::interpreter::StoredValue;

use crate::error_maker;
use crate::sources::CodeArea;

error_maker! {
    pub enum RuntimeError {
        #[
            Message = "Invalid operands", Area = area, Note = None,
            Labels = [
                area => "Operator `{}` cannot be used on {} and {}": @(op), @(a.value.get_type().to_str()), @(b.value.get_type().to_str());
                a.def_area => "This is of type {}": @(a.value.get_type().to_str());
                b.def_area => "This is of type {}": @(b.value.get_type().to_str());
            ]
        ]
        InvalidOperands {
            a: StoredValue,
            b: StoredValue,
            op: String,
            area: CodeArea,
        },
    }
}

// custom wrapper `Result` type as all errors will be runtime errors
pub type Result<T> = std::result::Result<T, RuntimeError>;
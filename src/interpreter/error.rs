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
        #[
            Message = "Invalid unary operand", Area = area, Note = None,
            Labels = [
                area => "Unary operator `{}` cannot be used on {}": @(op), @(a.value.get_type().to_str());
                a.def_area => "This is of type {}": @(a.value.get_type().to_str());
            ]
        ]
        InvalidUnaryOperand {
            a: StoredValue,
            op: String,
            area: CodeArea,
        },
        #[
            Message = "Cannot convert to bool", Area = a.def_area, Note = None,
            Labels = [
                a.def_area => "{} can't be converted to a boolean": @(a.value.get_type().to_str());
            ]
        ]
        BoolConversion {
            a: StoredValue,
        },
        #[
            Message = "Use of undefined type", Area = area, Note = None,
            Labels = [
                area => "{} is undefined": @(format!("@{}", name));
            ]
        ]
        UndefinedType {
            name: String,
            area: CodeArea,
        },
        #[
            Message = "Invalid call base", Area = area, Note = None,
            Labels = [
                area => "Cannot call {}": @(base.value.get_type().to_str());
                base.def_area => "Value was defined as {} here": @(base.value.get_type().to_str());
            ]
        ]
        CannotCall {
            base: StoredValue,
            area: CodeArea,
        },
    }
}

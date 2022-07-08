use super::interpreter::StoredValue;

use crate::error_maker;
use crate::interpreter::value::ValueType;
use crate::sources::CodeArea;

error_maker! {
    Globals: globals;
    Module: runtime_errors;
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
            Message = "Cannot convert type", Area = a.def_area, Note = None,
            Labels = [
                a.def_area => "{} can't be converted to a {}": @(a.value.get_type().to_str()), @(to.to_str());
            ]
        ]
        CannotConvert {
            a: StoredValue,
            to: ValueType,
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

        #[
            Message = "Use of undefined macro argument", Area = area, Note = None,
            Labels = [
                area => "Argument `{}` is undefined": @(name);
                macr.def_area => "Macro defined here";
            ]
        ]
        UndefinedArgument {
            name: String,
            macr: StoredValue,
            area: CodeArea,
        },

        #[
            Message = "Type mismatch", Area = area, Note = None,
            Labels = [
                area => "Expected {}, found {}": @(expected), @(v.value.get_type().to_str());
                v.def_area => "This is of type {}": @(v.value.get_type().to_str());
            ]
        ]
        TypeMismatch {
            v: StoredValue,
            expected: String,
            area: CodeArea,
        },

        #[
            Message = "Pattern mismatch", Area = area, Note = None,
            Labels = [
                area => "This {} is not {}": @(v.value.get_type().to_str()), @(pat.value.to_str(globals));
                v.def_area => "This is of type {}": @(v.value.get_type().to_str());
                pat.def_area => "Pattern defined as {} here": @(pat.value.to_str(globals));
            ]
        ]
        PatternMismatch {
            v: StoredValue,
            pat: StoredValue,
            area: CodeArea,
        },

        #[
            Message = "Argument not satisfied", Area = call_area, Note = None,
            Labels = [
                arg_area => "Argument `{}` defined as mandatory here": @(arg_name);
                call_area => "Argument not provided here";
            ]
        ]
        ArgumentNotSatisfied {
            arg_name: String,
            call_area: CodeArea,
            arg_area: CodeArea,
        },

        #[
            Message = "Too many arguments!", Area = call_area, Note = None,
            Labels = [
                func.def_area => "Macro defined to take {} arguments here": @(expected);
                call_area => "Called with {} arguments": @(provided);
            ]
        ]
        TooManyArguments {
            expected: usize,
            provided: usize,
            call_area: CodeArea,
            func: StoredValue,
        },

        #[
            Message = "Type has no constructor!", Area = area, Note = None,
            Labels = [
                area => "Tried to call `{}`'s constructor here": @(typ);
            ]
        ]
        NoConstructor {
            typ: String,
            area: CodeArea,
        },

        // #[
        //     Message = "", Area = area, Note = None,
        //     Labels = [
        //         area => "";
        //     ]
        // ]
        // Wrapper {
        //     err: RuntimeError,
        //     area: CodeArea,
        // },
    }
}

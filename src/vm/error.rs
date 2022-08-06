use crate::{error_maker, sources::CodeArea};

use super::value::{StoredValue, ValueType};

error_maker! {
    Module: runtime_errors;
    pub enum RuntimeError {
        #[
            Message = "Undefined variable", Area = area, Note = None,
            Labels = [
                area => "This variable is not defined yet!";
            ]
        ]
        UndefinedVariable {
            area: CodeArea,
        },

        #[
            Message = "Invalid operands", Area = area, Note = None,
            Labels = [
                area => "Operator `{}` cannot be used on {} and {}": @(op), @(a.value.typ().to_str()), @(b.value.typ().to_str());
                a.def_area => "This is of type {}": @(a.value.typ().to_str());
                b.def_area => "This is of type {}": @(b.value.typ().to_str());
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
                area => "Unary operator `{}` cannot be used on {}": @(op), @(a.value.typ().to_str());
                a.def_area => "This is of type {}": @(a.value.typ().to_str());
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
                a.def_area => "{} can't be converted to a {}": @(a.value.typ().to_str()), @(to.to_str());
            ]
        ]
        CannotConvert {
            a: StoredValue,
            to: ValueType,
        },

        // #[
        //     Message = "Not an iterator", Area = a.def_area, Note = None,
        //     Labels = [
        //         a.def_area => "Cannot iterate over {}": @(a.value.typ().to_str());
        //     ]
        // ]
        // CannotIterate {
        //     a: StoredValue,
        // },

        // #[
        //     Message = "Use of undefined type", Area = area, Note = None,
        //     Labels = [
        //         area => "{} is undefined": @(format!("@{}", name));
        //     ]
        // ]
        // UndefinedType {
        //     name: String,
        //     area: CodeArea,
        // },

        #[
            Message = "Invalid call base", Area = area, Note = None,
            Labels = [
                area => "Cannot call {}": @(base.value.typ().to_str());
                base.def_area => "Value was defined as {} here": @(base.value.typ().to_str());
            ]
        ]
        CannotCall {
            base: StoredValue,
            area: CodeArea,
        },

        // #[
        //     Message = "Use of undefined macro argument", Area = area, Note = None,
        //     Labels = [
        //         area => "Argument `{}` is undefined": @(name);
        //         macr.def_area => "Macro defined here";
        //     ]
        // ]
        // UndefinedArgument {
        //     name: String,
        //     macr: StoredValue,
        //     area: CodeArea,
        // },

        #[
            Message = "Type mismatch", Area = area, Note = None,
            Labels = [
                area => "Expected {}, found {}": @(expected), @(v.value.typ().to_str());
                v.def_area => "This is of type {}": @(v.value.typ().to_str());
            ]
        ]
        TypeMismatch {
            v: StoredValue,
            expected: String,
            area: CodeArea,
        },

        // #[
        //     Message = "Pattern mismatch", Area = area, Note = None,
        //     Labels = [
        //         area => "This {} is not {}": @(v.value.typ().to_str()), @(pat.0.to_str());
        //         v.def_area => "This is of type {}": @(v.value.typ().to_str());
        //         pat.1 => "Pattern defined as {} here": @(pat.0.to_str());
        //     ]
        // ]
        // PatternMismatch {
        //     v: StoredValue,
        //     pat: (Pattern, CodeArea),
        //     area: CodeArea,
        // },

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
                func_area => "Macro defined to take {} arguments here": @(expected);
                call_area => "Called with {} arguments": @(provided);
            ]
        ]
        TooManyArguments {
            expected: usize,
            provided: usize,
            call_area: CodeArea,
            func_area: CodeArea,
        },

        // #[
        //     Message = "Type has no constructor!", Area = area, Note = None,
        //     Labels = [
        //         area => "Tried to call `{}`'s constructor here": @(typ);
        //     ]
        // ]
        // NoConstructor {
        //     typ: String,
        //     area: CodeArea,
        // },

        // #[
        //     Message = "Use of undefined member!", Area = area, Note = None,
        //     Labels = [
        //         area => "`{}` is undefined": @(name);
        //     ]
        // ]
        // UndefinedMember {
        //     name: String,
        //     area: CodeArea,
        // },
    }
}

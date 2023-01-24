use std::string::ToString;

use super::value::ValueType;
use crate::error_maker;
use crate::parsing::utils::operators::{BinOp, UnaryOp};
use crate::sources::CodeArea;

error_maker! {
    Title: "Runtime Error"
    Extra: {}
    pub enum RuntimeError {
        /////////
        #[
            Message: "Invalid operands", Note: None;
            Labels: [
                area => "Invalid operands for `{}` operator": op.to_str();
                a.1 => "This is of type `{}`": a.0;
                b.1 => "This is of type `{}`": b.0;
            ]
        ]
        InvalidOperands {
            a: (ValueType, CodeArea),
            b: (ValueType, CodeArea),
            op: BinOp,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Invalid unary operand", Note: None;
            Labels: [
                area => "Invalid operand for `{}` unary operator": op.to_str();
                v.1 => "This is of type `{}`": v.0;
            ]
        ]
        InvalidUnaryOperand {
            v: (ValueType, CodeArea),
            op: UnaryOp,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Type mismatch", Note: None;
            Labels: [
                area => "Expected `{}`, found `{}`": expected, v.0;
                v.1 => "Value defined as `{}` here": v.0;
            ]
        ]
        TypeMismatch {
            v: (ValueType, CodeArea),
            area: CodeArea,
            expected: ValueType,
        },

        /////////
        #[
            Message: "Too many arguments", Note: None;
            Labels: [
                call_area => "Received {} arguments, expected {}": call_arg_amount, macro_arg_amount;
                macro_def_area => "Macro defined to take {} arguments here": macro_arg_amount;
            ]
        ]
        TooManyArguments {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            macro_arg_amount: usize,
            call_arg_amount: usize,
        },

        /////////
        #[
            Message: "Nonexistent argument", Note: None;
            Labels: [
                call_area => "Argument `{}` does not exist": arg_name;
                macro_def_area => "Macro defined here";
            ]
        ]
        NonexistentArgument {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg_name: String,
        },

        /////////
        #[
            Message: "Argument not satisfied", Note: None;
            Labels: [
                call_area => "Argument `{}` not satisfied": arg_name;
                macro_def_area => "Macro defined here";
            ]
        ]
        ArgumentNotSatisfied {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg_name: String,
        },
    }
}

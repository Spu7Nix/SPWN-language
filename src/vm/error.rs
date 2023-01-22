use std::string::ToString;

use crate::error_maker;
use crate::lexing::tokens::Token;
use crate::parsing::utils::operators::{BinOp, UnaryOp};
use crate::sources::CodeArea;

use super::value::ValueType;

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
    }
}

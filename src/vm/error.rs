use std::string::ToString;

use crate::error_maker;
use crate::lexing::tokens::Token;
use crate::parsing::utils::operators::BinOp;
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
                a.1 => "This is of type `{}`": a.0.type_name();
                b.1 => "This is of type `{}`": b.0.type_name();
            ]
        ]
        InvalidOperands {
            a: (ValueType, CodeArea),
            b: (ValueType, CodeArea),
            op: BinOp,
            area: CodeArea,
        },

    }
}

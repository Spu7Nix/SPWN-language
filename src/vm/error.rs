use std::string::ToString;

use super::context::CallStackItem;
use super::interpreter::Vm;
use super::value::ValueType;
use crate::error_maker;
use crate::parsing::utils::operators::{BinOp, UnaryOp};
use crate::sources::CodeArea;
use crate::util::hyperlink;
use crate::vm::builtins::builtin_utils::BuiltinValueType;

error_maker! {
    Title: "Runtime Error"
    Extra: {
        vm: &Vm,
    }
    pub enum RuntimeError {
        /////////
        #[
            Message: "Invalid operands", Note: None;
            Labels: [
                area => "Invalid operands for `{}` operator": op.to_str();
                a.1 => "This is of type {}": a.0.runtime_display(vm);
                b.1 => "This is of type {}": b.0.runtime_display(vm);
            ]
        ]
        InvalidOperands {
            a: (ValueType, CodeArea),
            b: (ValueType, CodeArea),
            op: BinOp,
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid unary operand", Note: None;
            Labels: [
                area => "Invalid operand for `{}` unary operator": op.to_str();
                v.1 => "This is of type {}": v.0.runtime_display(vm);
            ]
        ]
        InvalidUnaryOperand {
            v: (ValueType, CodeArea),
            op: UnaryOp,
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Type mismatch", Note: None;
            Labels: [
                area => "Expected {}, found {}": expected.runtime_display(vm), v.0.runtime_display(vm);
                v.1 => "Value defined as {} here": v.0.runtime_display(vm);
            ]
        ]
        TypeMismatch {
            v: (ValueType, CodeArea),
            area: CodeArea,
            expected: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Cannot instance builtin type", Note: None;
            Labels: [
                area => "Cannot instance builtin type {}": typ.runtime_display(vm);
            ]
        ]
        CannotInstanceBuiltinType {
            area: CodeArea,
            typ: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid object value", Note: None;
            Labels: [
                v.1 => "{} is not a valid object value": v.0;
                area => "Object key used here";
            ]
        ]
        InvalidObjectValue {
            v: (String, CodeArea),
            area: CodeArea,
            [call_stack]
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
            [call_stack]
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
            [call_stack]
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
            [call_stack]
        },

        /////////
        #[
            Message: "Nonexistent member", Note: None;
            Labels: [
                area => "Member `{}` does not exist on this {}": member, base_type.runtime_display(vm);
            ]
        ]
        NonexistentMember {
            area: CodeArea,
            member: String,
            base_type: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Nonexistent type member", Note: None;
            Labels: [
                area => "Type {} does not exist in this module": format!("@{type_name}");
            ]
        ]
        NonexistentTypeMember {
            area: CodeArea,
            type_name: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid index", Note: None;
            Labels: [
                area => "{} cannot be indexed by {}": base.0.runtime_display(vm), index.0.runtime_display(vm);
                base.1 => "This is of type {}": base.0.runtime_display(vm);
                index.1 => "This is of type {}": index.0.runtime_display(vm);
            ]
        ]
        InvalidIndex {
            base: (ValueType, CodeArea),
            index: (ValueType, CodeArea),
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Index out of bounds", Note: None;
            Labels: [
                area => "Index {} is out of bounds for this {} of length {}": index, typ.runtime_display(vm), len;
            ]
        ]
        IndexOutOfBounds {
            len: usize,
            index: i64,
            area: CodeArea,
            typ: ValueType,
            [call_stack]
        },

        /////////
        #[
            Message: "Assertion failed", Note: None;
            Labels: [
                area => "Assertion happened here";
            ]
        ]
        AssertionFailed {
            area: CodeArea,
            [call_stack]
        },

        /////////
        #[
            Message: "Equality assertion failed", Note: None;
            Labels: [
                area => "{} is not equal to {}": left, right;
            ]
        ]
        EqAssertionFailed {
            area: CodeArea,
            left: String,
            right: String,
            [call_stack]
        },

        /////////
        #[
            Message: "Too few arguments provided to builtin", Note: Some(format!("The valid builtins are listed {}", hyperlink("https://spu7nix.net/spwn/#/builtins?id=list-of-built-in-functions", Some("here"))));
            Labels: [
                call_area => "Builtin called here";
            ]
        ]
        TooFewBuiltinArguments {
            call_area: CodeArea,
            //builtin: Builtin,
            [call_stack]
        },

        /////////
        #[
            Message: "Too many arguments provided to builtin", Note: Some(format!("The valid builtins are listed {}", hyperlink("https://spu7nix.net/spwn/#/builtins?id=list-of-built-in-functions", Some("here"))));
            Labels: [
                call_area => "Builtin called here";
            ]
        ]
        TooManyBuiltinArguments {
            call_area: CodeArea,
            //builtin: Builtin,
            [call_stack]
        },

        /////////
        #[
            Message: "Invalid builtin argument type", Note: Some(format!("The valid builtins are listed {}", hyperlink("https://spu7nix.net/spwn/#/builtins?id=list-of-built-in-functions", Some("here"))));
            Labels: [
                call_area => "Builtin expected type {} here": expected;
                def_area => "Value defined as {} here": found.runtime_display(vm);
            ]
        ]
        InvalidBuiltinArgumentType {
            call_area: CodeArea,
            def_area: CodeArea,
            expected: BuiltinValueType,
            found: ValueType,
            [call_stack]
        },
    }
}

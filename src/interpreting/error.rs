use std::string::ToString;

use itertools::Either;

use super::context::CallInfo;
use super::value::{StoredValue, Value, ValueType};
use super::vm::Vm;
use crate::error_maker;
use crate::interpreting::vm::ValueRef;
use crate::parsing::operators::operators::{BinOp, UnaryOp};
use crate::sources::CodeArea;
use crate::util::hyperlink;

error_maker! {
    Title: "Runtime Error"
    Extra: {
        vm: &Vm,
    }
    #[derive(strum::EnumDiscriminants, PartialEq)]
    #[strum_discriminants(name(ErrorDiscriminants), derive(delve::EnumVariantNames), delve(rename_variants = "SCREAMING_SNAKE_CASE"))]
    pub enum RuntimeError {

        // ==================================================================
        #[
            Message: "Invalid operands", Note: None;
            Main Area: area;
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

        // ==================================================================
        #[
            Message: "Invalid unary operand", Note: None;
            Main Area: area;
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

        // ==================================================================
        #[
            Message: "Type mismatch", Note: None;
            Main Area: area;
            Labels: [
                area => "Expected {}, found {}": {
                    use itertools::Itertools;
                    let len = expected.len();
                    expected.iter().enumerate().map(|(i, t)| {
                        if len > 1 && i == len - 1 {
                            format!("or {}", t.runtime_display(vm))
                        } else {
                            t.runtime_display(vm)
                        }
                    }).join(if len > 2 { ", " } else { " " })
                }, value_type.runtime_display(vm);
                value_area => "Value defined as {} here": value_type.runtime_display(vm);
            ]
        ]
        TypeMismatch {
            value_type: ValueType,
            value_area: CodeArea,
            area: CodeArea,
            expected: &'static [ValueType],
            [call_stack]
        },

        // // ==================================================================
        // #[
        //     Message: "Cannot convert between types", Note: None;
        //     Labels: [
        //         area => "Cannot convert {} to {}": v.0.runtime_display(vm), to.runtime_display(vm);
        //         v.1 => "This is of type {}": v.0.runtime_display(vm);
        //     ]
        // ]
        // CannotConvertType {
        //     v: (ValueType, CodeArea),
        //     to: ValueType,
        //     area: CodeArea,
        //     [call_stack]
        // },

        // ==================================================================
        #[
            Message: "Cannot iterator", Note: match value.0 {
                ValueType::Custom(..) => Some("Try overloading the `_iter_` method (`#[overload = _iter_]`)".into()),
                _ => None,
            };
            Main Area: area;
            Labels: [
                area => "Cannot iterate over {}": value.0.runtime_display(vm);
                value.1 => "Value defined as {} here": value.0.runtime_display(vm);
            ]
        ]
        CannotIterate {
            value: (ValueType, CodeArea),
            area: CodeArea,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Cannot instance builtin type", Note: None;
            Main Area: area;
            Labels: [
                area => "Cannot instance builtin type {}": typ.runtime_display(vm);
            ]
        ]
        CannotInstanceBuiltinType {
            area: CodeArea,
            typ: ValueType,
            [call_stack]
        },

        // // ==================================================================
        // #[
        //     Message: "Invalid object value", Note: None;
        //     Labels: [
        //         v.1 => "{} is not a valid object value": v.0;
        //         area => "Object key used here";
        //     ]
        // ]
        // InvalidObjectValue {
        //     v: (String, CodeArea),
        //     area: CodeArea,
        //     [call_stack]
        // },

        // ==================================================================
        #[
            Message: "Too many arguments", Note: None;
            Main Area: call_area;
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

        // ==================================================================
        #[
            Message: "Unknown keyword argument", Note: None;
            Main Area: call_area;
            Labels: [
                macro_def_area => "Macro defined to take these arguments";
                call_area => "Keyword argument `{}` received here": name;
            ]
        ]
        UnknownKeywordArgument {
            name: String,
            macro_def_area: CodeArea,
            call_area: CodeArea,
            [call_stack]
        },

        // // ==================================================================
        // #[
        //     Message: "Invalid keyword argument", Note: None;
        //     Labels: [
        //         call_area => "Keyword argument `{}` is invalid": arg_name;
        //         macro_def_area => "Macro defined here";
        //     ]
        // ]
        // InvalidKeywordArgument {
        //     call_area: CodeArea,
        //     macro_def_area: CodeArea,
        //     arg_name: String,
        //     [call_stack]
        // },

        // ==================================================================
        #[
            Message: "Argument not satisfied", Note: None;
            Main Area: call_area;
            Labels: [
                macro_def_area => "Macro defined to take these arguments";
                call_area => "Argument {} not satisfied": match arg {
                    Either::Left(name) => format!("`{}`", name),
                    Either::Right(idx) => format!("at pos {}", idx),
                };
            ]
        ]
        ArgumentNotSatisfied {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg: Either<String, usize>,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Mutable argument required",
            Note: Some(format!("Use `{}` to define a variable as mutable: `mut ... = ...`", hyperlink("https://spu7nix.net/spwn/#/triggerlanguage/1variables?id=variables", Some("mut"))));
            Main Area: call_area;
            Labels: [
                macro_def_area => "This macro changes the value of this argument";
                call_area => "The value passed to argument {} must be mutable": match arg {
                    Either::Left(name) => format!("`{}`", name),
                    Either::Right(idx) => format!("at pos {}", idx),
                };
            ]
        ]
        ArgumentNotMutable {
            call_area: CodeArea,
            macro_def_area: CodeArea,
            arg: Either<String, usize>,
            [call_stack]
        },


        // ==================================================================
        #[
            Message: "Pattern mismatch", Note: None;
            Main Area: pattern_area;
            Labels: [
                pattern_area => "The {} doesn't match this pattern": v.0.runtime_display(vm);
                v.1 => "Value defined as {} here": v.0.runtime_display(vm);
            ]
        ]
        PatternMismatch {
            v: (ValueType, CodeArea),
            pattern_area: CodeArea,
            [call_stack]
        },

        // // ==================================================================
        // #[
        //     Message: "Argument pattern mismatch", Note: None;
        //     Labels: [
        //         call_area => "Call occurred here";
        //         macro_def_area => "Argument `{}` was defined as taking {} here": arg_name, pattern.runtime_display(vm);
        //         v.1 => "This `{}` is not {}": v.0.runtime_display(vm), pattern.runtime_display(vm);
        //     ]
        // ]
        // ArgumentPatternMismatch {
        //     call_area: CodeArea,
        //     macro_def_area: CodeArea,
        //     arg_name: String,
        //     pattern: ConstPattern,
        //     v: (ValueType, CodeArea),
        //     [call_stack]
        // },

        // ==================================================================
        #[
            Message: "Nonexistent member", Note: None;
            Main Area: area;
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

        // ==================================================================
        #[
            Message: "Tried to access private member", Note: None;
            Main Area: area;
            Labels: [
                area => "Member `{}` is private": member;
            ]
        ]
        PrivateMemberAccess {
            area: CodeArea,
            member: String,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Nonexistent associated member", Note: None;
            Main Area: area;
            Labels: [
                area => "Associated member `{}` does not exist on {}": member, base_type.runtime_display(vm);
            ]
        ]
        NonexistentAssociatedMember {
            area: CodeArea,
            member: String,
            base_type: ValueType,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Associated member is not method", Note: if *member_type == ValueType::Macro {
                Some("Methods require a `self` argument".to_string())
            } else {
                None
            };
            Main Area: area;
            Labels: [
                area => "Member `{}` implemented on type {} is not a method": member_name, base_type.runtime_display(vm);
                def_area => "Member defined as {} here": member_type.runtime_display(vm);
            ]
        ]
        AssociatedMemberNotAMethod {
            area: CodeArea,
            def_area: CodeArea,
            member_name: String,
            member_type: ValueType,
            base_type: ValueType,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Nonexistent type member", Note: None;
            Main Area: area;
            Labels: [
                area => "Type {} does not exist in this module": format!("@{type_name}");
            ]
        ]
        NonexistentTypeMember {
            area: CodeArea,
            type_name: String,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Tried to access private type", Note: None;
            Main Area: area;
            Labels: [
                area => "Type {} is private": format!("@{type_name}");
            ]
        ]
        PrivateType {
            area: CodeArea,
            type_name: String,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Invalid index", Note: None;
            Main Area: area;
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

        // ==================================================================
        #[
            Message: "Index out of bounds", Note: None;
            Main Area: area;
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

        // // ==================================================================
        // #[
        //     Message: "Assertion failed", Note: None;
        //     Labels: [
        //         area => "Assertion happened here";
        //     ]
        // ]
        // AssertionFailed {
        //     area: CodeArea,
        //     [call_stack]
        // },

        // // ==================================================================
        // #[
        //     Message: "Equality assertion failed", Note: None;
        //     Labels: [
        //         area => "{} is not equal to {}": left, right;
        //     ]
        // ]
        // EqAssertionFailed {
        //     area: CodeArea,
        //     left: String,
        //     right: String,
        //     [call_stack]
        // },

        // // ==================================================================
        // #[
        //     Message: "Added object in runtime context", Note: Some("TODO (link to docs)".into());
        //     Labels: [
        //         area => "Cannot add this object at runtime";
        //     ]
        // ]
        // AddObjectInTriggerContext {
        //     area: CodeArea,
        //     [call_stack]
        // },

        // ==================================================================
        #[
            Message: "Thrown error", Note: None;
            Main Area: area;
            Labels: [
                area => "{}": "69";
            ]
        ]
        ThrownError {
            area: CodeArea,
            value: ValueRef,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Cannot implement on a builtin type", Note: None;
            Main Area: area;
            Labels: [
                area => "Implementation happens here";
            ]
        ]
        ImplOnBuiltin {
            area: CodeArea,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Returning twice from this macro/module is not allowed", Note: None;
            Main Area: area;
            Labels: [
                area => "Context split happens here";
            ]
        ]
        ContextSplitDisallowed {
            area: CodeArea,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: "Attempted to divide by zero", Note: None;
            Main Area: area;
            Labels: [
                area => "Division occurs here";
            ]
        ]
        DivisionByZero {
            area: CodeArea,
            [call_stack]
        },


        // ==================================================================
        #[
            Message: "Recursion limit", Note: None;
            Main Area: area;
            Labels: [
                area => "Reached maximum recursion limit from this macro call";
            ]
        ]
        RecursionLimit {
            area: CodeArea,
            [call_stack]
        },

        // ==================================================================
        #[
            Message: format!("Error `{}` occurred while running an overload", error.to_report(vm).message), Note: None;
            Main Area: area;
            Labels: [
                area => "Call of `{}` occurrs here": builtin;
                -> error.to_report(vm).labels
            ]
        ]
        WhileCallingOverload {
            area: CodeArea,
            error: Box<RuntimeError>,
            builtin: &'static str,
        },

        // ==================================================================
        #[
            Message: "Cannot convert type", Note: None;
            Main Area: from_area;
            Labels: [
                from_area => "{} can't be converted to a {}": from_type.runtime_display(vm), to.runtime_display(vm);
            ]
        ]
        CannotConvert {
            from_type: ValueType,
            from_area: CodeArea,
            to: ValueType,
        },

        // ==================================================================
        #[
            Message: "Invalid string for conversion", Note: None;
            Main Area: area;
            Labels: [
                area => "This string cannot be converted to {}": to.runtime_display(vm);
            ]
        ]
        InvalidStringForConversion {
            area: CodeArea,
            to: ValueType,
        },


        // // ============================ BUILTIN FUNC ERRORS ============================



        // // ==================================================================
        // #[
        //     Message: "Invalid hex code", Note: None;
        //     Labels: [
        //         area => "{}": => (msg);
        //     ]
        // ]
        // InvalidHexString {
        //     area: CodeArea,
        //     msg: String,
        //     [call_stack]
        // },
    }
}

// impl RuntimeError {
//     pub fn to_value(&self, vm: &mut Vm) -> Value {
//         match self {
//             RuntimeError::InvalidOperands { a, b, op, area, call_stack } => todo!(),
//             RuntimeError::InvalidUnaryOperand { v, op, area, call_stack } => todo!(),
//             RuntimeError::TypeMismatch { v, area, expected, call_stack } => todo!(),
//             RuntimeError::ThrownError { area, message, call_stack } => todo!(),
//             RuntimeError::ContextSplitDisallowed { area, call_stack } => todo!(),
//         }
//     }

//     pub fn get
// }

// impl R

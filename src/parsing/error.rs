use std::string::ToString;

use super::ast::AttrStyle;
use crate::error_maker;
use crate::lexing::lexer::LexerError;
use crate::lexing::tokens::Token;
use crate::sources::CodeArea;

fn list_join<T: std::fmt::Display>(l: &[T]) -> String {
    l.iter()
        .map(|v| format!("`{v}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

error_maker! {
    Title: "Syntax Error"
    Extra: {}
    pub enum SyntaxError {
        // ==================================================================
        #[
            Message: "Unexpected token", Note: None;
            Main Area: area;
            Labels: [
                area => "Expected `{}`, found `{}`": expected, found.to_str();
            ]
        ]
        UnexpectedToken {
            expected: String,
            found: Token,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unmatched token", Note: None;
            Main Area: area;
            Labels: [
                area => "Couldn't find matching `{}` for this `{}`": not_found.to_str(), for_char.to_str();
            ]
        ]
        UnmatchedToken {
            for_char: Token,
            not_found: Token,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unexpected character", Note: None;
            Main Area: area;
            Labels: [
                area => "Expected `{}`, found `{}`": expected.to_str(), found;
            ]
        ]
        UnexpectedCharacter {
            expected: Token,
            found: String,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unexpected string flag", Note: None;
            Main Area: area;
            Labels: [
                area => "Expected valid string flag, found `{}`": flag;
            ]
        ]
        UnexpectedFlag {
            flag: String,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Error parsing escape sequence", Note: None;
            Main Area: area;
            Labels: [
                area => "Unknown escape sequence \\`{}`": character;
            ]
        ]
        InvalidEscape {
            character: char,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Error parsing unicode escape sequence", Note: None;
            Main Area: area;
            Labels: [
                area => "Invalid unicode sequence `{}`": sequence;
            ]
        ]
        InvalidUnicode {
            sequence: String,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Cannot have multiple spread arguments", Note: None;
            Main Area: area;
            Labels: [
                area => "Second spread argument provided here";
                prev_area => "First spread argument provided here";
            ]
        ]
        MultipleSpreadArguments {
            area: CodeArea,
            prev_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Positional argument after keyword argument", Note: None;
            Main Area: area;
            Labels: [
                area => "This positional argument was provided after keyword arguments";
                keyword_area => "First keyword argument provided here";
            ]
        ]
        PositionalArgAfterKeyword {
            area: CodeArea,
            keyword_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Duplicate keyword argument", Note: None;
            Main Area: area;
            Labels: [
                area => "Keyword argument `{}` was provided twice": name;
                prev_area => "Argument previously provided here";
            ]
        ]
        DuplicateKeywordArg {
            name: String,
            area: CodeArea,
            prev_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Duplicate attribute field", Note: None;
            Main Area: used_again;
            Labels: [
                first_used => "Field `{}` first used here": field;
                used_again => "Used again here";
            ]
        ]
        DuplicateAttributeField {
            used_again: CodeArea,
            field: String,
            first_used: CodeArea,
        },

        // // ==================================================================
        // #[
        //     Message: "Invalid number of arguments", Note: None;
        //     Labels: [
        //         area => "Attribute `{}` expected {} arguments, found `{}`": attribute, expected, found;
        //     ]
        // ]
        // InvalidAttributeArgCount {
        //     attribute: String,
        //     expected: usize,
        //     found: usize,

        //     area: CodeArea,
        // },

        // // ==================================================================
        // #[
        //     Message: "Invalid type for attribute", Note: None;
        //     Labels: [
        //         area => "Attribute expected type `{}` as string literal": expected;
        //     ]
        // ]
        // InvalidAttributeArgType {
        //     expected: &'static str,
        //     area: CodeArea,
        // },

        // // ==================================================================
        // #[
        //     Message: "Unknown attribute", Note: Some(format!("The valid attributes are: {}", list_join(valid)));
        //     Labels: [
        //         area => "Attribute `{}` does not exist": attribute;
        //     ]
        // ]
        // UnknownAttribute {
        //     attribute: String,
        //     area: CodeArea,

        //     valid: Vec<String>,
        // },
        // // ==================================================================
        // #[
        //     Message: "Mismatched attribute", Note: None;
        //     Labels: [
        //         area => "Attribute `{}` cannot be added to this element": attr;

        //         expr_area => "{}": =>(match valid {
        //             Some(v) => format!("The valid attributes for this element are: {}", list_join(v)),
        //             None => "This element doesn't support any attributes".into(),
        //         });
        //     ]
        // ]
        // MismatchedAttribute {
        //     area: CodeArea,
        //     expr_area: CodeArea,
        //     attr: String,

        //     valid: Option<Vec<String>>,
        // },

        // // ==================================================================
        // #[
        //     Message: "Invalid attribute field", Note: Some(format!("Valid fields for attribute `{}` are {}", attribute, list_join(fields)));
        //     Labels: [
        //         area => "Unexpected field `{}`": field;
        //     ]
        // ]
        // InvalidAttributeField {
        //     field: String,
        //     area: CodeArea,
        //     attribute: String,
        //     fields: Vec<String>,
        // },

        // ==================================================================
        #[
            Message: "Lexer error", Note: None;
            Main Area: area;
            Labels: [
                area => "{}": =>(err);
            ]
        ]
        LexingError {
            err: LexerError,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Invalid string type used for dictionary key", Note: Some("f-strings and byte strings are not allowed as keys".into());
            Main Area: area;
            Labels: [
                area => "Invalid string here";
            ]
        ]
        InvalidDictStringKey {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unbalanced block in format string", Note: None;
            Main Area: area;
            Labels: [
                area => "Expected `{}`": expected;
            ]
        ]
        UnbalancedFormatStringBlock {
            expected: &'static str,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Invalid `self` argument position", Note: None;
            Main Area: area;
            Labels: [
                area => "Argument is at position {}": pos;
            ]
        ]
        SelfArgumentNotFirst {
            pos: usize,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "`self` argument cannot be spread", Note: None;
            Main Area: area;
            Labels: [
                area => "Spread occurs on this `self`";
            ]
        ]
        SelfArgumentCannotBeSpread {
            area: CodeArea,
        },




        // ==================================================================
        #[
            Message: "Unknown attribute namespace", Note: None;
            Main Area: area;
            Labels: [
                area => "Namespace `{}` does not exist": namespace;
            ]
        ]
        UnknownAttributeNamespace {
            namespace: String,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unknown attribute", Note: None;
            Main Area: area;
            Labels: [
                area => "Attribute `{}` does not exist": attribute;
            ]
        ]
        UnknownAttribute {
            attribute: String,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Mismatched attribute style", Note: Some("`#![...]` in an inner attribute and `#[...]` is an outer attribute".into());
            Main Area: area;
            Labels: [
                area => "Attribute does not exist as an {} attribute": =>(style);
            ]
        ]
        MismatchedAttributeStyle {
            style: AttrStyle,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Duplicate attribute", Note: None;
            Main Area: current_area;
            Labels: [
                old_area => "Attribute `{}` originally specified here": attribute;
                current_area => "Attribute also specified here";
            ]
        ]
        DuplicateAttribute {
            attribute: String,
            current_area: CodeArea,
            old_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "No arguments provided to attribute", Note: Some("A `word` attribute is an attribute without values (E.G. `#[debug_bytecode]`)".into());
            Main Area: attribute_area;
            Labels: [
                attribute_area => "Attribute `{}` expected to take value(s)": attribute;
                attribute_area => "Attribute used as a word here";
            ]
        ]
        NoArgumentsProvidedToAttribute {
            attribute: String,
            attribute_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unknown argument in attribute", Note: None;
            Main Area: attribute_area;
            Labels: [
                attribute_area => "Unknown argument for attribute `{}`": attribute;
                arg_area => "Argument provided here";
            ]
        ]
        UnknownAttributeArgument {
            attribute: String,
            attribute_area: CodeArea,
            arg_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unexpected value for attribute", Note: None;
            Main Area: attribute_area;
            Labels: [
                attribute_area => "Unexpected value provided to attribute `{}`": attribute;
                value_area => "Argument provided here";
            ]
        ]
        UnexpectedValueForAttribute {
            attribute: String,
            attribute_area: CodeArea,
            value_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Missing required arguments for attribute", Note: Some(format!("The missing arguments may be: {}", list_join(missing)));
            Main Area: attribute_area;
            Labels: [
                attribute_area => "Expected {} required arguments for attribute `{}`": expected, attribute;
                args_area => "Found only {} args here": found;
            ]
        ]
        MissingRequiredArgumentsForAttribute {
            attribute: String,
            expected: usize,
            found: usize,
            attribute_area: CodeArea,
            args_area: CodeArea,
            missing: Vec<String>,
        },

        // ==================================================================
        #[
            Message: "Mismatched attribute target", Note: None;
            Main Area: target_area;
            Labels: [
                target_area => "Attribute `{}` cannot be added to this element": attribute;
            ]
        ]
        MismatchedAttributeTarget {
            target_area: CodeArea,
            attribute: String,
        },

        // ==================================================================
        #[
            Message: "Found `mut self`", Note: Some("`mut self` is unlikely the behaviour you want as it will clone `self`. Instead, to make `self` mutable, take a mutable reference: `&self`".into());
            Main Area: area;
            Labels: [
                area => "Found here";
            ]
        ]
        MutSelf {
            area: CodeArea,
        },
    }
}

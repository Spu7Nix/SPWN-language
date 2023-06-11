use std::string::ToString;

use crate::error_maker;
use crate::lexing::tokens::Token;
use crate::sources::CodeArea;

error_maker! {
    Title: "Syntax Error"
    Extra: {}
    pub enum SyntaxError {
        /////////
        #[
            Message: "Unexpected token", Note: None;
            Labels: [
                area => "Expected `{}`, found `{}`": expected, found.to_str();
            ]
        ]
        UnexpectedToken {
            expected: String,
            found: Token,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Unmatched token", Note: None;
            Labels: [
                area => "Couldn't find matching `{}` for this `{}`": not_found.to_str(), for_char.to_str();
            ]
        ]
        UnmatchedToken {
            for_char: Token,
            not_found: Token,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Unexpected character", Note: None;
            Labels: [
                area => "Expected `{}`, found `{}`": expected.to_str(), found;
            ]
        ]
        UnxpectedCharacter {
            expected: Token,
            found: String,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Unexpected string flag", Note: None;
            Labels: [
                area => "Expected valid string flag, found `{}`": flag;
            ]
        ]
        UnexpectedFlag {
            flag: String,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Error parsing escape sequence", Note: None;
            Labels: [
                area => "Unknown escape sequence \\`{}`": character;
            ]
        ]
        InvalidEscape {
            character: char,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Error parsing unicode escape sequence", Note: None;
            Labels: [
                area => "Invalid unicode sequence `{}`": sequence;
            ]
        ]
        InvalidUnicode {
            sequence: String,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Unknown attribute", Note: Some(format!("The valid attributes are: {}", valid.join(", ")));
            Labels: [
                area => "Attribute `{}` does not exist": attribute;
            ]
        ]
        UnknownAttribute {
            attribute: String,
            area: CodeArea,

            valid: Vec<String>,
        },

        /////////
        #[
            Message: "Cannot have multiple spread arguments", Note: None;
            Labels: [
                area => "Second spread argument provided here";
                prev_area => "First spread argument provided here";
            ]
        ]
        MultipleSpreadArguments {
            area: CodeArea,
            prev_area: CodeArea,
        },

        /////////
        #[
            Message: "Positional argument after keyword argument", Note: None;
            Labels: [
                area => "This positional argument was provided after keyword arguments";
                keyword_area => "First keyword argument provided here";
            ]
        ]
        PositionalArgAfterKeyword {
            area: CodeArea,
            keyword_area: CodeArea,
        },

        /////////
        #[
            Message: "Mismatched attribute", Note: None;
            Labels: [
                area => "Attribute `{}` cannot be added to this expression": attr;

                expr_area => "{}": =>(match valid {
                    Some(v) => format!("The valid attributes for this expression are: {}", v.join(", ")),
                    None => "This expression doesn't support any attributes".into(),
                });
            ]
        ]
        MismatchedAttribute {
            area: CodeArea,
            expr_area: CodeArea,
            attr: String,

            valid: Option<Vec<String>>,
        },

        /////////
        #[
            Message: "Invalid attribute field", Note: Some(format!("Valid fields for attribute `{}` are {}", attribute, fields.join(", ")));
            Labels: [
                area => "Unexpected attribute";
            ]
        ]
        InvalidAttributeField {
            area: CodeArea,
            attribute: String,
            fields: Vec<String>,
        },

        /////////
        #[
            Message: "Duplicate attribute field", Note: None;
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

        /////////
        #[
            Message: "Invalid number of arguments", Note: None;
            Labels: [
                area => "Attribute `{}` expected {} arguments": attribute, expected;
            ]
        ]
        InvalidAttributeArgCount {
            attribute: String,
            expected: usize,

            area: CodeArea,
        },

        /////////
        #[
            Message: "Invalid type for attribute", Note: None;
            Labels: [
                area => "Attribute expected `{}`": expected;
            ]
        ]
        InvalidAttributeArgType {
            expected: &'static str,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Invalid string type", Note: None;
            Labels: [
                area => "Expected {} string": typ;
            ]
        ]
        InvalidStringType {
            typ: &'static str,
            area: CodeArea,
        },

        /////////
        #[
            Message: "Catch-all block must be last", Note: None;
            Labels: [
                area => "Catch-all block defined here";
                named_catch_area => "Named catch block defined here, following the catch-all block";
            ]
        ]
        CatchAllNotFinal {
            area: CodeArea,
            named_catch_area: CodeArea,
        },

         /////////
         #[
            Message: "Duplicate catch-all blocks", Note: None;
            Labels: [
                area => "First catch-all defined here";
                second_area => "Next catch-all defined here";
            ]
        ]
        DuplicateCatchAll {
            area: CodeArea,
            second_area: CodeArea,
        },
    }
}

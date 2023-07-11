use std::string::ToString;

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
            Labels: [
                area => "Expected `{}`, found `{}`": expected.to_str(), found;
            ]
        ]
        UnxpectedCharacter {
            expected: Token,
            found: String,
            area: CodeArea,
        },

        // ==================================================================
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

        // ==================================================================
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

        // ==================================================================
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

        // ==================================================================
        #[
            Message: "Unknown attribute", Note: Some(format!("The valid attributes are: {}", list_join(valid)));
            Labels: [
                area => "Attribute `{}` does not exist": attribute;
            ]
        ]
        UnknownAttribute {
            attribute: String,
            area: CodeArea,

            valid: Vec<String>,
        },

        // ==================================================================
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

        // ==================================================================
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

        // ==================================================================
        #[
            Message: "Duplicate keyword argument", Note: None;
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
            Message: "Mismatched attribute", Note: None;
            Labels: [
                area => "Attribute `{}` cannot be added to this element": attr;

                expr_area => "{}": =>(match valid {
                    Some(v) => format!("The valid attributes for this element are: {}", list_join(v)),
                    None => "This element doesn't support any attributes".into(),
                });
            ]
        ]
        MismatchedAttribute {
            area: CodeArea,
            expr_area: CodeArea,
            attr: String,

            valid: Option<Vec<String>>,
        },

        // ==================================================================
        #[
            Message: "Invalid attribute field", Note: Some(format!("Valid fields for attribute `{}` are {}", attribute, list_join(fields)));
            Labels: [
                area => "Unexpected field `{}`": field;
            ]
        ]
        InvalidAttributeField {
            field: String,
            area: CodeArea,
            attribute: String,
            fields: Vec<String>,
        },

        // ==================================================================
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

        // ==================================================================
        #[
            Message: "Invalid number of arguments", Note: None;
            Labels: [
                area => "Attribute `{}` expected {} arguments, found `{}`": attribute, expected, found;
            ]
        ]
        InvalidAttributeArgCount {
            attribute: String,
            expected: usize,
            found: usize,

            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Invalid type for attribute", Note: None;
            Labels: [
                area => "Attribute expected type `{}` as string literal": expected;
            ]
        ]
        InvalidAttributeArgType {
            expected: &'static str,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Lexer error", Note: None;
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
            Labels: [
                area => "Spread occurs on this `self`";
            ]
        ]
        SelfArgumentCannotBeSpread {
            area: CodeArea,
        },
    }
}

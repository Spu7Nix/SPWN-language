use std::string::ToString;

use crate::error_maker;
use crate::sources::CodeArea;

error_maker! {
    Module: syntax_errors;
    pub enum SyntaxError {
        #[
            Message = "Unexpected character", Area = area, Note = None,
            Labels = [
                area => "Expected `{}` found `{}`": @(expected), @(found);
            ]
        ]
        ExpectedToken {
            expected: String,
            found: String,
            area: CodeArea,
        },
        #[
            Message = "Unmatched character", Area = area, Note = None,
            Labels = [
                area => "Couldn't find matching `{}` for this `{}`": @(not_found), @(for_char);
            ]
        ]
        UnmatchedChar {
            for_char: String,
            not_found: String,
            area: CodeArea,
        },
        #[
            Message = "Error parsing escape sequence", Area = area, Note = None,
            Labels = [
                area => "Unknown escape sequence: \\`{}`": @(character);
            ]
        ]
        InvalidEscape {
            character: char,
            area: CodeArea,
        },
        #[
            Message = "Error parsing literal", Area = area, Note = None,
            Labels = [
                area => "Expected valid literal, found: `{}`": @(literal);
            ]
        ]
        InvalidLiteral {
            literal: String,
            area: CodeArea,
        },

        #[
            Message = "Unexpected string flag", Area = area, Note = None,
            Labels = [
                area => "Expected valid string flag, found: `{}`": @(flag);
            ]
        ]
        UnexpectedFlag {
            flag: String,
            area: CodeArea,
        },
    }
}

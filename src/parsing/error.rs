// use miette::{Diagnostic, GraphicalTheme, NamedSource, SourceSpan, ThemeCharacters, ThemeStyles};

use std::string::ToString;
// use thiserror::Error;

use crate::error_maker;
use crate::lexing::tokens::Token;
use crate::sources::CodeArea;

// pub(crate) fn init_miette() {
//     miette::set_hook(Box::new(|_| {
//         Box::new(
//             miette::MietteHandlerOpts::new()
//                 .graphical_theme(GraphicalTheme {
//                     characters: ThemeCharacters::unicode(),
//                     styles: ThemeStyles {
//                         error: Style::new().fg_rgb::<250, 107, 107>().bold(),
//                         warning: Style::new().fg_rgb::<244, 191, 117>(),
//                         advice: Style::new().fg_rgb::<106, 159, 181>(),
//                         help: Style::new().fg_rgb::<106, 159, 181>(),
//                         link: Style::new().fg_rgb::<92, 157, 255>().underline().bold(),
//                         linum: Style::new().fg_rgb::<133, 133, 133>(),
//                         highlights: vec![
//                             Style::new().fg_rgb::<204, 255, 255>(),
//                             Style::new().fg_rgb::<204, 255, 238>(),
//                             Style::new().fg_rgb::<204, 255, 221>(),
//                             Style::new().fg_rgb::<204, 255, 204>(),
//                             Style::new().fg_rgb::<204, 238, 204>(),
//                             Style::new().fg_rgb::<204, 221, 204>(),
//                             Style::new().fg_rgb::<204, 204, 204>(),
//                             Style::new().fg_rgb::<204, 187, 204>(),
//                             Style::new().fg_rgb::<204, 170, 204>(),
//                             Style::new().fg_rgb::<204, 170, 221>(),
//                             Style::new().fg_rgb::<204, 170, 238>(),
//                             Style::new().fg_rgb::<204, 170, 255>(),
//                             Style::new().fg_rgb::<255, 170, 255>(),
//                             Style::new().fg_rgb::<255, 170, 238>(),
//                             Style::new().fg_rgb::<255, 170, 221>(),
//                             Style::new().fg_rgb::<255, 170, 204>(),
//                             Style::new().fg_rgb::<255, 187, 204>(),
//                             Style::new().fg_rgb::<255, 204, 204>(),
//                             Style::new().fg_rgb::<255, 221, 204>(),
//                             Style::new().fg_rgb::<255, 238, 204>(),
//                             Style::new().fg_rgb::<255, 255, 204>(),
//                         ],
//                     },
//                 })
//                 .terminal_links(false)
//                 .context_lines(5)
//                 .tab_width(4)
//                 .build(),
//         )
//     }))
//     .unwrap();
// }

// #[derive(Error, Debug, Diagnostic)]
// enum SyntaxError2 {
//     #[error("Unexpected token")]
//     #[diagnostic(code(parser::unexpected_token))]
//     UnexpectedToken {
//         #[source_code]
//         src: NamedSource,

//         #[label("Expected: `{}`, found: `{}`", expected, found)]
//         area: SourceSpan,

//         expected: String,
//         found: String,
//     },

//     #[error("Unmatched character")]
//     #[diagnostic(code(parser::unmatched_character))]
//     UnmatchedChar {
//         #[source_code]
//         src: NamedSource,

//         #[label("Couldn't find matching: `{}` for this: `{}`", not_found, for_char)]
//         area: SourceSpan,

//         for_char: String,
//         not_found: String,
//     },

//     #[error("Error parsing literal")]
//     #[diagnostic(code(parser::invalid_literal))]
//     InvalidLiteral {
//         #[source_code]
//         src: NamedSource,

//         #[label("Expected valid literal, found: `{}`", literal)]
//         area: SourceSpan,

//         literal: String,
//     },
// }

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
            Message: "Mismatched attribute", Note: Some(help.to_string());
            Labels: [
                area => "Attributes cannot be added to this expression";
            ]
        ]
        MismatchedAttribute {
            area: CodeArea,

            help: String,
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
            Message: "Invalid attribute value", Note: None;
            Labels: [
                area => "{}": message;
            ]
        ]
        InvalidAttributeValue {
            area: CodeArea,
            message: String,
        },
    }
}

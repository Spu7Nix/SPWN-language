use crate::error_maker;
use crate::sources::CodeArea;

error_maker! {
    pub enum SyntaxError {
        #[
            Message = "Unexpected character", Area = area, Note = None,
            Labels = [
                area => "Expected `{}` found {} `{}`": @(expected), @(typ), @(found);
            ]
        ]
        Expected {
            expected: String,
            found: String,
            typ: String,
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
    }
}

// custom wrapper `Result` type as all errors will be syntax errors
pub type Result<T> = std::result::Result<T, SyntaxError>;

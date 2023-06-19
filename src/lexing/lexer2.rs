use unicode_segmentation::{Graphemes, UnicodeSegmentation};

pub struct Lexer2<'a> {
    code: Graphemes<'a>,
}

impl<'a> Lexer2<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            code: code.graphemes(true),
        }
    }
}

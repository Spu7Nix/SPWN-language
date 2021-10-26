use parser::ast::Statement;
use parser::parser::ParseNotes;

use crate::shared::MinifyOptions;

use crate::fmt::fmt;

pub fn minify(statements: Vec<Statement>, notes: ParseNotes, opts: MinifyOptions) -> String {
    let mut minified = fmt(statements, &opts);

    minified = format!("#[{}]", notes.tag.tags.iter().map(|t| t.0.to_owned()).collect::<Vec<_>>().join(",")) + &minified;

    return minified;
}
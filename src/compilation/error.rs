use std::string::ToString;

use crate::error_maker;
use crate::sources::CodeArea;

error_maker! {
    Module: syntax_errors;
    pub enum CompilerError {
        #[
            Message = "Nonexistent variable", Area = area, Note = None,
            Labels = [
                area => "Variable `{}` does not exist": @(name);
            ]
        ]
        NonexistentVariable {
            name: String,
            area: CodeArea,
        },
        #[
            Message = "Attempted to modify immutable variable", Area = area, Note = None,
            Labels = [
                def_area => "Variable `{}` defined as immutable here": @(name);
                area => "Tried to modify here";
            ]
        ]
        ModifyImmutable {
            name: String,
            def_area: CodeArea,
            area: CodeArea,
        },
    }
}

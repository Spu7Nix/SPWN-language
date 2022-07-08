use crate::error_maker;
use crate::sources::CodeArea;

error_maker! {
    Module: compiler_errors;
    pub enum CompilerError {
        #[
            Message = "Nonexistent variable", Area = area, Note = None,
            Labels = [
                area => "Variable `{}` does not exist": @(name);
            ]
        ]
        NonexistentVar {
            name: String,
            area: CodeArea,
        },
        #[
            Message = "Attempted to modify immutable variable", Area = area, Note = None,
            Labels = [
                def_area => "Variable `{}` declared as immutable here": @(name);
                area => "Attempted to modify here";
            ]
        ]
        ModifyImmutable {
            name: String,
            area: CodeArea,
            def_area: CodeArea,
        },
    }
}

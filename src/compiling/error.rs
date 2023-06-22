use std::string::ToString;

use crate::error_maker;
use crate::lexing::tokens::Token;
use crate::parsing::error::SyntaxError;
use crate::sources::CodeArea;
use crate::util::{hyperlink, ImmutStr};

fn list_join<T: std::fmt::Display>(l: &[T]) -> String {
    l.iter()
        .map(|v| format!("`{v}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

error_maker! {
    Title: "Compile Error"
    Extra: {}
    pub enum CompileError {
        // ==================================================================
        #[
            Message: "Nonexistent variable", Note: None;
            Labels: [
                area => "Variable `{}` does not exist": var;
            ]
        ]
        NonexistentVariable {
            area: CodeArea,
            var: ImmutStr,
        },

        // ==================================================================
        #[
            Message: "Tried to modify an immutable variable",
            Note: Some(format!("Use `{}` to define a variable as mutable: `let {var} = ...`", hyperlink("https://spu7nix.net/spwn/#/triggerlanguage/1variables?id=variables", Some("let"))));
            Labels: [
                def_area => "Variable `{}` defined as immutable here": var;
                area => "Tried to modify it here";
            ]
        ]
        ImmutableAssign {
            area: CodeArea,
            def_area: CodeArea,
            var: ImmutStr,
        },

        // ==================================================================
        #[
            Message: "Illegal action inside trigger function", Note: None;
            Labels: [
                def => "Trigger function defined here";
                area => "This is not allowed inside a trigger function";
            ]
        ]
        BreakInTriggerFuncScope {// break/return/continue
            area: CodeArea,
            def: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Illegal action inside arrow statement", Note: None;
            Labels: [
                def => "Arrow statement defined here";
                area => "This is not allowed inside an arrow statement";
            ]
        ]
        BreakInArrowStmtScope { // break/return/continue
            area: CodeArea,
            def: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Return used outside of macro", Note: None;
            Labels: [
                area => "Return used here";
            ]
        ]
        ReturnOutsideMacro {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Invalid module return", Note: None;
            Labels: [
                area => "Module return expects a dictionary value";
            ]
        ]
        InvalidModuleReturn {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Duplicate module return", Note: None;
            Labels: [
                area => "Invalid second module return found here";
                prev_area => "Previous module return used here";
            ]
        ]
        DuplicateModuleReturn {
            area: CodeArea,
            prev_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Break used outside of loop", Note: None;
            Labels: [
                area => "Break used here";
            ]
        ]
        BreakOutsideLoop {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Continue used outside of loop", Note: None;
            Labels: [
                area => "Continue used here";
            ]
        ]
        ContinueOutsideLoop {
            area: CodeArea,
        },
        // ==================================================================
        #[
            Message: "Type definition outside global scope", Note: None;
            Labels: [
                area => "Type definitions can only be used on the top level";
            ]
        ]
        TypeDefNotGlobal {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Type already defined in this scope", Note: None;
            Labels: [
                area => "Duplicate type defined here";
            ]
        ]
        TypeAlreadyDefined {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Import could not be resolved", Note: None;
            Labels: [
                area => "{} `{}` could not be found": => (if *is_file { "File" } else { "Library" }), name;
            ]
        ]
        NonexistentImport {
            is_file: bool,
            name: String,
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Cannot override builtin type", Note: None;
            Labels: [
                area => "Tried to override a builtin type here";
            ]
        ]
        BuiltinTypeOverride {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Syntax error in import", Note: None;
            Labels: [
                area => "Syntax error occured while importing this {}": => (if *is_file { "file" } else { "library" });
                -> err.to_report().labels
            ]
        ]
        ImportSyntaxError {
            is_file: bool,
            err: SyntaxError,
            area: CodeArea,
        },

    }
}

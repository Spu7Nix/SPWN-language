use std::string::ToString;

use crate::error_maker;
use crate::parsing::error::SyntaxError;
use crate::sources::CodeArea;
use crate::util::hyperlink;

error_maker! {
    Title: "Compile Error"
    Extra: {}
    pub enum CompilerError {
        /////
        #[
            Message: "Tried to modify an immutable variable", Note: Some(format!("Use `{}` to define a variable as mutable: `let {var} = ...`", hyperlink("https://spu7nix.net/spwn/#/triggerlanguage/1variables?id=variables", Some("let"))));
            Labels: [
                def_area => "Variable `{}` defined as immutable here": var;
                area => "Tried to modify it here";
            ]
        ]
        ImmutableAssign {
            area: CodeArea,
            def_area: CodeArea,
            var: String,
        },

        /////
        #[
            Message: "Nonexistent variable", Note: None;
            Labels: [
                area => "Variable `{}` does not exist": var;
            ]
        ]
        NonexistentVariable {
            area: CodeArea,
            var: String,
        },

        /////
        #[
            Message: "Break used outside of loop", Note: None;
            Labels: [
                area => "Break used here";
            ]
        ]
        BreakOutsideLoop {
            area: CodeArea,
        },

        /////
        #[
            Message: "Continue used outside of loop", Note: None;
            Labels: [
                area => "Continue used here";
            ]
        ]
        ContinueOutsideLoop {
            area: CodeArea,
        },

        /////
        #[
            Message: "Return used outside of macro", Note: None;
            Labels: [
                area => "Return used here";
            ]
        ]
        ReturnOutsideMacro {
            area: CodeArea,
        },

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

        /////
        #[
            Message: "Invalid module return", Note: None;
            Labels: [
                area => "Module return expects a dictionary value";
            ]
        ]
        InvalidModuleReturn {
            area: CodeArea,
        },

        /////
        #[
            Message: "Type definition outside global scope", Note: None;
            Labels: [
                area => "Type definitions can only be used on the top level";
            ]
        ]
        TypeDefNotGlobal {
            area: CodeArea,
        },

        /////
        #[
            Message: "Duplicate type definition", Note: None;
            Labels: [
                area => "Duplicate type defined here";
                prev_area => "Previously defined here";
            ]
        ]
        DuplicateTypeDef {
            area: CodeArea,
            prev_area: CodeArea,
        },

        /////
        #[
            Message: "Duplicate imported type name", Note: None;
            Labels: [
                area => "This type definition has the same name as a type from within a previous `extract import`";
            ]
        ]
        DuplicateImportedType {
            area: CodeArea,
        },

        /////
        #[
            Message: "Import could not be resolved", Note: None;
            Labels: [
                area => "{} `{}` could not be found": => (if *is_module { "Module" } else { "Library" }), name;
            ]
        ]
        NonexistentImport {
            is_module: bool,
            name: String,
            area: CodeArea,
        },

        /////
        #[
            Message: "Syntax error in import", Note: None;
            Labels: [
                area => "Syntax error occured while importing this {}": => (if *is_module { "module" } else { "library" });
                -> err.to_report().labels
            ]
        ]
        ImportSyntaxError {
            is_module: bool,
            err: SyntaxError,
            area: CodeArea,
        },

        /////
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

        /////
        #[
            Message: "Cannot override builtin type", Note: None;
            Labels: [
                area => "Tried to override a builtin type here";
            ]
        ]
        BuiltinTypeOverride {
            area: CodeArea,
        },

        /////////
        #[
            Message: "Nonexistent type", Note: None;
            Labels: [
                area => "Type {} does not exist or has not been imported and extracted": format!("@{type_name}");
            ]
        ]
        NonexistentType {
            area: CodeArea,
            type_name: String,
        },

        /////////
        #[
            Message: "Invalid overload", Note: None;
            Labels: [
                area => "Overload expected {}": expected;
            ]
        ]
        InvalidOverload {
            expected: String,

            area: CodeArea,
        },
    }
}

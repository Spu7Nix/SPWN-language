use std::string::ToString;

use crate::error_maker;
use crate::lexing::tokens::Token;
use crate::parsing::error::SyntaxError;
use crate::sources::CodeArea;
use crate::util::{hyperlink, ImmutStr};

error_maker! {
    Title: "Compile Error"
    Extra: {}
    pub enum CompileError {
        // ==================================================================
        #[
            Message: "Nonexistent variable", Note: None;
            Main Area: area;
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
            Note: Some(format!("Use `{}` to define a variable as mutable: `mut {var} = ...`", hyperlink("https://spu7nix.net/spwn/#/triggerlanguage/1variables?id=variables", Some("mut"))));
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
            Labels: [
                area => "Type definitions can only be used on the top level";
            ]
        ]
        TypeDefNotGlobal {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Duplicate type definition", Note: None;
            Main Area: area;
            Labels: [
                area => "Duplicate type defined here";
                prev_area => "Previously defined here";
            ]
        ]
        DuplicateTypeDef {
            area: CodeArea,
            prev_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Duplicate imported type name", Note: None;
            Main Area: area;
            Labels: [
                area => "This type definition has the same name as a type from within a previous `extract import`";
            ]
        ]
        DuplicateImportedType {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Import could not be resolved", Note: None;
            Main Area: area;
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
            Main Area: area;
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
            Main Area: area;
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

        // ==================================================================
        #[
            Message: "Nonexistent type", Note: None;
            Main Area: area;
            Labels: [
                area => "Type {} does not exist or has not been imported and extracted": format!("@{type_name}");
            ]
        ]
        NonexistentType {
            area: CodeArea,
            type_name: String,
        },

        // ==================================================================
        #[
            Message: "Illegal pattern for augmented assigment", Note: None;
            Main Area: area;
            Labels: [
                area => "This pattern cannot be assigned to";
            ]
        ]
        IllegalAugmentedAssign {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Illegal pattern for assigment", Note: None;
            Main Area: area;
            Labels: [
                area => "This pattern cannot be assigned to";
            ]
        ]
        IllegalAssign {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "`builtin` attribute used outside of core", Note: None;
            Main Area: area;
            Labels: [
                area => "This attribute is only allowed inside the core";
            ]
        ]
        BuiltinAttrOutsideOfCore {
            area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Invalid type for attribute argument", Note: None;
            Main Area: args_area;
            Labels: [
                args_area => "Expected type `{}`": expected;
            ]
        ]
        InvalidAttributeArgType {
            expected: &'static str,
            args_area: CodeArea,
        },

        // ==================================================================
        #[
            Message: "Unexpected item in overload", Note: Some("Only explicit macro definitions are allowed".into());
            Main Area: area;
            Labels: [
                area => "Found non-macro item here";
            ]
        ]
        UnexpectedItemInOverload {
            area: CodeArea,
        },
    }
}

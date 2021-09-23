use std::path::PathBuf;

pub type StoredValue = u32; //index to stored value in globals.stored_values
pub type FileRange = (usize, usize);

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ImportType {
    Script(PathBuf),
    Lib(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakType {
    // used for return statements
    Macro(Option<StoredValue>, bool),
    // used for Break statements
    Loop,
    // used for continue statements
    ContinueLoop,
    // used for switch cases
    Switch(StoredValue),
    // used for contexts
}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SpwnSource {
    File(PathBuf),
    BuiltIn(PathBuf),
}

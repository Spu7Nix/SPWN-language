use ahash::AHashMap;

use crate::lexing::tokens::Token;
use crate::util::ImmutStr;

struct Deprecated {
    version: semver::Version,
    note: ImmutStr,
}

struct DocData {
    doc: Option<ImmutStr>,
    deprecated: Option<Deprecated>,
}

struct TypeDoc {
    name: ImmutStr,
    doc: DocData,
}
struct VarDoc {
    name: ImmutStr,
    doc: DocData,
}
struct ImplDoc {
    typ: ImmutStr,
    map: AHashMap<ImmutStr, DocData>,
}

struct FileDoc {
    types: Vec<TypeDoc>,
    vars: Vec<VarDoc>,
    impls: Vec<ImplDoc>,
}

use ahash::AHashMap;
use slotmap::{new_key_type, SlotMap};

use super::docgen::Source;

new_key_type! {
    pub struct StmtKey;
}


pub struct DocComment(String);
pub struct DocLine(String);

struct MacroArg {
    name: String,
    typ: Vec<Box<Value>>,
}


#[derive(Default)]
pub struct DocData {
    data: SlotMap<StmtKey, (Vec<DocComment>, DocLine, Source)>,

    // stores every ident found in every file so it can get the source, and subsequently link to it
    // TODO: same ident name in diff files - store source?
    known_idents: AHashMap<String, StmtKey>,
}

// a variable cannot be set to a constant that's defined elsewhere in the file (without using a variable which is the purpose of the `Values` enum)
// therefore these do not need to anchor / redirect on the docs page
pub enum Constant {
    True,
    False,
    String(String),
    Int(i64),
    Float(f64),
    TriggerFunc,
    Block,

    // a value thats unknown (such as `1 + 2` or `(if y { z } else { a })`)
    Unknown,
}

// any values here should be anchors on the docs page / redirect to the values' defitinion
pub enum Value {
    Ident(String),
    TypeIndicator(String),

    Macro {
        name: String,
        args: Vec<MacroArg>,
        // the value can only be a type indicator or macro
        ret: Vec<Box<Value>>,
    }
}

pub enum Values {
    Constant(Constant),
    Value(Value),
}


pub enum Line {
    // module doc comment (very top of file)
    Empty,

    // `x = 20`
    AssociatedConst {
        ident: Value, // `x`
        value: Values, // `20`
    },

    // `type @xyz`
    Type {
        ident: Value, // `@xyx`
    },

    // `impl @xyz { }`
    Impl {
       ident: Value, // `@xyz`
       block: Constant, // `{ ... }`
    },
}


pub type Statements = Vec<StmtKey>;

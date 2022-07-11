use ahash::AHashMap;
use slotmap::{new_key_type, SlotMap};

use super::docgen::Source;

new_key_type! {
    pub struct LineKey;
}

#[derive(Debug)]
pub struct MacroArg {
    pub name: Option<String>,
    pub typ: Option<Values>,
    pub default: Option<Values>,
}

#[derive(Default, Debug)]
pub struct DocData {
    pub data: SlotMap<LineKey, (Vec<String>, Line, Source)>,

    // stores every ident found in every file so it can get the source, and subsequently link to it
    pub known_idents: AHashMap<String, Source>,
}

// a variable cannot be set to a constant that's defined elsewhere in the file (without using a variable which is the purpose of the `Values` enum)
// therefore these do not need to anchor / redirect on the docs page
#[derive(Debug)]
pub enum Constant {
    True,
    False,
    String(String),
    Int(String),
    Float(String),
    TriggerFunc,
    // `()`
    Empty,
    // `{ ... }`
    Block,
    // `obj { }`
    Object,
    // `trigger { }`
    Trigger,

    // a value that's unknown (such as `1 + 2` or `(if y { z } else { a })`)
    Unknown,
}

// any values here should be anchors on the docs page / redirect to the values' defitinion
#[derive(Debug)]
pub enum Value {
    Ident(String),
    TypeIndicator(String),
    Array(Vec<Values>),
    // `?g`, `3c`, `2b`...
    Id((String, String)),

    Macro {
        args: Vec<MacroArg>,
        // the value can only be a type indicator or macro
        ret: Option<Box<Values>>,
    },
}

#[derive(Debug)]
pub enum Values {
    Constant(Constant),
    Value(Value),
}

#[derive(Debug)]
pub enum Line {
    // module doc comment (very top of file)
    Empty,

    // `x = 20`
    AssociatedConst {
        ident: Value,  // `x`
        value: Values, // `20`
    },

    // `type @xyz`
    Type {
        ident: Value, // `@xyx`
    },

    // `impl @xyz { }`
    Impl {
        ident: Value,                   // `@xyz`
        members: Vec<(String, Values)>, // members defined in the impl block
    },
}

pub type Lines = Vec<LineKey>;

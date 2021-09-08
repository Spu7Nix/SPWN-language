mod _lib;
pub use ::parser::ast;
pub mod builtin;
pub mod compiler;
pub use errors::compiler_info;
pub mod compiler_types;
pub mod context;
pub mod documentation;
#[cfg_attr(target_os = "macos", path = "editorlive_mac.rs")]
#[cfg_attr(windows, path = "editorlive_win.rs")]
#[cfg_attr(
    not(any(target_os = "macos", windows)),
    path = "editorlive_unavailable.rs"
)]
pub use ::parser::fmt;
pub mod globals;
pub mod leveldata;
pub use ::parser::parser;
pub use optimize;
pub mod value;
pub mod value_storage;
pub use ::parser::parser::parse_spwn;
pub use _lib::{eprint_with_color, print_with_color, Compiler, STD_PATH};

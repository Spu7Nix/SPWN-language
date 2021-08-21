mod _lib;
pub mod ast;
pub mod builtin;
pub mod compiler;
pub mod compiler_info;
pub mod compiler_types;
pub mod context;
pub mod documentation;
#[cfg_attr(target_os = "macos", path = "editorlive_mac.rs")]
#[cfg_attr(windows, path = "editorlive_win.rs")]
#[cfg_attr(
    not(any(target_os = "macos", windows)),
    path = "editorlive_unavailable.rs"
)]
mod editorlive;
pub mod fmt;
pub mod globals;
pub mod levelstring;
pub mod optimize;
pub mod parser;
pub mod value;
pub mod value_storage;
pub use _lib::{eprint_with_color, print_with_color, Compiler, STD_PATH};
pub use parser::parse_spwn;

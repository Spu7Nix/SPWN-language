#[cfg_attr(target_os = "macos", path = "editorlive_mac.rs")]
#[cfg_attr(windows, path = "editorlive_win.rs")]
#[cfg_attr(
    not(any(target_os = "macos", windows)),
    path = "editorlive_unavailable.rs"
)]
pub mod editorlive;

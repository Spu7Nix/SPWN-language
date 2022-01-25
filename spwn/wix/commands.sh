python spwn/wix/generate_wix_lib_file.py
rustup override unset

rustup default nightly-i686-pc-windows-msvc
cargo install cargo-wix
cargo wix spwn/Cargo.toml --nocapture --package spwn

rustup default nightly-x86_64-pc-windows-msvc
cargo install cargo-wix
cargo wix spwn/Cargo.toml --nocapture --package spwn

rustup override set nightly-x86_64-pc-windows-msvc

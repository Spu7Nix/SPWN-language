use std::{panic, path::Path};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panics() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn run_spwn(code: &str) -> String {
    let output = spwn::run_spwn(code.to_string(), Vec::new());
    let (Err(s) | Ok(s)) = output;
    s
}

// #[wasm_bindgen]
// extern "C" {
//     pub fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet(name: &str) {
//     unsafe {
//         alert(&format!("Hello, {}!", name));
//     }
// }

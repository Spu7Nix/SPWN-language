use std::panic;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panics() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

fn js_array(values: Vec<String>) -> JsValue {
    JsValue::from(
        values
            .into_iter()
            .map(|x| JsValue::from_str(&x))
            .collect::<js_sys::Array>(),
    )
}

#[wasm_bindgen]
pub fn run_spwn(code: &str) -> JsValue {
    let output = spwn::run_spwn(code.to_string(), Vec::new());
    js_array(match output {
        Ok(a) => a.to_vec(),
        Err(e) => vec![e, String::new()],
    })
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

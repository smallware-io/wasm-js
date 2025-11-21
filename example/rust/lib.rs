extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn greet(name: &str) {
    let js_value = JsValue::from_str(&format!("Hello, {}!", name));
    console::log_1(&js_value);
}

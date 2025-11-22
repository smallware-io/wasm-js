# Quickstart

See `example/` for a simple sample Typescript + rust project.

To add a rust library to your Javascript package:

1. Install `rust` using [`rustup`].
2. Install `wasm-js` (build from source or use cargo install)
3. Add a Rust library `Cargo.toml` to your package root:
   ```toml
   [package]
   name = "hello-wasm"
   version = "0.1.0"
   edition = "2024"

   [lib]
   crate-type = ["cdylib"]
   path = "rust/lib.rs"

   [dependencies]
   wasm-bindgen = "0.2"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   console_error_panic_hook = "0.1"

   [dependencies.web-sys]
   version = "0.3"
   features = [
   "console"
   ]
   ```
4. Write your Rust code in `rust/lib.rs`:
   ```rust
   extern crate wasm_bindgen;
   use wasm_bindgen::prelude::*;
   use web_sys::console;

   #[wasm_bindgen]
   pub fn greet(name: &str) {
      let js_value = JsValue::from_str(&format!("Hello, {}!", name));
      console::log_1(&js_value);
   }
   ```
5. Run `wasm-js build` (add this to your package build script). This tool generates
   files in a `dist` directory (by default).  If you're compiling Typescript, you
   should normally use the compiler output directory:
   - `hello_wasm.js` - The Javascript module to import.
   - `hello_wasm.d.ts` - TypeScript definitions
   - `hello_wasm_bg.js` - wasm-bindgen generated import object module
6. Import and use it in your JavaScript:
   ```javascript
   // By going up a level, this import works in the Typescript `src` directory
   // to support, for example, `tsx` AND from the `dist` directory itself, to
   // support the Typescript compiler outputs.
   import {getWasm}  from '../dist/hello_wasm.js';

   getWasm().then((wasm) => {
      wasm.greet('World')
   });
   ```
   
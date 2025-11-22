# Quickstart

1. Install `rust` using [`rustup`].
2. Install `wasm-js` (build from source or use cargo install)
3. Create a new Rust library project: `cargo new --lib hello-wasm`
4. `cd hello-wasm`
5. Add `wasm-bindgen` to your `Cargo.toml`:
   ```toml
   [dependencies]
   wasm-bindgen = "0.2"

   [lib]
   crate-type = ["cdylib"]
   ```
6. Write your Rust code in `src/lib.rs`:
   ```rust
   use wasm_bindgen::prelude::*;

   #[wasm_bindgen]
   pub fn greet(name: &str) {
       web_sys::console::log_1(&format!("Hello, {}!", name).into());
   }
   ```
7. Run `wasm-js build`
8. This tool generates files in a `dist` directory (by default):
   - `hello_wasm.js` - Main JavaScript module with embedded, compressed WASM
   - `hello_wasm_bg.js` - wasm-bindgen generated bindings
   - `hello_wasm.d.ts` - TypeScript definitions
   - `hello_wasm_bg.wasm` - Raw WASM file (for reference)
9. Import and use it in your JavaScript:
   ```javascript
   import { getWasm } from "./dist/hello_wasm.js";

   const wasm = await getWasm();
   wasm.greet("World");
   ```

## Key Differences from wasm-pack

- No `new`, `test`, `pack`, or `publish` commands - `wasm-js` focuses solely on building
- Output directory defaults to `dist` instead of `pkg`
- Generated JavaScript contains compressed, embedded WASM - no separate `.wasm` file needs to be served
- No automatic npm package.json generation or publishing workflow

[`rustup`]: https://rustup.rs/

![wasm ferris](https://drager.github.io/wasm-pack/public/img/wasm-ferris.png)

<h1 style="text-align: center;">Welcome to the <code>wasm-js</code> docs!</h1>

`wasm-js` is a specialized build tool for compiling Rust code to WebAssembly and generating **self-contained JavaScript modules** with embedded WASM. Unlike traditional WASM workflows that require separate `.wasm` files, `wasm-js` produces a single JavaScript file with the compiled WebAssembly compressed and embedded directly within it.

## Key Features

- **JavaScript-Embedded WASM**: The compiled WebAssembly is compressed (using Zlib), base64-encoded, and embedded directly in the generated JavaScript file. No separate `.wasm` files to deploy.
- **Universal Compatibility**: Works seamlessly in both browsers and Node.js environments
- **TypeScript Support**: Automatically generates TypeScript definition files for type-safe integration
- **Build Profiles**: Supports dev, profiling, and release builds with customizable optimization settings
- **wasm-bindgen Integration**: Leverages wasm-bindgen for seamless Rust-JavaScript interoperability

## Origins

`wasm-js` is a fork of [wasm-pack](https://github.com/rustwasm/wasm-pack) that has been streamlined and modified to focus specifically on generating JavaScript-embedded WebAssembly modules. It has been stripped down to support only the `build` command, removing features like npm publishing, testing, and project generation.

[webpack]: https://webpack.js.org/

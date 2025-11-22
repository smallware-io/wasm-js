# Prerequisites

## Rust

Since `wasm-js` is a build tool for Rust projects, you'll need to have [Rust][rust] installed. Make sure `rustc -V` prints out at least version 1.30.0. We recommend using [rustup][rustup] to manage your Rust installation.

[rust]: https://www.rust-lang.org/tools/install
[rustup]: https://rustup.rs/

## wasm32 Target

`wasm-js` compiles your code using the `wasm32-unknown-unknown` target. If you're using rustup, this target will be automatically added when you run `wasm-js build` for the first time. If you want to add it manually, run:

```bash
rustup target add wasm32-unknown-unknown
```

## Installing wasm-js

You can install `wasm-js` by building from source:

```bash
git clone https://github.com/smallware-io/wasm-js.git
cd wasm-js
cargo install --path .
```

Verify the installation by running `wasm-js -V` which should print the installed version.

## Additional Tools

`wasm-js` will automatically download and use the correct version of `wasm-bindgen` for your project. Optionally, you can configure it to use `wasm-opt` for additional optimizations via the `Cargo.toml` configuration.

---

Using a non-rustup setup? Learn how to configure it for wasm-js [here](./non-rustup-setups.html).

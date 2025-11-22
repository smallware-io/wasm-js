# Contributing

## Prerequisites

The technical prerequisites for contributing to this project are the same as for
using it. You can find them documented [here][1].

[1]: ./prerequisites/index.html

## 🏃‍♀️ Up and Running

1. Fork and clone the `smallware-io/wasm-js` repository
2. `cd wasm-js`
3. `cargo run` to build and run the tool. To test command line arguments you can run `cargo run -- <args>`.

## Documentation

Documentation lives in the `/docs` directory. The main documentation files include:
- Introduction and quickstart guides
- Build command documentation
- Cargo.toml configuration reference
- Prerequisites and setup instructions

## Tests

Tests live in the `/tests` directory. To run the tests you can run:

```
cargo test
```

You can also manually test the CLI tool by running:

```
cargo run -- <args>
```

...for example:

```
cargo run -- build --dev
```

## Project Structure

`wasm-js` is a focused fork of wasm-pack that has been streamlined to support only the `build` command with JavaScript-embedded WASM output. Key differences from wasm-pack:

- Removed: npm publishing, testing infrastructure, project generation
- Added: JavaScript embedding functionality via `src/js_bin.rs`
- Modified: Build pipeline to generate compressed, embedded WASM in JavaScript files

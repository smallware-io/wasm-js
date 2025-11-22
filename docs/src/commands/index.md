# Commands

`wasm-js` provides a single command focused on building Rust-generated WebAssembly projects with JavaScript-embedded WASM output.

- `build`: This command compiles your Rust code to WebAssembly and generates JavaScript modules with embedded, compressed WASM. [Learn more][build]

[build]: ./build.html

## Log Levels

By default `wasm-js` displays useful information about the build process.

You can cause it to display even *more* information by using `--verbose`, or you can silence *all* stdout by using `--quiet`.

You can also use `--log-level` to have fine-grained control over wasm-js's log output:

* `--log-level info` is the default, it causes all messages to be logged.
* `--log-level warn` causes warnings and errors to be displayed, but not info.
* `--log-level error` causes only errors to be displayed.

These flags are global flags, and they must come *before* the command:

```sh
wasm-js --log-level error build
wasm-js --quiet build
wasm-js --verbose build
```

## Removed Commands

Unlike its parent project wasm-pack, `wasm-js` has removed the following commands to focus specifically on building JavaScript-embedded WASM:

- `new` - Project generation (use `cargo new --lib` instead)
- `test` - Testing infrastructure
- `pack` and `publish` - npm publishing workflow
- `init` - Deprecated command

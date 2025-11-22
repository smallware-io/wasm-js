# wasm-js build

The `wasm-js build` command compiles your Rust code to WebAssembly and generates JavaScript modules with **embedded, compressed WASM**. Unlike traditional WASM workflows, `wasm-js` produces self-contained JavaScript files where the WebAssembly binary is compressed (using Zlib), base64-encoded, and embedded directly within the JavaScript.

## Output Files

By default, `wasm-js` generates files in a `dist` directory:

- `{name}.js` - Main JavaScript module with embedded, compressed WASM chunks
- `{name}_bg.js` - wasm-bindgen generated bindings and glue code
- `{name}.d.ts` - TypeScript definitions for type-safe integration


## How JavaScript-Embedded WASM Works

1. The WASM binary is compressed using Zlib compression
2. The compressed data is base64-encoded and split into 32KB chunks
3. These chunks are embedded as string literals in the generated JavaScript
4. At runtime, the chunks are decompressed using the browser's native `DecompressionStream` API (or Node.js equivalent)
5. The decompressed WASM is instantiated and initialized automatically

This approach means you only need to deploy JavaScript files - no separate `.wasm` files to serve.

## Path

The `wasm-js build` command can be given an optional path argument, e.g.:

```
wasm-js build examples/js-hello-world
```

This path should point to a directory that contains a `Cargo.toml` file. If no
path is given, the `build` command will run in the current directory.

## Output Directory

By default, `wasm-js` will generate a directory for its build output called `dist`.
If you'd like to customize this you can use the `--out-dir` flag.

```
wasm-js build --out-dir out
```

The above command will put your build artifacts in a directory called `out`, instead
of the default `dist`.

If you are using `tsc` or some other transpiler, then it is often convenient to put
the `wasm-js` outputs in the same directory as your compiler outputs.

## Generated file names

Flag `--out-name` sets the prefix for output file names. If not provided, rust package name is used.

Usage examples, assuming our crate is named `alpha`:

```
wasm-js build
# will produce files
# alpha.d.ts  alpha.js  alpha_bg.js

wasm-js build --out-name index
# will produce files
# index.d.ts  index.js  index_bg.js
```

## Profile

The `build` command accepts an optional profile argument: one of `--dev`,
`--profiling`, or `--release`. If none is supplied, then `--release` is used.

This controls whether debug assertions are enabled, debug info is generated, and
which (if any) optimizations are enabled.

| Profile       | Debug Assertions | Debug Info | Optimizations | Notes                                 |
|---------------|------------------|------------|---------------|---------------------------------------|
| `--dev`       | Yes              | Yes        | No            | Useful for development and debugging. |
| `--profiling` | No               | Yes        | Yes           | Useful when profiling and investigating performance issues. |
| `--release`   | No               | No         | Yes           | Useful for shipping to production.    |

The `--dev` profile will build the output package using cargo's [default
non-release profile][cargo-profile-sections-documentation]. Building this way is
faster but applies few optimizations to the output, and enables debug assertions
and other runtime correctness checks. The `--profiling` and `--release` profiles
use cargo's release profile, but the former enables debug info as well, which
helps when investigating performance issues in a profiler.

The exact meaning of the profile flags may evolve as the platform matures.

[cargo-profile-sections-documentation]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-profile-sections

## Mode

The `build` command accepts an optional `--mode` argument.
```
wasm-js build examples/js-hello-world --mode no-install
```

| Option        | Description                                                                              |
|---------------|------------------------------------------------------------------------------------------|
| `normal`      | Install the correct version of `wasm-bindgen` if necessary.                                    |
| `no-install`  | Rely only on existing `wasm-bindgen`.                 |

## Extra options

The `build` command can pass extra options straight to `cargo build` even if
they are not supported in wasm-js. To use them simply add the extra arguments
at the very end of your command, just as you would for `cargo build`. For
example, to build using cargo's offline feature:

```
wasm-js build examples/js-hello-world --mode no-install -- --offline
```

# wasm-js build

The `wasm-js build` command compiles your Rust code to WebAssembly and generates JavaScript modules with **embedded, compressed WASM**. Unlike traditional WASM workflows, `wasm-js` produces self-contained JavaScript files where the WebAssembly binary is compressed (using Zlib), base64-encoded, and embedded directly within the JavaScript.

## Output Files

By default, `wasm-js` generates files in a `dist` directory:

- `{name}.js` - Main JavaScript module with embedded, compressed WASM chunks
- `{name}_bg.js` - wasm-bindgen generated bindings and glue code
- `{name}.d.ts` - TypeScript definitions for type-safe integration
- `{name}_bg.wasm` - Raw WASM binary (kept for reference, but not needed for runtime)

The `dist` directory is automatically `.gitignore`d by default, since it contains
build artifacts which are not intended to be checked into version control.

## How JavaScript-Embedded WASM Works

1. The WASM binary is compressed using Zlib compression
2. The compressed data is base64-encoded and split into 8KB chunks
3. These chunks are embedded as string literals in the generated JavaScript
4. At runtime, the chunks are decompressed using the browser's native `DecompressionStream` API (or Node.js equivalent)
5. The decompressed WASM is instantiated and initialized automatically

This approach means you only need to deploy JavaScript files - no separate `.wasm` files to serve or configure MIME types for.

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

## Generated file names

Flag `--out-name` sets the prefix for output file names. If not provided, package name is used instead.

Usage examples, assuming our crate is named `dom`:

```
wasm-js build
# will produce files
# dom.d.ts  dom.js  dom_bg.js  dom_bg.wasm

wasm-js build --out-name index
# will produce files
# index.d.ts  index.js  index_bg.js  index_bg.wasm
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

## Target

The `build` command accepts a `--target` argument. This will customize the JS
that is emitted and how the WebAssembly files are instantiated and loaded. For
more documentation on the various strategies here, see the [documentation on
using the compiled output][deploy].

```
wasm-pack build --target nodejs
```

| Option    | Usage | Description                                                                                                     |
|-----------|------------|-----------------------------------------------------------------------------------------------------|
| *not specified* or `bundler` | [Bundler][bundlers] | Outputs JS that is suitable for interoperation with a Bundler like Webpack. You'll `import` the JS and the `module` key is specified in `package.json`. `sideEffects: false` is by default. |
| `nodejs`  | [Node.js][deploy-nodejs] | Outputs JS that uses CommonJS modules, for use with a `require` statement. `main` key in `package.json`. |
| `web` | [Native in browser][deploy-web] | Outputs JS that can be natively imported as an ES module in a browser, but the WebAssembly must be manually instantiated and loaded. |
| `no-modules` | [Native in browser][deploy-web] | Same as `web`, except the JS is included on a page and modifies global state, and doesn't support as many `wasm-bindgen` features as `web` |
| `deno` | [Deno][deploy-deno] | Outputs JS that can be natively imported as an ES module in deno. |

[deploy]: https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html
[bundlers]: https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#bundlers
[deploy-nodejs]: https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#nodejs
[deploy-web]: https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#without-a-bundler
[deploy-deno]: https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html#deno

## Mode

The `build` command accepts an optional `--mode` argument.
```
wasm-js build examples/js-hello-world --mode no-install
```

| Option        | Description                                                                              |
|---------------|------------------------------------------------------------------------------------------|
| `no-install`  | `wasm-js build` creates wasm bindings without installing `wasm-bindgen`.                 |
| `normal`      | Performs full build with `wasm-bindgen` installation.                                    |

## Extra options

The `build` command can pass extra options straight to `cargo build` even if
they are not supported in wasm-js. To use them simply add the extra arguments
at the very end of your command, just as you would for `cargo build`. For
example, to build using cargo's offline feature:

```
wasm-js build examples/js-hello-world --mode no-install -- --offline
```

## Removed Features

Unlike wasm-pack, `wasm-js` does not support:

- **npm scope**: No `--scope` flag, as npm publishing is not supported
- **package.json generation**: No automatic package.json or README copying
- **npm publishing**: No integration with npm registries

If you need to publish to npm, you can manually create a package.json and use standard npm publishing tools with the generated JavaScript files.

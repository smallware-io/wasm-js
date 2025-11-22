<div align="center">

  <h1>wasm-js</h1>

  <p>
    <strong>Build rust libraries into vanilla JS that works everywhere</strong>
  </p>

  <h3>
    <a href="https://github.com/smallware-io/wasm-js">Repository</a>
    <span> | </span>
    <a href="https://github.com/smallware-io/wasm-js/blob/master/CONTRIBUTING.md">Contributing</a>
  </h3>

<sub>Originally forked from <a href="https://github.com/rustwasm/wasm-pack">wasm-pack</a> by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a> and then modified to suit a different purpose.</sub>

</div>

## About

This tool solves one problem.  It builds a rust/web-assembly library into a vanilla javacript module
(esm) that you can *easily* use in your own Javascript/Typescript projects or resusable libraries.

At this moment in history, support for web assembly files and modules across all the various consumers
of Javascript and Typescript is spotty.  Different delivery systems (node, bun, browsers, bundlers) require
different kinds of hoop-jumping to make `.wasm` files work.

For this reason, the output of `wasm-js` does not include any `.wasm` files at all.  It also doesn't use
or require top-level `await`.  Your rust library is compiled into web assembly and processed by `wasm-bindgen`,
and then the web assembly is transformed into plain ol' Javascript that reconstitutes and instantiates
the web assembly.  The resulting module can be loaded by browsers, bundled by all the reasonable bundlers,
transpiled and run directly with `tsx`, or used in NodeJS or (presumbably -- I haven't tried it) Bun.

A `.dt.s` file is also produced to support Typescript.

## When to use `wasm-js`

This is not `wasm-pack`.  It is specifically _not_ a goal of `wasm-js` to turn your rust library directly
into Javascript package that you can publish on [npm](https://www.npmjs.com/).  `wasm-js` does not produce
a `package.json`, or a `README.md`, or a `LICENSE.md` or any of that stuff.  You can easily do that yourself,
and what you put up on `npm` is your _product_.  I'm sure you want to think about it and design it to be
awesome and delightful instead of being limited to whatever I decide to support.

Instead, `wasm-js` makes it easy to make your _own_ Javascript or Typescript project that includes your
own rust/wasm components, and exposes them or leverages them however you like.  Such a package will
include both rust and wasm source trees, and it will have both a `package.json` and a `Cargo.toml`,
although only the Javascript artifacts need to be published.

## Using Your Javascript

The generated Javascript module loads your web assembly asynchronously, but `wasm-js` does not require
top-level `await`.  For this reason, the Javascript module exports an asynchronous function called
`getWasm()` that you call to get a `Promise` for your web assembly interface.

If your project is restricted to environments that _do_ support top-level `await`, then you can wait
at the top level of one of your own modules for this promise to resolve.

## Prerequisites

This project requires Rust 1.30.0 or later and `rustup`

## 🎙️ Commands

At this time, only one command is supported by `wasm-js`:

- `wasm-js build ...`: Compile as rust/wasm crate and transform the output in to Javascript.  See [build docs](docs/build.md)

## 📝 Logging

`wasm-js` uses [`env_logger`] to produce logs when `wasm-js` runs.

To configure your log level, use the `RUST_LOG` environment variable. For example:

```
RUST_LOG=info wasm-js build
```

[`env_logger`]: https://crates.io/crates/env_logger

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Docs

- [Getting Started](docs/getting-started.md)

- [Cargo.toml Configuration](docs/cargo-toml-configuration.md)

- [Build Command](docs/build.md)

- [Prerequisites](docs/prerequisites.md)

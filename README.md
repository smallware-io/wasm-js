<div align="center">

  <h1>📦✨  wasm-js</h1>

  <p>
    <strong>Build rust libraries into vanilla JS that works everywhere</strong>
  </p>

  <p>
    <a href="https://github.com/smallware-io/wasm-js/actions/workflows/test.yml"><img alt="Build Status" src="https://github.com/smallware-io/wasm-js/actions/workflows/test.yml/badge.svg?branch=master"/></a>
  </p>

  <h3>
    <a href="https://github.com/smallware-io/wasm-js">Repository</a>
    <span> | </span>
    <a href="https://github.com/smallware-io/wasm-js/blob/master/CONTRIBUTING.md">Contributing</a>
  </h3>

<sub>Forked from <a href="https://github.com/rustwasm/wasm-pack">wasm-pack</a> by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>

</div>

## About

This tool seeks to be a one-stop shop for building and working with rust-
generated WebAssembly that you would like to interop with JavaScript, in the
browser or with Node.js. `wasm-js` helps you build rust-generated
WebAssembly packages that you could publish to the npm registry, or otherwise use
alongside any javascript packages in workflows that you already use, such as [webpack].

[bundler-support]: https://github.com/rustwasm/team/blob/master/goals/bundler-integration.md#details
[webpack]: https://webpack.js.org/

This project is a fork of [wasm-pack](https://github.com/rustwasm/wasm-pack), originally created by the [rust-wasm] group.

[rust-wasm]: https://github.com/rustwasm/team

![demo](demo.gif)

## 🔮 Prerequisites

This project requires Rust 1.30.0 or later.

## 🎙️ Commands

- `new`: Generate a new RustWasm project using a template
- `build`: Generate an npm wasm pkg from a rustwasm crate
- `test`: Run browser tests
- `pack` and `publish`: Create a tarball of your rustwasm pkg and/or publish to a registry

For detailed documentation, see the original [wasm-pack documentation](https://rustwasm.github.io/docs/wasm-pack/).

## 📝 Logging

`wasm-js` uses [`env_logger`] to produce logs when `wasm-js` runs.

To configure your log level, use the `RUST_LOG` environment variable. For example:

```
RUST_LOG=info wasm-js build
```

[`env_logger`]: https://crates.io/crates/env_logger

## 👯 Contributing

Check out our [contribution policy](CONTRIBUTING.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

This is a fork of [wasm-pack](https://github.com/rustwasm/wasm-pack), originally created by [Ashley Williams](https://github.com/ashleygwilliams) and maintained by the [Rust and WebAssembly Working Group](https://github.com/rustwasm/team).

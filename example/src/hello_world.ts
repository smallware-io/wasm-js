import {getWasm}  from './index.js';

await getWasm().then((wasm) => wasm.greet('World'));

import {getWasm}  from './index.js';

getWasm().then((wasm) => {
  wasm.greet('World')
});

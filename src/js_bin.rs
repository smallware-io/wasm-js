//! Writer the converts a stream of WASM bytes into a JS module

use crate::utils::StrUtils;
use base64::Engine;
use std::io::{self, Write};
use std::sync::LazyLock;

const CHUNK_WORDS: usize = 8192;

pub struct WasmJsWriter<W: Write> {
    out: W,
    imports_module: String,
    wasm_buf: [u8; CHUNK_WORDS * 3],
    out_buf: [u8; CHUNK_WORDS * 4],
    n: usize,
    started: bool,
    finished: bool,
}

static PROLOG: LazyLock<Vec<u8>> = LazyLock::new(|| {
    r#"
const CHUNK_STACK = [
""#
    .to_os_bytes()
});

static CHUNK_SEP: LazyLock<Vec<u8>> = LazyLock::new(|| {
    r#"",
""#
    .to_os_bytes()
});

static EPILOG: LazyLock<Vec<u8>> = LazyLock::new(|| {
    r#""
].reverse();

async function chunkBytes(base64) {
  if (typeof Buffer !== 'undefined') {
    return Buffer.from(base64, 'base64');
  }
  const res = await fetch("data:application/octet-stream;base64," + base64);
  return res.bytes();
}

export const WASM_PROMISE = (async () => {
  const compressed = new ReadableStream({
    type: 'bytes',
    cancel: () => {
      CHUNK_STACK.length = 0;
    },
    pull: async (ctrl) => {
      if (CHUNK_STACK.length) {
        ctrl.enqueue(await chunkBytes(CHUNK_STACK.pop()));
      } else {
        ctrl.close();
      }
    }
  });
  const body = compressed.pipeThrough(new DecompressionStream('deflate'));
  const response = new Response(body,
  {
    status: 200,
      statusText: 'OK',
        headers: {
      'content-type': 'application/wasm'
    }
  });
  const {instance} = await WebAssembly.instantiateStreaming(response, {
    [IMPORTS_KEY]: importObject
  });
  importObject.__wbg_set_wasm(instance.exports);
  instance.exports.__wbindgen_start();
  return importObject;
})();

export function getWasm() {
  return WASM_PROMISE;
}
"#
    .to_os_bytes()
});

impl<W: Write> WasmJsWriter<W> {
    pub fn new(out: W, imports_module: &str) -> Self {
        Self {
            out,
            imports_module: imports_module.to_string(),
            wasm_buf: [0; CHUNK_WORDS * 3],
            out_buf: [0; CHUNK_WORDS * 4],
            n: 0,
            started: false,
            finished: false,
        }
    }

    fn push_chunk(&mut self) -> std::io::Result<()> {
        if self.finished {
            return Err(io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot write to finished WasmJsWriter",
            ));
        }

        let sz = base64::engine::general_purpose::STANDARD
            .encode_slice(&self.wasm_buf[..self.n], &mut self.out_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        if !self.started {
            let opening = format!(
                "import * as importObject from '{}';\nconst IMPORTS_KEY = '{}'\n;",
                self.imports_module, self.imports_module
            );
            self.out.write_all(opening.to_os_bytes().as_ref())?;
            self.out.write_all(PROLOG.as_ref())?;
            self.started = true;
        } else {
            self.out.write_all(CHUNK_SEP.as_ref())?;
        }
        self.out.write_all(&self.out_buf[..sz])?;
        self.n = 0;
        Ok(())
    }
}

impl<W: Write> Write for WasmJsWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        let mut written = 0;
        let mut space = CHUNK_WORDS * 3 - self.n;
        while buf.len() - written > space {
            self.wasm_buf[self.n..(self.n + space)]
                .copy_from_slice(&buf[written..(written + space)]);
            self.n += space;
            written += space;
            self.push_chunk()?;
            space = CHUNK_WORDS * 3;
        }
        let sz = buf.len() - written;
        self.wasm_buf[self.n..(self.n + sz)].copy_from_slice(&buf[written..]);
        self.n += sz;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.push_chunk()?;
        self.finished = true;
        self.out.write_all(EPILOG.as_ref())?;
        self.out.flush()?;
        Ok(())
    }
}

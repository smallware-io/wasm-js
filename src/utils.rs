//! Utility functions for commands.
use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(windows)]
const SYS_LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const SYS_LINE_ENDING: &'static str = "\n";

/// If an explicit path is given, then use it, otherwise assume the current
/// directory is the crate path.
pub fn get_crate_path(path: Option<PathBuf>) -> Result<PathBuf> {
    match path {
        Some(p) => Ok(p),
        None => find_manifest_from_cwd(),
    }
}

/// Search up the path for the manifest file from the current working directory
/// If we don't find the manifest file then return back the current working directory
/// to provide the appropriate error
fn find_manifest_from_cwd() -> Result<PathBuf> {
    let mut parent_path = std::env::current_dir()?;
    let mut manifest_path = parent_path.join("Cargo.toml");
    loop {
        if !manifest_path.is_file() {
            if parent_path.pop() {
                manifest_path = parent_path.join("Cargo.toml");
            } else {
                return Ok(PathBuf::from("."));
            }
        } else {
            return Ok(parent_path);
        }
    }
}

/// Construct our `dist` directory in the crate.
pub fn create_output_dir(out_dir: &Path) -> Result<()> {
    fs::create_dir_all(&out_dir)?;
    Ok(())
}

/// Get wasm-pack's binary cache.
pub fn get_install_cache(spec: &Option<String>) -> Result<binary_install::Cache> {
    if let Some(path) = spec {
        Ok(binary_install::Cache::at(Path::new(&path)))
    } else {
        binary_install::Cache::new("wasm-js")
    }
}

/// Render a `Duration` to a form suitable for display on a console
pub fn elapsed(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs >= 60 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        format!("{}.{:02}s", secs, duration.subsec_nanos() / 10_000_000)
    }
}

/// Reads a file from `input_path` and returns its contents compressed using DEFLATE
/// as an in-memory vector of bytes (`Vec<u8>`).
pub fn read_and_compress<W: Write>(out: W, input_path: &Path) -> Result<()> {
    let mut encoder = ZlibEncoder::new(out, Compression::best());
    let mut input_file = File::open(input_path)?;
    io::copy(&mut input_file, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

pub trait StrUtils {
    fn to_os_bytes(self: &Self) -> Vec<u8>;
}

impl StrUtils for str {
    fn to_os_bytes(self: &Self) -> Vec<u8> {
        self.replace("\r\n", "\n")
            .replace("\r", "\n")
            .replace("\n", SYS_LINE_ENDING)
            .as_bytes()
            .to_vec()
    }
}

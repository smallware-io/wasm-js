extern crate anyhow;
extern crate cargo_metadata;
extern crate console;
extern crate glob;
extern crate parking_lot;
extern crate serde;
extern crate strsim;
extern crate which;
#[macro_use]
extern crate serde_derive;
extern crate binary_install;
extern crate dialoguer;
extern crate log;
extern crate serde_ignored;
extern crate serde_json;
extern crate toml;
extern crate walkdir;

pub mod bindgen;
pub mod build;
pub mod child;
pub mod command;
pub mod install;
pub mod js_bin;
pub mod lockfile;
pub mod manifest;
pub mod progressbar;
pub mod stamps;
pub mod target;
pub mod test;
pub mod utils;
pub mod wasm_opt;

use crate::progressbar::{LogLevel, ProgressOutput};
use clap::builder::ArgAction;
use clap::Parser;

/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();

#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    /// The subcommand to run.
    #[clap(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: command::Command,

    /// Log verbosity is based off the number of v used
    #[clap(long = "verbose", short = 'v', action = ArgAction::Count)]
    pub verbosity: u8,

    #[clap(long = "quiet", short = 'q')]
    /// No output printed to stdout
    pub quiet: bool,

    #[clap(long = "log-level", default_value = "info")]
    /// The maximum level of messages that should be logged by
    pub log_level: LogLevel,

    #[clap(long = "install-cache")]
    /// Sets the location of the binary install cache
    pub install_cache: Option<String>,
}

impl Cli {
    pub fn from_command(cmd: command::Command) -> Self {
        Self {
            cmd,
            verbosity: 0,
            quiet: false,
            log_level: LogLevel::Info,
            install_cache: None,
        }
    }
}

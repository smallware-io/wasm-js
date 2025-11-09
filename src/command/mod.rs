//! CLI command structures, parsing, and execution.
pub mod build;
pub mod utils;

use self::build::{Build, BuildOptions};
use anyhow::Result;
use clap::Subcommand;
use log::info;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 🏗️  build your npm package!
    #[clap(name = "build")]
    Build(BuildOptions),
}

/// Run a command with the given logger!
pub fn run_wasm_pack(command: Command) -> Result<()> {
    // Run the correct command based off input and store the result of it so that we can clear
    // the progress bar then return it
    match command {
        Command::Build(build_opts) => {
            info!("Running build command...");
            Build::try_from_opts(build_opts).and_then(|mut b| b.run())
        }
    }
}

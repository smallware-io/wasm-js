extern crate anyhow;
extern crate clap;
extern crate env_logger;
extern crate human_panic;
extern crate log;
extern crate which;

use anyhow::Result;
use clap::Parser;
use std::env;
use std::panic;
use wasm_js::command::run_command;
use wasm_js::Cli;
use wasm_js::PBAR;

fn main() {
    env_logger::init();

    setup_panic_hooks();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.chain() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();

    PBAR.set_log_level(args.log_level);

    if args.quiet {
        PBAR.set_quiet(true);
    }

    run_command(&args)?;

    Ok(())
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if let Err(_) = env::var("RUST_BACKTRACE") {
        panic::set_hook(Box::new(move |info: &panic::PanicHookInfo| {
            // First call the default hook that prints to standard error.
            default_hook(info);

            // Then call human_panic.
            let file_path = human_panic::handle_dump(&meta, info);
            human_panic::print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }));
    }
}

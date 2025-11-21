extern crate anyhow;
extern crate assert_cmd;
extern crate binary_install;
extern crate lazy_static;
extern crate predicates;
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serial_test;
extern crate clap;
extern crate tempfile;

mod build;
mod log_level;
mod stamps;
mod utils;
mod wasm_opt;

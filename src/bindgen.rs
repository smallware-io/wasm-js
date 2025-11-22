//! Functionality related to running `wasm-bindgen`.

use crate::child;
use crate::command::build::BuildProfile;
use crate::install::{self, Tool};
use crate::manifest::CrateData;
use anyhow::{Context, Result};
use semver;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    install_status: &install::Status,
    out_name: &Option<String>,
    weak_refs: bool,
    reference_types: bool,
    profile: BuildProfile,
    extra_options: &Vec<String>,
) -> Result<PathBuf> {
    let profile_name = match profile.clone() {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
        BuildProfile::Custom(profile_name) => &profile_name.clone(),
    };

    let target_directory = {
        let mut has_target_dir_iter = extra_options.iter();
        has_target_dir_iter
            .find(|&it| it == "--target-dir")
            .and_then(|_| has_target_dir_iter.next())
            .map(Path::new)
            .unwrap_or(data.target_directory())
    };

    let wasm_path = target_directory
        .join("wasm32-unknown-unknown")
        .join(profile_name)
        .join(data.crate_name())
        .with_extension("wasm");

    let out_dir_path = target_directory.join("wasm-bindgen");
    let out_dir = out_dir_path.to_str().unwrap();

    let dts_arg = "--typescript";
    let bindgen_path = install::get_tool_path(install_status, Tool::WasmBindgen)?
        .binary(&Tool::WasmBindgen.to_string())?;

    let mut cmd = Command::new(&bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg);

    if weak_refs {
        cmd.arg("--weak-refs");
    }

    if reference_types {
        cmd.arg("--reference-types");
    }

    let target_arg = build_target_arg(&bindgen_path)?;
    if supports_dash_dash_target(&bindgen_path)? {
        cmd.arg("--target").arg(target_arg);
    } else {
        cmd.arg(target_arg);
    }

    if let Some(value) = out_name {
        cmd.arg("--out-name").arg(value);
    }

    let profile = data.configured_profile(profile);
    if profile.wasm_bindgen_debug_js_glue() {
        cmd.arg("--debug");
    }
    if !profile.wasm_bindgen_demangle_name_section() {
        cmd.arg("--no-demangle");
    }
    if profile.wasm_bindgen_dwarf_debug_info() {
        cmd.arg("--keep-debug");
    }
    if profile.wasm_bindgen_omit_default_module_path() {
        cmd.arg("--omit-default-module-path");
    }
    if profile.wasm_bindgen_split_linked_modules() {
        cmd.arg("--split-linked-modules");
    }

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(out_dir_path)
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the --target flag
fn supports_dash_dash_target(cli_path: &Path) -> Result<bool> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.40")?;
    Ok(cli_version >= expected_version)
}

fn build_target_arg(cli_path: &Path) -> Result<String> {
    if !supports_dash_dash_target(cli_path)? {
        Ok("--browser".to_string())
    } else {
        Ok("bundler".to_string())
    }
}

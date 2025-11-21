use crate::bindgen;
use crate::build;
use crate::install::{self, InstallMode, Tool};
use crate::js_bin::WasmJsWriter;
use crate::lockfile::Lockfile;
use crate::manifest;
use crate::utils::*;
use crate::wasm_opt;
use crate::PBAR;

use anyhow::{anyhow, bail, Result};
use binary_install::Cache;
use clap::Args;
use log::info;
use path_clean::PathClean;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

/// Everything required to configure and run the `wasm-js build` command.
pub struct Build {
    pub crate_path: PathBuf,
    pub crate_data: manifest::CrateData,
    pub scope: Option<String>,
    pub disable_dts: bool,
    pub weak_refs: bool,
    pub reference_types: bool,
    pub no_opt: bool,
    pub profile: BuildProfile,
    pub mode: InstallMode,
    pub out_dir: PathBuf,
    pub out_name: Option<String>,
    pub bindgen: Option<install::Status>,
    pub cache: Cache,
    pub extra_options: Vec<String>,
}

/// The build profile controls whether optimizations, debug info, and assertions
/// are enabled or disabled.
#[derive(Clone, Debug)]
pub enum BuildProfile {
    /// Enable assertions and debug info. Disable optimizations.
    Dev,
    /// Enable optimizations. Disable assertions and debug info.
    Release,
    /// Enable optimizations and debug info. Disable assertions.
    Profiling,
    /// User-defined profile with --profile flag
    Custom(String),
}

/// Everything required to configure and run the build command.
#[derive(Debug, Args)]
#[command(allow_hyphen_values = true, trailing_var_arg = true)]
pub struct BuildOptions {
    /// The path to the Rust crate. If not set, searches up the path from the current directory.
    #[clap()]
    pub path: Option<PathBuf>,

    /// The npm scope to use in package.json, if any.
    #[clap(long = "scope", short = 's')]
    pub scope: Option<String>,

    #[clap(long = "mode", short = 'm', default_value = "normal")]
    /// Sets steps to be run. [possible values: no-install, normal, force]
    pub mode: InstallMode,

    #[clap(long = "no-typescript")]
    /// By default a *.d.ts file is generated for the generated JS file, but
    /// this flag will disable generating this TypeScript file.
    pub disable_dts: bool,

    #[clap(long = "weak-refs")]
    /// Enable usage of the JS weak references proposal.
    pub weak_refs: bool,

    #[clap(long = "reference-types")]
    /// Enable usage of WebAssembly reference types.
    pub reference_types: bool,

    #[clap(long = "debug")]
    /// Deprecated. Renamed to `--dev`.
    pub debug: bool,

    #[clap(long = "dev")]
    /// Create a development build. Enable debug info, and disable
    /// optimizations.
    pub dev: bool,

    #[clap(long = "release")]
    /// Create a release build. Enable optimizations and disable debug info.
    pub release: bool,

    #[clap(long = "profiling")]
    /// Create a profiling build. Enable optimizations and debug info.
    pub profiling: bool,

    #[clap(long = "profile")]
    /// User-defined profile with --profile flag
    pub profile: Option<String>,

    #[clap(long = "out-dir", short = 'd', default_value = "dist")]
    /// Sets the output directory with a relative path.
    pub out_dir: String,

    #[clap(long = "out-name")]
    /// Sets the output file names. Defaults to package name.
    pub out_name: Option<String>,

    #[clap(long = "no-opt", alias = "no-optimization")]
    /// Option to skip optimization with wasm-opt
    pub no_opt: bool,

    /// List of extra options to pass to `cargo build`
    pub extra_options: Vec<String>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            path: None,
            scope: None,
            mode: InstallMode::default(),
            disable_dts: false,
            weak_refs: false,
            reference_types: false,
            debug: false,
            dev: false,
            no_opt: false,
            release: false,
            profiling: false,
            profile: None,
            out_dir: String::new(),
            out_name: None,
            extra_options: Vec::new(),
        }
    }
}

impl Build {
    /// Construct a build command from the given options.
    pub fn try_from_opts(args: &crate::Cli, build_opts: &BuildOptions) -> Result<Self> {
        let mut extra_options: Vec<String> = build_opts.extra_options.clone();
        let mut path_arg = build_opts.path.clone();
        if let Some(path) = &build_opts.path {
            if path.to_string_lossy().starts_with("--") {
                let path = path_arg.take().unwrap();
                extra_options.insert(0, path.to_string_lossy().into_owned());
            }
        }
        let crate_path = get_crate_path(path_arg)?;
        let crate_data = manifest::CrateData::new(&crate_path, build_opts.out_name.clone())?;
        let out_dir = crate_path.join(PathBuf::from(&build_opts.out_dir)).clean();

        let dev = build_opts.dev || build_opts.debug;
        let profile = match (
            dev,
            build_opts.release,
            build_opts.profiling,
            &build_opts.profile,
        ) {
            (false, false, false, None) | (false, true, false, None) => BuildProfile::Release,
            (true, false, false, None) => BuildProfile::Dev,
            (false, false, true, None) => BuildProfile::Profiling,
            (false, false, false, Some(profile)) => BuildProfile::Custom(profile.clone()),
            // Unfortunately, `clap` doesn't expose clap's `conflicts_with`
            // functionality yet, so we have to implement it ourselves.
            _ => bail!("Can only supply one of the --dev, --release, --profiling, or --profile 'name' flags"),
        };

        Ok(Build {
            crate_path,
            crate_data,
            scope: build_opts.scope.clone(),
            disable_dts: build_opts.disable_dts,
            weak_refs: build_opts.weak_refs,
            reference_types: build_opts.reference_types,
            no_opt: build_opts.no_opt,
            profile,
            mode: build_opts.mode,
            out_dir,
            out_name: build_opts.out_name.clone(),
            bindgen: None,
            cache: get_install_cache(&args.install_cache)?,
            extra_options: extra_options,
        })
    }

    /// Configures the global binary cache used for this build
    pub fn set_cache(&mut self, cache: Cache) {
        self.cache = cache;
    }

    /// Execute this `Build` command.
    pub fn run(&mut self) -> Result<()> {
        let started = Instant::now();

        if self.mode != InstallMode::Force {
            self.step_check_rustc_version()?;
            self.step_check_crate_config()?;
            self.step_check_for_wasm_target()?;
        }

        self.step_build_wasm()?;
        self.step_create_dir()?;
        self.step_install_wasm_bindgen()?;
        let temp_dir = self.step_run_wasm_bindgen()?;

        if !self.no_opt {
            self.step_run_wasm_opt()?;
        }
        self.step_transform_wasm(&temp_dir)?;

        let duration = elapsed(started.elapsed());
        info!("Done in {}.", &duration);
        info!("Javascript files created in {}.", self.out_dir.display());

        PBAR.info(&format!("Done in {}", &duration));

        PBAR.info(&format!(
            "Javascript files created in {}.",
            self.out_dir.display()
        ));
        Ok(())
    }

    fn step_check_rustc_version(&mut self) -> Result<()> {
        info!("Checking rustc version...");
        let version = build::check_rustc_version()?;
        let msg = format!("rustc version is {}.", version);
        info!("{}", &msg);
        Ok(())
    }

    fn step_check_crate_config(&mut self) -> Result<()> {
        info!("Checking crate configuration...");
        self.crate_data.check_crate_config()?;
        info!("Crate is correctly configured.");
        Ok(())
    }

    fn step_check_for_wasm_target(&mut self) -> Result<()> {
        info!("Checking for wasm-target...");
        build::wasm_target::check_for_wasm32_target()?;
        info!("Checking for wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self) -> Result<()> {
        info!("Building wasm...");
        build::cargo_build_wasm(&self.crate_path, self.profile.clone(), &self.extra_options)?;

        info!(
            "wasm built at {:#?}.",
            &self
                .crate_path
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
        );
        Ok(())
    }

    fn step_create_dir(&mut self) -> Result<()> {
        info!("Creating a dist directory...");
        create_output_dir(&self.out_dir)?;
        info!("Created a dist directory at {:#?}.", &self.crate_path);
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self) -> Result<()> {
        info!("Identifying wasm-bindgen dependency...");
        let lockfile = Lockfile::new(&self.crate_data)?;
        let bindgen_version = lockfile.require_wasm_bindgen()?;
        info!("Installing wasm-bindgen-cli...");
        let bindgen = install::download_prebuilt_or_cargo_install(
            Tool::WasmBindgen,
            &self.cache,
            bindgen_version,
            self.mode.install_permitted(),
        )?;
        self.bindgen = Some(bindgen);
        info!("Installing wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_run_wasm_bindgen(&mut self) -> Result<PathBuf> {
        info!("Building the wasm bindings...");
        let temp_dir = bindgen::wasm_bindgen_build(
            &self.crate_data,
            self.bindgen.as_ref().unwrap(),
            &self.out_name,
            self.disable_dts,
            self.weak_refs,
            self.reference_types,
            self.profile.clone(),
            &self.extra_options,
        )?;
        info!("wasm bindings were built at {:#?}.", &temp_dir);
        Ok(temp_dir)
    }

    fn step_run_wasm_opt(&mut self) -> Result<()> {
        let mut args = match self
            .crate_data
            .configured_profile(self.profile.clone())
            .wasm_opt_args()
        {
            Some(args) => args,
            None => return Ok(()),
        };
        if self.reference_types {
            args.push("--enable-reference-types".into());
        }
        info!("executing wasm-opt with {:?}", args);
        wasm_opt::run(
            &self.cache,
            &self.out_dir,
            &args,
            self.mode.install_permitted(),
        ).map_err(|e| {
            anyhow!(
                "{}\nTo disable `wasm-opt`, add `wasm-opt = false` to your package metadata in your `Cargo.toml`.", e
            )
        })
    }

    fn step_transform_wasm(&mut self, temp_dir: &Path) -> Result<()> {
        let name_prefix = self.crate_data.name_prefix();
        let wasm_filename = format!("{}_bg.wasm", name_prefix);
        let imports_filename = format!("{}_bg.js", name_prefix);
        let types_filename = format!("{}.d.ts", name_prefix);
        let module_filename = format!("{}.js", name_prefix);
        let imports_module = format!("./{}", imports_filename);
        // convert wasm to JS
        {
            let mut outfile = File::create(self.out_dir.join(module_filename))?;
            {
                let mut outbw = BufWriter::new(&mut outfile);
                let mut wasm_writer = WasmJsWriter::new(&mut outbw, &imports_module);
                let input_path = temp_dir.join(wasm_filename);
                read_and_deflate(&mut wasm_writer, &input_path)?;
                wasm_writer.flush()?;
            }
            outfile.sync_all()?;
        }
        fs::copy(
            temp_dir.join(&imports_filename),
            self.out_dir.join(&imports_filename),
        )?;
        // transform types file
        {
            let types_text = fs::read(temp_dir.join(&types_filename))?;
            let mut outfile = File::create(self.out_dir.join(&types_filename))?;
            {
                let mut outbw = BufWriter::new(&mut outfile);
                outbw.write_all(
                    "/* tslint:disable */\n/* eslint-disable */\ndeclare namespace WasmDecls {\n"
                        .to_os_bytes()
                        .as_ref(),
                )?;
                outbw.write_all(&types_text)?;
                outbw.write_all(
                    "\n}\nexport type WasmExports = typeof WasmDecls;\nexport function getWasm(): Promise<typeof WasmExports>;\n"
                        .to_os_bytes()
                        .as_ref(),
                )?;
                outbw.flush()?;
            }
            outfile.sync_all()?;
        }

        for file in self.out_dir.read_dir()? {
            let file = file?;
            let path = file.path();
            let extension = path.extension().and_then(|s| s.to_str());
            if extension == Some("wasm") {}
        }

        Ok(())
    }
}

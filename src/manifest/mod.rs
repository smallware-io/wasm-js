//! Reading and writing Cargo.toml and package.json manifests.
use anyhow::{anyhow, bail, Context, Result};

use std::fs;
use std::path::Path;

use crate::command::build::BuildProfile;
use crate::PBAR;
use cargo_metadata::Metadata;
use serde::{self, Deserialize};
use std::collections::BTreeSet;
use strsim::levenshtein;
use toml;

const CARGO_METADATA_KEY: &str = "package.metadata.wasm-js";

/// Store for metadata learned about a crate
pub struct CrateData {
    data: Metadata,
    current_idx: usize,
    manifest: CargoManifest,
    out_name: Option<String>,
}

#[doc(hidden)]
#[derive(Deserialize)]
pub struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,

    #[serde(default)]
    metadata: CargoMetadata,
}

#[derive(Default, Deserialize)]
struct CargoMetadata {
    #[serde(default, rename = "wasm-js")]
    wasm_js: CargoWasmPack,
}

#[derive(Default, Deserialize)]
struct CargoWasmPack {
    #[serde(default)]
    profile: CargoWasmPackProfiles,
}

#[derive(Deserialize)]
struct CargoWasmPackProfiles {
    #[serde(
        default = "CargoWasmPackProfile::default_dev",
        deserialize_with = "CargoWasmPackProfile::deserialize_dev"
    )]
    dev: CargoWasmPackProfile,

    #[serde(
        default = "CargoWasmPackProfile::default_release",
        deserialize_with = "CargoWasmPackProfile::deserialize_release"
    )]
    release: CargoWasmPackProfile,

    #[serde(
        default = "CargoWasmPackProfile::default_profiling",
        deserialize_with = "CargoWasmPackProfile::deserialize_profiling"
    )]
    profiling: CargoWasmPackProfile,

    #[serde(
        default = "CargoWasmPackProfile::default_custom",
        deserialize_with = "CargoWasmPackProfile::deserialize_custom"
    )]
    custom: CargoWasmPackProfile,
}

impl Default for CargoWasmPackProfiles {
    fn default() -> CargoWasmPackProfiles {
        CargoWasmPackProfiles {
            dev: CargoWasmPackProfile::default_dev(),
            release: CargoWasmPackProfile::default_release(),
            profiling: CargoWasmPackProfile::default_profiling(),
            custom: CargoWasmPackProfile::default_custom(),
        }
    }
}

/// This is where configuration goes for wasm-bindgen, wasm-opt, wasm-snip, or
/// anything else that we run.
#[derive(Default, Deserialize)]
pub struct CargoWasmPackProfile {
    #[serde(default, rename = "wasm-bindgen")]
    wasm_bindgen: CargoWasmPackProfileWasmBindgen,
    #[serde(default, rename = "wasm-opt")]
    wasm_opt: Option<CargoWasmPackProfileWasmOpt>,
}

#[derive(Default, Deserialize)]
struct CargoWasmPackProfileWasmBindgen {
    #[serde(default, rename = "debug-js-glue")]
    debug_js_glue: Option<bool>,

    #[serde(default, rename = "demangle-name-section")]
    demangle_name_section: Option<bool>,

    #[serde(default, rename = "dwarf-debug-info")]
    dwarf_debug_info: Option<bool>,

    #[serde(default, rename = "omit-default-module-path")]
    omit_default_module_path: Option<bool>,

    #[serde(default, rename = "split-linked-modules")]
    split_linked_modules: Option<bool>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum CargoWasmPackProfileWasmOpt {
    Enabled(bool),
    ExplicitArgs(Vec<String>),
}

impl Default for CargoWasmPackProfileWasmOpt {
    fn default() -> Self {
        CargoWasmPackProfileWasmOpt::Enabled(false)
    }
}

impl CargoWasmPackProfile {
    fn default_dev() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(true),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
                omit_default_module_path: Some(false),
                split_linked_modules: Some(false),
            },
            wasm_opt: None,
        }
    }

    fn default_release() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(false),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
                omit_default_module_path: Some(false),
                split_linked_modules: Some(false),
            },
            wasm_opt: Some(CargoWasmPackProfileWasmOpt::Enabled(true)),
        }
    }

    fn default_profiling() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(false),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
                omit_default_module_path: Some(false),
                split_linked_modules: Some(false),
            },
            wasm_opt: Some(CargoWasmPackProfileWasmOpt::Enabled(true)),
        }
    }

    fn default_custom() -> Self {
        CargoWasmPackProfile {
            wasm_bindgen: CargoWasmPackProfileWasmBindgen {
                debug_js_glue: Some(false),
                demangle_name_section: Some(true),
                dwarf_debug_info: Some(false),
                omit_default_module_path: Some(false),
                split_linked_modules: Some(false),
            },
            wasm_opt: Some(CargoWasmPackProfileWasmOpt::Enabled(true)),
        }
    }

    fn deserialize_dev<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_dev());
        Ok(profile)
    }

    fn deserialize_release<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_release());
        Ok(profile)
    }

    fn deserialize_profiling<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_profiling());
        Ok(profile)
    }

    fn deserialize_custom<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut profile = <Option<Self>>::deserialize(deserializer)?.unwrap_or_default();
        profile.update_with_defaults(&Self::default_custom());
        Ok(profile)
    }

    fn update_with_defaults(&mut self, defaults: &Self) {
        macro_rules! d {
            ( $( $path:ident ).* ) => {
                self. $( $path ).* .get_or_insert(defaults. $( $path ).* .unwrap());
            }
        }
        d!(wasm_bindgen.debug_js_glue);
        d!(wasm_bindgen.demangle_name_section);
        d!(wasm_bindgen.dwarf_debug_info);
        d!(wasm_bindgen.omit_default_module_path);
        d!(wasm_bindgen.split_linked_modules);

        if self.wasm_opt.is_none() {
            self.wasm_opt = defaults.wasm_opt.clone();
        }
    }

    /// Get this profile's configured `[wasm-bindgen.debug-js-glue]` value.
    pub fn wasm_bindgen_debug_js_glue(&self) -> bool {
        self.wasm_bindgen.debug_js_glue.unwrap()
    }

    /// Get this profile's configured `[wasm-bindgen.demangle-name-section]` value.
    pub fn wasm_bindgen_demangle_name_section(&self) -> bool {
        self.wasm_bindgen.demangle_name_section.unwrap()
    }

    /// Get this profile's configured `[wasm-bindgen.dwarf-debug-info]` value.
    pub fn wasm_bindgen_dwarf_debug_info(&self) -> bool {
        self.wasm_bindgen.dwarf_debug_info.unwrap()
    }

    /// Get this profile's configured `[wasm-bindgen.omit-default-module-path]` value.
    pub fn wasm_bindgen_omit_default_module_path(&self) -> bool {
        self.wasm_bindgen.omit_default_module_path.unwrap()
    }

    /// Get this profile's configured `[wasm-bindgen.split-linked-modules]` value.
    pub fn wasm_bindgen_split_linked_modules(&self) -> bool {
        self.wasm_bindgen.split_linked_modules.unwrap()
    }

    /// Get this profile's configured arguments for `wasm-opt`, if enabled.
    pub fn wasm_opt_args(&self) -> Option<Vec<String>> {
        match self.wasm_opt.as_ref()? {
            CargoWasmPackProfileWasmOpt::Enabled(false) => None,
            CargoWasmPackProfileWasmOpt::Enabled(true) => Some(vec!["-O".to_string()]),
            CargoWasmPackProfileWasmOpt::ExplicitArgs(s) => Some(s.clone()),
        }
    }
}

#[doc(hidden)]
pub struct ManifestAndUnsedKeys {
    pub manifest: CargoManifest,
    pub unused_keys: BTreeSet<String>,
}

impl CrateData {
    /// Reads all metadata for the crate whose manifest is inside the directory
    /// specified by `path`.
    pub fn new(crate_path: &Path, out_name: Option<String>) -> Result<CrateData> {
        let manifest_path = crate_path.join("Cargo.toml");
        if !manifest_path.is_file() {
            bail!(
                "crate directory is missing a `Cargo.toml` file; is `{}` the \
                 wrong directory?",
                crate_path.display()
            )
        }

        let data = cargo_metadata::MetadataCommand::new()
            .manifest_path(&manifest_path)
            .exec()?;

        let manifest_and_keys = CrateData::parse_crate_data(&manifest_path)?;
        CrateData::warn_for_unused_keys(&manifest_and_keys);

        let manifest = manifest_and_keys.manifest;
        let current_idx = data
            .packages
            .iter()
            .position(|pkg| {
                pkg.name == manifest.package.name
                    && CrateData::is_same_path(pkg.manifest_path.as_std_path(), &manifest_path)
            })
            .ok_or_else(|| anyhow!("failed to find package in metadata"))?;

        Ok(CrateData {
            data,
            manifest,
            current_idx,
            out_name,
        })
    }

    fn is_same_path(path1: &Path, path2: &Path) -> bool {
        if let Ok(path1) = fs::canonicalize(&path1) {
            if let Ok(path2) = fs::canonicalize(&path2) {
                return path1 == path2;
            }
        }
        path1 == path2
    }

    /// Read the `manifest_path` file and deserializes it using the toml Deserializer.
    /// Returns a Result containing `ManifestAndUnsedKeys` which contains `CargoManifest`
    /// and a `BTreeSet<String>` containing the unused keys from the parsed file.
    ///
    /// # Errors
    /// Will return Err if the file (manifest_path) couldn't be read or
    /// if deserialize to `CargoManifest` fails.
    pub fn parse_crate_data(manifest_path: &Path) -> Result<ManifestAndUnsedKeys> {
        let manifest = fs::read_to_string(&manifest_path)
            .with_context(|| anyhow!("failed to read: {}", manifest_path.display()))?;
        let manifest = toml::Deserializer::new(&manifest);

        let mut unused_keys = BTreeSet::new();
        let levenshtein_threshold = 1;

        let manifest: CargoManifest = serde_ignored::deserialize(manifest, |path| {
            let path_string = path.to_string();

            if path_string.starts_with("package.metadata")
                && (path_string.contains("wasm-js")
                    || levenshtein(CARGO_METADATA_KEY, &path_string) <= levenshtein_threshold)
            {
                unused_keys.insert(path_string);
            }
        })
        .with_context(|| anyhow!("failed to parse manifest: {}", manifest_path.display()))?;

        Ok(ManifestAndUnsedKeys {
            manifest,
            unused_keys,
        })
    }

    /// Iterating through all the passed `unused_keys` and output
    /// a warning for each unknown key.
    pub fn warn_for_unused_keys(manifest_and_keys: &ManifestAndUnsedKeys) {
        manifest_and_keys.unused_keys.iter().for_each(|path| {
            PBAR.warn(&format!(
                "\"{}\" is an unknown key and will be ignored. Please check your Cargo.toml.",
                path
            ));
        });
    }

    /// Get the configured profile.
    pub fn configured_profile(&self, profile: BuildProfile) -> &CargoWasmPackProfile {
        match profile {
            BuildProfile::Dev => &self.manifest.package.metadata.wasm_js.profile.dev,
            BuildProfile::Profiling => &self.manifest.package.metadata.wasm_js.profile.profiling,
            BuildProfile::Release => &self.manifest.package.metadata.wasm_js.profile.release,
            BuildProfile::Custom(_) => &self.manifest.package.metadata.wasm_js.profile.custom,
        }
    }

    /// Check that the crate the given path is properly configured.
    pub fn check_crate_config(&self) -> Result<()> {
        self.check_crate_type()?;
        Ok(())
    }

    fn check_crate_type(&self) -> Result<()> {
        let pkg = &self.data.packages[self.current_idx];
        let any_cdylib = pkg
            .targets
            .iter()
            .filter(|target| target.kind.iter().any(|k| k == "cdylib"))
            .any(|target| target.crate_types.iter().any(|s| s == "cdylib"));
        if any_cdylib {
            return Ok(());
        }
        bail!(
            "crate-type must be cdylib to compile to wasm32-unknown-unknown. Add the following to your \
             Cargo.toml file:\n\n\
             [lib]\n\
             crate-type = [\"cdylib\", \"rlib\"]"
        )
    }

    fn pkg(&self) -> &cargo_metadata::Package {
        &self.data.packages[self.current_idx]
    }

    /// Get the crate name for the crate at the given path.
    pub fn crate_name(&self) -> String {
        let pkg = self.pkg();
        match pkg
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "cdylib"))
        {
            Some(lib) => lib.name.replace("-", "_"),
            None => pkg.name.replace("-", "_"),
        }
    }

    /// Get the prefix for output file names
    pub fn name_prefix(&self) -> String {
        match &self.out_name {
            Some(value) => value.clone(),
            None => self.crate_name(),
        }
    }

    /// Gets the optional path to the readme, or None if disabled.
    pub fn crate_readme(&self) -> Option<String> {
        self.pkg()
            .readme
            .clone()
            .map(|readme_file| readme_file.into_string())
    }

    /// Returns the path to this project's target directory where artifacts are
    /// located after a cargo build.
    pub fn target_directory(&self) -> &Path {
        Path::new(&self.data.target_directory)
    }

    /// Returns the path to this project's root cargo workspace directory
    pub fn workspace_root(&self) -> &Path {
        Path::new(&self.data.workspace_root)
    }
}

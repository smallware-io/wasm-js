use binary_install::Cache;
use std::env;
use std::fs;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use std::thread;
use tempfile::TempDir;
use wasm_js::install;
use wasm_js::install::Tool;
use wasm_js::wasm_opt;

/// A test fixture in a temporary directory.
pub struct Fixture {
    // NB: we wrap the fixture's tempdir in a `ManuallyDrop` so that if a test
    // fails, its directory isn't deleted, and we have a chance to manually
    // inspect its state and figure out what is going on.
    pub dir: ManuallyDrop<TempDir>,
    pub path: PathBuf,
}

impl Fixture {
    /// Create a new test fixture in a temporary directory.
    pub fn new() -> Fixture {
        // Make sure that all fixtures end up sharing a target dir, and we don't
        // recompile wasm-bindgen and friends many times over.
        static SET_TARGET_DIR: Once = Once::new();
        let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        SET_TARGET_DIR.call_once(|| {
            env::set_var("CARGO_TARGET_DIR", &target_dir);
        });

        let root = target_dir.join("t");
        fs::create_dir_all(&root).unwrap();
        let dir = TempDir::new_in(&root).unwrap();
        let path = dir.path().join("wasm-js");
        eprintln!("Created fixture at {}", path.display());
        Fixture {
            dir: ManuallyDrop::new(dir),
            path,
        }
    }

    /// Create a file within this fixture.
    ///
    /// `path` should be a relative path to the file (relative within this
    /// fixture's path).
    ///
    /// The `contents` are written to the file.
    pub fn file<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, contents: C) -> &Self {
        assert!(path.as_ref().is_relative());
        let path = self.path.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
        self
    }

    /// Add a generic `README.md` file to the fixture.
    pub fn readme(&self) -> &Self {
        self.file(
            "README.md",
            r#"
                # Fixture!
                > an example rust -> wasm project
            "#,
        )
    }

    /// Add a `Cargo.toml` with a correctly configured `wasm-bindgen`
    /// dependency, `wasm-bindgen-test` dev-dependency, and `crate-type =
    /// ["cdylib"]`.
    ///
    /// `name` is the crate's name.
    pub fn cargo_toml(&self, name: &str) -> &Self {
        self.file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    description = "so awesome rust+wasm package"
                    license = "WTFPL"
                    name = "{}"
                    version = "0.1.0"

                    [lib]
                    crate-type = ["cdylib"]

                    [dependencies]
                    # Note that this uses and `=` dependency because there are
                    # various tests which assert that the version of wasm
                    # bindgen downloaded is what we expect, and if `=` is
                    # removed then it will download whatever the newest version
                    # of wasm-bindgen is which may not be what's listed here.
                    wasm-bindgen = "=0.2.95"

                    [dev-dependencies]
                    wasm-bindgen-test = "0.3"
                "#,
                name
            ),
        )
    }

    /// Add a `Cargo.toml` with a correctly configured `wasm-bindgen`
    /// dependency, `wasm-bindgen-test` dev-dependency, and `crate-type =
    /// ["cdylib"]`.
    ///
    /// `name` is the crate's name.
    /// `profile` is the custom profile name.
    pub fn cargo_toml_with_custom_profile(&self, name: &str, profile_name: &str) -> &Self {
        self.file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    description = "so awesome rust+wasm package"
                    license = "WTFPL"
                    name = "{}"
                    version = "0.1.0"

                    [lib]
                    crate-type = ["cdylib"]

                    [dependencies]
                    # Note that this uses and `=` dependency because there are
                    # various tests which assert that the version of wasm
                    # bindgen downloaded is what we expect, and if `=` is
                    # removed then it will download whatever the newest version
                    # of wasm-bindgen is which may not be what's listed here.
                    wasm-bindgen = "=0.2.95"

                    [dev-dependencies]
                    wasm-bindgen-test = "0.3"

                    [profile.{}]
                    inherits = "release"
                    opt-level = 'z'
                    lto = true
                "#,
                name, profile_name
            ),
        )
    }

    /// Add a `src/lib.rs` file that contains a "hello world" program.
    pub fn hello_world_src_lib(&self) -> &Self {
        self.file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                // Import the `window.alert` function from the Web.
                #[wasm_bindgen]
                extern {
                    fn alert(s: &str);
                }

                // Export a `greet` function from Rust to JavaScript, that alerts a
                // hello message.
                #[wasm_bindgen]
                pub fn greet(name: &str) {
                    alert(&format!("Hello, {}!", name));
                }
            "#,
        )
    }

    /// Install a local wasm-bindgen for this fixture.
    ///
    /// Takes care not to re-install for every fixture, but only the one time
    /// for the whole test suite.
    pub fn install_local_wasm_bindgen(&self) -> PathBuf {
        // If wasm-bindgen is being used then it's very likely wasm-opt is going
        // to be used as well.
        self.install_wasm_opt();

        static INSTALL_WASM_BINDGEN: Once = Once::new();
        let cache = self.cache();
        let version = "0.2.95";

        let download = || {
            if let Ok(download) =
                install::download_prebuilt(&Tool::WasmBindgen, &cache, version, true)
            {
                return Ok(download);
            }

            install::cargo_install(Tool::WasmBindgen, &cache, version, true)
        };

        // Only one thread can perform the actual download, and then afterwards
        // everything will hit the cache so we can run the same path.
        INSTALL_WASM_BINDGEN.call_once(|| {
            download().unwrap();
        });
        if let install::Status::Found(dl) = download().unwrap() {
            dl.binary("wasm-bindgen").unwrap()
        } else {
            panic!("Download failed")
        }
    }

    pub fn install_wasm_opt(&self) {
        static INSTALL_WASM_OPT: Once = Once::new();
        let cache = self.cache();

        INSTALL_WASM_OPT.call_once(|| {
            wasm_opt::find_wasm_opt(&cache, true).unwrap();
        });
    }

    pub fn cache_dir(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test_cache")
    }

    pub fn cache(&self) -> Cache {
        let cache_dir = self.cache_dir();
        fs::create_dir_all(&cache_dir).unwrap();
        Cache::at(&cache_dir)
    }

    /// Get a command configured to run in this fixure's temp
    /// directory and using the test cache.
    pub fn wasm_js(&self) -> Command {
        use assert_cmd::prelude::*;
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.current_dir(&self.path);
        cmd.arg("--install-cache");
        cmd.arg(self.cache_dir());

        // Some of the tests assume that Cargo's output does not contain colors.
        cmd.env_remove("CARGO_TERM_COLOR");

        cmd
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !thread::panicking() {
            unsafe { ManuallyDrop::drop(&mut self.dir) }
        }
    }
}

pub fn js_hello_world() -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .cargo_toml("js-hello-world")
        .hello_world_src_lib();
    fixture
}

pub fn js_hello_world_with_custom_profile(profile_name: &str) -> Fixture {
    let fixture = Fixture::new();
    fixture
        .readme()
        .cargo_toml_with_custom_profile("js-hello-world", profile_name)
        .hello_world_src_lib();
    fixture
}

pub fn not_a_crate() -> Fixture {
    let fixture = Fixture::new();
    fixture.file("README.md", "This is not a Rust crate!");
    fixture
}

pub fn transitive_dependencies() -> Fixture {
    fn project_main_fixture(fixture: &mut Fixture) {
        fixture.file(PathBuf::from("main/README"), "# Main Fixture\n");
        fixture.file(
            PathBuf::from("main/Cargo.toml"),
            r#"
            [package]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "main_project"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            wasm-bindgen = "0.2"
            project_a = { path = "../project_a" }
            project_b = { path = "../project_b" }

            [dev-dependencies]
            wasm-bindgen-test = "0.3"
        "#,
        );
        fixture.file(
            PathBuf::from("main/src/lib.rs"),
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                // Import the `window.alert` function from the Web.
                #[wasm_bindgen]
                extern {
                    fn alert(s: &str);
                }

                // Export a `greet` function from Rust to JavaScript, that alerts a
                // hello message.
                #[wasm_bindgen]
                pub fn greet(name: &str) {
                    alert(&format!("Hello, {}!", name));
                }
            "#,
        );
    }

    fn project_a_fixture(fixture: &mut Fixture) {
        fixture.file(
            PathBuf::from("project_a/README"),
            "# Project Alpha Fixture\n",
        );
        fixture.file(
            PathBuf::from("project_a/Cargo.toml"),
            r#"
            [package]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "project_a"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            wasm-bindgen = "0.2"
            project_b = { path = "../project_b" }

            [dev-dependencies]
            wasm-bindgen-test = "0.3"
        "#,
        );
        fixture.file(
            PathBuf::from("project_a/src/lib.rs"),
            r#"
                extern crate wasm_bindgen;
                // extern crate project_b;
                use wasm_bindgen::prelude::*;

                // Import the `window.alert` function from the Web.
                #[wasm_bindgen]
                extern {
                    fn alert(s: &str);
                }

                // Export a `greet` function from Rust to JavaScript, that alerts a
                // hello message.
                #[wasm_bindgen]
                pub fn greet(name: &str) {
                    alert(&format!("Hello, {}!", name));
                }
            "#,
        );
    }

    fn project_b_fixture(fixture: &mut Fixture) {
        fixture.file(
            PathBuf::from("project_b/README"),
            "# Project Beta Fixture\n",
        );
        fixture.file(
            PathBuf::from("project_b/Cargo.toml"),
            r#"
            [package]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "project_b"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            wasm-bindgen = "0.2"

            [dev-dependencies]
            wasm-bindgen-test = "0.3"
        "#,
        );
        fixture.file(
            PathBuf::from("project_b/src/lib.rs"),
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                // Import the `window.alert` function from the Web.
                #[wasm_bindgen]
                extern {
                    fn alert(s: &str);
                }

                // Export a `greet` function from Rust to JavaScript, that alerts a
                // hello message.
                #[wasm_bindgen]
                pub fn greet(name: &str) {
                    alert(&format!("Hello, {}!", name));
                }
            "#,
        );
    }

    let mut fixture = Fixture::new();
    project_b_fixture(&mut fixture);
    project_a_fixture(&mut fixture);
    project_main_fixture(&mut fixture);
    fixture
}

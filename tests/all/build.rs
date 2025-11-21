use crate::utils;
use assert_cmd::prelude::*;
use std::fs;
use wasm_js::{
    command::{self, build::BuildOptions},
    Cli,
};

#[test]
fn build_in_non_crate_directory_doesnt_panic() {
    let fixture = utils::fixture::not_a_crate();
    fixture
        .wasm_js()
        .arg("build")
        .arg(".")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing a `Cargo.toml`"));
}

#[test]
fn it_should_build_js_hello_world_example() {
    let fixture = utils::fixture::js_hello_world();
    let mut args = Cli::from_command(command::Command::Build(BuildOptions {
        path: Some(fixture.path.clone()),
        out_dir: "dist".into(),
        ..Default::default()
    }));
    args.install_cache = Some(fixture.cache_dir().clone().to_string_lossy().to_string());
    command::run_command(&args).unwrap();
}

#[test]
fn it_should_not_make_a_pkg_json() {
    let fixture = utils::fixture::js_hello_world();
    fixture.wasm_js().arg("build").assert().success();

    let pkg_path = fixture.path.join("dist");
    assert_eq!(pkg_path.join("package.json").exists(), false);
    assert_eq!(pkg_path.join("README.md").exists(), false);
    assert_eq!(pkg_path.join("licence").exists(), false);
}

#[test]
fn it_should_build_js_hello_world_example_with_custom_target_dir() {
    let fixture = utils::fixture::js_hello_world();
    fixture
        .wasm_js()
        .arg("build")
        .arg("--target-dir")
        .arg("target2")
        .arg("--all-features")
        .arg("--offline")
        .assert()
        .success();
}

#[test]
fn renamed_crate_name_works() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [lib]
                crate-type = ["cdylib"]
                name = 'bar'

                [dependencies]
                wasm-bindgen = "0.2"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn one() -> u32 { 1 }
            "#,
        )
        .install_local_wasm_bindgen();
    fixture.wasm_js().arg("build").assert().success();
}

#[test]
fn it_should_build_nested_project_with_transitive_dependencies() {
    let fixture = utils::fixture::transitive_dependencies();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_js()
        .current_dir(fixture.path.join("main"))
        .arg("build")
        .assert()
        .success();
}

#[test]
fn build_different_profiles() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();

    for profile in ["--dev", "--debug", "--profiling", "--release"]
        .iter()
        .cloned()
    {
        fixture
            .wasm_js()
            .arg("build")
            .arg(profile)
            .assert()
            .success();
    }
}

#[test]
fn build_custom_profile() {
    let profile_name = "my-custom-profile";
    let fixture = utils::fixture::js_hello_world_with_custom_profile(profile_name);
    fixture.install_local_wasm_bindgen();

    fixture
        .wasm_js()
        .arg("build")
        .arg("--profile")
        .arg(profile_name)
        .assert()
        .success();
}

#[test]
fn build_with_and_without_wasm_bindgen_debug() {
    for debug in [true, false].iter().cloned() {
        let fixture = utils::fixture::Fixture::new();
        fixture
            .readme()
            .file(
                "Cargo.toml",
                format!(
                    r#"
                    [package]
                    description = "so awesome rust+wasm package"
                    license = "WTFPL"
                    name = "whatever"
                    version = "0.1.0"

                    [lib]
                    crate-type = ["cdylib"]

                    [dependencies]
                    wasm-bindgen = "0.2"

                    [package.metadata.wasm-js.profile.dev.wasm-bindgen]
                    debug-js-glue = {}
                    "#,
                    debug
                ),
            )
            .file(
                "src/lib.rs",
                r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub struct MyThing {}

                #[wasm_bindgen]
                impl MyThing {
                    #[wasm_bindgen(constructor)]
                    pub fn new() -> MyThing {
                        MyThing {}
                    }
                }

                #[wasm_bindgen]
                pub fn take(foo: MyThing) {
                    drop(foo);
                }
                "#,
            )
            .install_local_wasm_bindgen();

        fixture
            .wasm_js()
            .arg("build")
            .arg("--dev")
            .assert()
            .success();

        let contents = fs::read_to_string(fixture.path.join("dist/whatever_bg.js")).unwrap();
        let contains_move_assertions =
            contents.contains("throw new Error('Attempt to use a moved value')");
        assert_eq!(
            contains_move_assertions, debug,
            "Should contain moved value assertions iff debug assertions are enabled. \
             Contains move assertions? {}. \
             Is a debug JS glue build? {}.",
            contains_move_assertions, debug,
        );
    }
}

#[test]
fn build_with_arbitrary_cargo_options() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_js()
        .arg("build")
        .arg("--no-default-features")
        .assert()
        .success();
}

#[test]
fn build_no_install() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_js()
        .arg("build")
        .arg("--mode")
        .arg("no-install")
        .assert()
        .success();
}

#[test]
fn build_force() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_js()
        .arg("build")
        .arg("--mode")
        .arg("force")
        .assert()
        .success();
}

#[test]
fn build_crates_with_same_names() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "somename1/Cargo.toml",
            r#"
            [package]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "somename"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            wasm-bindgen = "0.2"
            somenameother = { path = "../somename2", package = "somename" }
            "#,
        )
        .file(
            "somename1/src/lib.rs",
            r#"
            extern crate wasm_bindgen;
            use wasm_bindgen::prelude::*;
            #[wasm_bindgen]
            pub fn method() -> i32 {
                somenameother::method()
            }
            "#,
        )
        .file(
            "somename2/Cargo.toml",
            r#"
            [package]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "somename"
            version = "0.1.1"

            [lib]
            crate-type = ["rlib"]
            "#,
        )
        .file(
            "somename2/src/lib.rs",
            r#"
            pub fn method() -> i32 {
                0
            }
            "#,
        );
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_js()
        .current_dir(fixture.path.join("somename1"))
        .arg("build")
        .assert()
        .success();
}

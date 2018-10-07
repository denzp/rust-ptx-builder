#[macro_use]
extern crate lazy_static;
extern crate antidote;
extern crate ptx_builder;

use antidote::Mutex;
use std::env;
use std::env::current_dir;
use std::fs::{remove_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;

use ptx_builder::builder::{BuildStatus, Builder, Profile};
use ptx_builder::error::*;

lazy_static! {
    static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
}

#[test]
fn should_provide_output_path() {
    remove_dir_all(env::temp_dir().join("ptx-builder-0.5")).unwrap_or_default();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            assert!(
                output.get_assembly_path().starts_with(
                    env::temp_dir()
                        .join("ptx-builder-0.5")
                        .join("sample_ptx_crate"),
                )
            );
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_write_assembly() {
    remove_dir_all(env::temp_dir().join("ptx-builder-0.5")).unwrap_or_default();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(
                output
                    .get_assembly_path()
                    .to_string_lossy()
                    .contains("release")
            );

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_build_application_crate() {
    remove_dir_all(env::temp_dir().join("ptx-builder-0.5")).unwrap_or_default();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/app-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(
                output
                    .get_assembly_path()
                    .to_string_lossy()
                    .contains("release")
            );

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_write_assembly_in_debug_mode() {
    remove_dir_all(env::temp_dir().join("ptx-builder-0.5")).unwrap_or_default();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder
        .set_profile(Profile::Debug)
        .disable_colors()
        .build()
        .unwrap()
    {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(
                output
                    .get_assembly_path()
                    .to_string_lossy()
                    .contains("debug")
            );

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_report_about_build_failure() {
    remove_dir_all(env::temp_dir().join("ptx-builder-0.5")).unwrap_or_default();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/faulty-crate")
        .unwrap()
        .disable_colors();

    let output = builder.build();
    let crate_absoulte_path = current_dir()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("faulty-crate");

    let lib_path = PathBuf::from("src").join("lib.rs");

    let mut crate_absoulte_path_str = crate_absoulte_path.display().to_string();

    if cfg!(windows) {
        crate_absoulte_path_str = format!("/{}", crate_absoulte_path_str.replace("\\", "/"));
    }

    match output {
        Err(Error(ErrorKind::BuildFailed(diagnostics), _)) => {
            assert_eq!(
                diagnostics
                    .into_iter()
                    .filter(|item| !item.contains("Blocking waiting")
                        && !item.contains("Compiling core")
                        && !item.contains("Compiling compiler_builtins")
                        && !item.contains("Finished release [optimized] target(s)")
                        && !item.contains("Running")
                        && !item.starts_with("+ ")
                        && !item.starts_with("Caused by:")
                        && !item.starts_with("  process didn\'t exit successfully: "))
                    .collect::<Vec<_>>(),
                &[
                    format!(
                        "   Compiling faulty-ptx_crate v0.1.0 ({})",
                        crate_absoulte_path_str
                    ),
                    String::from("error[E0425]: cannot find function `external_fn` in this scope"),
                    format!(" --> {}:6:20", lib_path.display()),
                    String::from("  |"),
                    String::from("6 |     *y.offset(0) = external_fn(*x.offset(0)) * a;"),
                    String::from("  |                    ^^^^^^^^^^^ not found in this scope"),
                    String::from(""),
                    String::from("error: aborting due to previous error"),
                    String::from(""),
                    String::from(
                        "For more information about this error, try `rustc --explain E0425`.",
                    ),
                    String::from("error: Could not compile `faulty-ptx_crate`."),
                    String::from(""),
                ]
            );
        }

        Ok(_) => unreachable!("it should fail"),
        Err(_) => unreachable!("it should fail with proper error"),
    }
}

#[test]
fn should_provide_crate_source_files() {
    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut sources = output.dependencies().unwrap();
            let mut expectations = vec![
                current_dir()
                    .unwrap()
                    .join("tests")
                    .join("fixtures")
                    .join("sample-crate")
                    .join("src")
                    .join("lib.rs"),
                current_dir()
                    .unwrap()
                    .join("tests")
                    .join("fixtures")
                    .join("sample-crate")
                    .join("src")
                    .join("mod1.rs"),
                current_dir()
                    .unwrap()
                    .join("tests")
                    .join("fixtures")
                    .join("sample-crate")
                    .join("src")
                    .join("mod2.rs"),
                current_dir()
                    .unwrap()
                    .join("tests")
                    .join("fixtures")
                    .join("sample-crate")
                    .join("Cargo.toml"),
                current_dir()
                    .unwrap()
                    .join("tests")
                    .join("fixtures")
                    .join("sample-crate")
                    .join("Cargo.lock"),
            ];

            sources.sort();
            expectations.sort();

            assert_eq!(sources, expectations);
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_not_get_built_from_rls() {
    let _lock = ENV_MUTEX.lock();
    env::set_var("CARGO", "some/path/to/rls");

    assert_eq!(Builder::is_build_needed(), false);
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::NotNeeded => {}
        BuildStatus::Success(_) => unreachable!(),
    }

    env::set_var("CARGO", "");
}

#[test]
fn should_not_get_built_recursively() {
    let _lock = ENV_MUTEX.lock();
    env::set_var("PTX_CRATE_BUILDING", "1");

    assert_eq!(Builder::is_build_needed(), false);
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::NotNeeded => {}
        BuildStatus::Success(_) => unreachable!(),
    }

    env::set_var("PTX_CRATE_BUILDING", "");
}

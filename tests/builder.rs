use std::env;
use std::env::current_dir;
use std::fs::{remove_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;

use antidote::Mutex;
use lazy_static::*;

use ptx_builder::error::*;
use ptx_builder::prelude::*;

lazy_static! {
    static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
}

#[test]
fn should_provide_output_path() {
    cleanup_temp_location();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            assert!(output.get_assembly_path().starts_with(
                env::temp_dir()
                    .join("ptx-builder-0.5")
                    .join("sample_ptx_crate"),
            ));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_write_assembly() {
    cleanup_temp_location();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/sample-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(output
                .get_assembly_path()
                .to_string_lossy()
                .contains("release"));

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_build_application_crate() {
    cleanup_temp_location();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/app-crate").unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(output
                .get_assembly_path()
                .to_string_lossy()
                .contains("release"));

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_build_mixed_crate_lib() {
    cleanup_temp_location();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/mixed-crate").unwrap();

    match builder
        .set_crate_type(CrateType::Library)
        .disable_colors()
        .build()
        .unwrap()
    {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            println!("{}", output.get_assembly_path().display());

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(output
                .get_assembly_path()
                .to_string_lossy()
                .contains("release"));

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_build_mixed_crate_bin() {
    cleanup_temp_location();

    let _lock = ENV_MUTEX.lock();
    let builder = Builder::new("tests/fixtures/mixed-crate").unwrap();

    match builder
        .set_crate_type(CrateType::Binary)
        .disable_colors()
        .build()
        .unwrap()
    {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            println!("{}", output.get_assembly_path().display());

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(output
                .get_assembly_path()
                .to_string_lossy()
                .contains("release"));

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_handle_rebuild_without_changes() {
    cleanup_temp_location();

    let _lock = ENV_MUTEX.lock();
    let builder = {
        Builder::new("tests/fixtures/app-crate")
            .unwrap()
            .disable_colors()
    };

    builder.build().unwrap();

    match builder.build().unwrap() {
        BuildStatus::Success(output) => {
            let mut assembly_contents = String::new();

            File::open(output.get_assembly_path())
                .unwrap()
                .read_to_string(&mut assembly_contents)
                .unwrap();

            assert!(output
                .get_assembly_path()
                .to_string_lossy()
                .contains("release"));

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_write_assembly_in_debug_mode() {
    cleanup_temp_location();

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

            assert!(output
                .get_assembly_path()
                .to_string_lossy()
                .contains("debug"));

            assert!(assembly_contents.contains(".visible .entry the_kernel("));
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_report_about_build_failure() {
    cleanup_temp_location();

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

    let crate_absoulte_path_str = crate_absoulte_path.display().to_string();

    match output.unwrap_err().kind() {
        BuildErrorKind::BuildFailed(diagnostics) => {
            assert_eq!(
                diagnostics
                    .into_iter()
                    .filter(|item| !item.contains("Blocking waiting")
                        && !item.contains("Compiling core")
                        && !item.contains("Compiling compiler_builtins")
                        && !item.contains("Finished release [optimized] target(s)"))
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
                    String::from("error: could not compile `faulty-ptx_crate`."),
                    String::from(""),
                ]
            );
        }

        _ => unreachable!("it should fail with proper error"),
    }
}

#[test]
fn should_provide_crate_source_files() {
    let _lock = ENV_MUTEX.lock();

    let crate_path = {
        current_dir()
            .unwrap()
            .join("tests")
            .join("fixtures")
            .join("sample-crate")
    };

    let builder = Builder::new(&crate_path.display().to_string()).unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut sources = output.dependencies().unwrap();
            let mut expectations = vec![
                crate_path.join("src").join("lib.rs"),
                crate_path.join("src").join("mod1.rs"),
                crate_path.join("src").join("mod2.rs"),
                crate_path.join("Cargo.toml"),
                crate_path.join("Cargo.lock"),
            ];

            sources.sort();
            expectations.sort();

            assert_eq!(sources, expectations);
        }

        BuildStatus::NotNeeded => unreachable!(),
    }
}

#[test]
fn should_provide_application_crate_source_files() {
    let _lock = ENV_MUTEX.lock();

    let crate_path = {
        current_dir()
            .unwrap()
            .join("tests")
            .join("fixtures")
            .join("app-crate")
    };

    let builder = Builder::new(&crate_path.display().to_string()).unwrap();

    match builder.disable_colors().build().unwrap() {
        BuildStatus::Success(output) => {
            let mut sources = output.dependencies().unwrap();
            let mut expectations = vec![
                crate_path.join("src").join("main.rs"),
                crate_path.join("src").join("mod1.rs"),
                crate_path.join("src").join("mod2.rs"),
                crate_path.join("Cargo.toml"),
                crate_path.join("Cargo.lock"),
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

fn cleanup_temp_location() {
    let crate_names = &[
        "faulty_ptx_crate",
        "sample_app_ptx_crate",
        "sample_ptx_crate",
        "mixed_crate",
    ];

    for name in crate_names {
        remove_dir_all(env::temp_dir().join("ptx-builder-0.5").join(name)).unwrap_or_default();
    }
}

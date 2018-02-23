extern crate ptx_builder;

use std::io::prelude::*;
use std::fs::{remove_dir_all, File};
use std::env::current_dir;

use ptx_builder::error::*;
use ptx_builder::project::{Crate, Project};
use ptx_builder::builder::Builder;

#[test]
fn should_provide_output_path() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();
    let output = Builder::new("tests/fixtures/sample-crate")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(
        output.get_assembly_path(),
        project
            .get_proxy_crate()
            .unwrap()
            .get_output_path()
            .join("nvptx64-nvidia-cuda/release/proxy.ptx")
    );
}

#[test]
fn should_write_assembly() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();
    remove_dir_all(project.get_proxy_crate().unwrap().get_output_path()).unwrap_or_default();

    let output = Builder::new("tests/fixtures/sample-crate")
        .unwrap()
        .build()
        .unwrap();

    let mut assembly_contents = String::new();

    File::open(output.get_assembly_path())
        .unwrap()
        .read_to_string(&mut assembly_contents)
        .unwrap();

    assert!(assembly_contents.contains(".visible .entry the_kernel("));
}

#[test]
fn should_report_about_build_failure() {
    let output = Builder::new("tests/fixtures/faulty-crate")
        .unwrap()
        .disable_colors()
        .build();

    let crate_absoulte_path = current_dir().unwrap().join("tests/fixtures/faulty-crate");

    match output {
        Err(Error(ErrorKind::BuildFailed(diagnostics), _)) => {
            assert_eq!(
                diagnostics
                    .into_iter()
                    .filter(|item| !item.contains("Blocking waiting")
                        && !item.contains("Compiling core")
                        && !item.contains("Finished release [optimized] target(s)"))
                    .collect::<Vec<_>>(),
                &[
                    format!(
                        "   Compiling faulty-ptx_crate v0.1.0 (file://{})",
                        crate_absoulte_path.as_path().to_str().unwrap()
                    ),
                    String::from("error[E0425]: cannot find function `external_fn` in this scope"),
                    format!(
                        " --> {}/src/lib.rs:6:20",
                        crate_absoulte_path.as_path().to_str().unwrap()
                    ),
                    String::from("  |"),
                    String::from("6 |     *y.offset(0) = external_fn(*x.offset(0)) * a;"),
                    String::from("  |                    ^^^^^^^^^^^ not found in this scope"),
                    String::from(""),
                    String::from("error: aborting due to previous error"),
                    String::from(""),
                    String::from("error: Could not compile `faulty-ptx_crate`."),
                    String::from(""),
                    String::from("To learn more, run the command again with --verbose."),
                ]
            );
        }

        Ok(_) => unreachable!("it should fail"),
        Err(_) => unreachable!("it should fail with proper error"),
    }
}

#[test]
fn should_provide_crate_source_files() {
    let output = Builder::new("tests/fixtures/sample-crate")
        .unwrap()
        .build()
        .unwrap();

    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();
    let proxy_crate = project.get_proxy_crate().unwrap();

    let mut sources = output.source_files().unwrap();
    sources.sort();

    assert_eq!(
        sources,
        &[
            current_dir()
                .unwrap()
                .join("tests/fixtures/sample-crate/src/lib.rs"),
            current_dir()
                .unwrap()
                .join("tests/fixtures/sample-crate/src/mod1.rs"),
            current_dir()
                .unwrap()
                .join("tests/fixtures/sample-crate/src/mod2.rs"),
            proxy_crate.get_path().join("src/lib.rs"),
        ]
    );
}

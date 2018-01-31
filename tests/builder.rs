extern crate ptx_builder;

use std::io::prelude::*;
use std::fs::{remove_dir_all, File};

use ptx_builder::error::*;
use ptx_builder::project::Project;
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
            .get_output_path()
            .join("nvptx64-nvidia-cuda/release/sample_ptx_crate.ptx")
    );
}

#[test]
fn should_write_assembly() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();
    remove_dir_all(project.get_output_path()).unwrap_or_default();

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
        .as_mut()
        .unwrap()
        .disable_colors()
        .build();

    match output {
        Err(Error(ErrorKind::BuildFailed(diagnostics), _)) => {
            assert_eq!(diagnostics, &[
                "   Compiling faulty-ptx_crate v0.1.0 (file:///home/den/rust-ptx-builder/tests/fixtures/faulty-crate)",
                "error[E0425]: cannot find function `external_fn` in this scope",
                " --> src/lib.rs:6:20",
                "  |",
                "6 |     *y.offset(0) = external_fn(*x.offset(0)) * a;",
                "  |                    ^^^^^^^^^^^ not found in this scope",
                "",
                "error: aborting due to previous error",
                "",
                "error: Could not compile `faulty-ptx_crate`.",
                "",
                "To learn more, run the command again with --verbose.",
            ]);
        }

        Ok(_) => unreachable!("it should fail"),
        Err(_) => unreachable!("it should fail with proper error"),
    }
}

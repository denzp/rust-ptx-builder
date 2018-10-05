extern crate ptx_builder;

use std::env;

use ptx_builder::error::*;
use ptx_builder::source::Crate;

#[test]
fn should_find_crate_names() {
    let source = Crate::analyze("tests/fixtures/sample-crate").unwrap();

    assert_eq!(source.get_output_file_prefix(), "sample_ptx_crate");
    assert_eq!(source.get_deps_file_prefix(), "libsample_ptx_crate");
}

#[test]
fn should_find_app_crate_names() {
    let source = Crate::analyze("tests/fixtures/app-crate").unwrap();

    assert_eq!(source.get_output_file_prefix(), "sample_app_ptx_crate");
    assert_eq!(source.get_deps_file_prefix(), "sample_app_ptx_crate");
}

#[test]
fn should_check_existence_of_crate_path() {
    let result = Crate::analyze("tests/fixtures/non-existing-crate");

    match result {
        Err(Error(ErrorKind::InvalidCratePath(path), _)) => {
            assert!(path.ends_with("tests/fixtures/non-existing-crate"));
        }

        Ok(_) => unreachable!("it should fail"),
        Err(_) => unreachable!("it should fail with proper error"),
    }
}

#[test]
fn should_check_validity_of_crate_path() {
    let result = Crate::analyze("tests/builder.rs");

    match result {
        Err(Error(ErrorKind::InvalidCratePath(path), _)) => {
            assert!(path.ends_with("tests/builder.rs"));
        }

        Ok(_) => unreachable!("it should fail"),
        Err(_) => unreachable!("it should fail with proper error"),
    }
}

#[test]
fn should_provide_output_path() {
    let source_crate = Crate::analyze("tests/fixtures/sample-crate").unwrap();

    assert!(
        source_crate.get_output_path().unwrap().starts_with(
            env::temp_dir()
                .join("ptx-builder-0.5")
                .join("sample_ptx_crate")
        )
    );
}

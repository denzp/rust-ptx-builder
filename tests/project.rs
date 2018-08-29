extern crate ptx_builder;

use std::env;

use ptx_builder::error::*;
use ptx_builder::project::{Crate, Project};

#[test]
fn should_find_crate_names() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();

    assert_eq!(project.get_name(), "sample-ptx_crate");
    assert_eq!(project.get_rustc_name(), "sample_ptx_crate");
}

#[test]
fn should_check_existence_of_crate_path() {
    let result = Project::analyze("tests/fixtures/non-existing-crate");

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
    let result = Project::analyze("tests/builder.rs");

    match result {
        Err(Error(ErrorKind::InvalidCratePath(path), _)) => {
            assert!(path.ends_with("tests/builder.rs"));
        }

        Ok(_) => unreachable!("it should fail"),
        Err(_) => unreachable!("it should fail with proper error"),
    }
}

#[test]
fn should_provide_proxy_crate() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();
    let proxy = project.get_proxy_crate().unwrap();

    assert!(
        proxy.get_output_path().starts_with(
            env::temp_dir()
                .join("ptx-builder-0.4")
                .join("sample_ptx_crate")
        )
    );
}

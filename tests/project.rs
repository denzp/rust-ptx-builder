extern crate ptx_builder;

use ptx_builder::project::Project;

#[test]
fn should_find_crate_name() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();

    assert_eq!(project.get_name(), "sample_ptx_crate");
}

#[test]
fn should_provide_consistent_hash() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();

    assert_eq!(project.get_hash(), project.get_hash());
}

#[test]
fn should_provide_output_path() {
    let project = Project::analyze("tests/fixtures/sample-crate").unwrap();

    assert!(project.get_output_path().starts_with("/tmp/ptx-builder"));
    assert!(project.get_output_path().ends_with("sample_ptx_crate"));
}

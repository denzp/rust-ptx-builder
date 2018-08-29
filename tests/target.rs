extern crate ptx_builder;

use std::env;
use std::fs::{remove_dir_all, File};
use std::io::prelude::*;

use ptx_builder::target::TargetInfo;

#[test]
fn should_provide_target_name() {
    let target = TargetInfo::new().unwrap();

    assert_eq!(target.get_target_name(), "nvptx64-nvidia-cuda");
}

#[test]
fn should_provide_definitions_path() {
    let target = TargetInfo::new().unwrap();

    assert_eq!(
        target.get_path(),
        env::temp_dir().join("ptx-builder-targets")
    );
}

#[test]
fn should_store_json_definition() {
    remove_dir_all("/tmp/ptx-builder-targets").unwrap_or_default();

    TargetInfo::new().unwrap();
    let path = env::temp_dir()
        .join("ptx-builder-targets")
        .join("nvptx64-nvidia-cuda.json");

    let mut contents = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    assert!(contents.contains(r#""arch": "nvptx64","#));
}

use std::env;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use crate::error::*;
use crate::executable::{ExecutableRunner, Linker};

const DEFAULT_TARGET_NAME: &str = "nvptx64-nvidia-cuda";

/// Details about CUDA target.
///
/// Only `nvptx64-nvidia-cuda` is supported right now.
pub struct TargetInfo {
    path: PathBuf,
}

impl TargetInfo {
    /// Prepares temporary location of JSON definition for default target.
    pub fn new() -> Result<Self> {
        let output_dir = env::temp_dir().join("ptx-builder-targets-0.5");

        create_dir_all(output_dir.as_path())
            .chain_err(|| "Unable to create target definitions directory")?;

        let linker_output = ExecutableRunner::new(Linker)
            .with_args(&["print", DEFAULT_TARGET_NAME])
            .run()?;

        let output_path = output_dir.join(format!("{}.json", DEFAULT_TARGET_NAME));

        BufWriter::new(File::create(output_path.as_path())?)
            .write_all(&linker_output.stdout.as_bytes())
            .chain_err(|| format!("Unable to write {}", output_path.display()))?;

        Ok(TargetInfo { path: output_dir })
    }

    /// Returns target JSON definition location.
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    /// Returns target name.
    pub fn get_target_name(&self) -> &str {
        DEFAULT_TARGET_NAME
    }
}

#[cfg(test)]
use std::fs::remove_dir_all;

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
        env::temp_dir().join("ptx-builder-targets-0.5")
    );
}

#[test]
fn should_store_json_definition() {
    remove_dir_all("/tmp/ptx-builder-targets").unwrap_or_default();

    TargetInfo::new().unwrap();
    let path = env::temp_dir()
        .join("ptx-builder-targets-0.5")
        .join("nvptx64-nvidia-cuda.json");

    let mut contents = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    assert!(contents.contains(r#""arch": "nvptx64","#));
}

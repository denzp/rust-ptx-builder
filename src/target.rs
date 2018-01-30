use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::BufWriter;
use std::env;

use error::*;
use executable::{ExecutableRunner, Linker};

const DEFAULT_TARGET_NAME: &str = "nvptx64-nvidia-cuda";

pub struct TargetInfo {
    path: PathBuf,
}

impl TargetInfo {
    pub fn new() -> Result<Self> {
        let output_dir = env::temp_dir().join("ptx-builder-targets");
        let output_path = output_dir
            .clone()
            .join(format!("{}.json", DEFAULT_TARGET_NAME));

        let linker_output = ExecutableRunner::new(Linker)
            .with_args(&["--print-target-json", DEFAULT_TARGET_NAME])
            .run()?;

        create_dir_all(output_dir.as_path())
            .chain_err(|| "Unable to create target definitions directory")?;

        BufWriter::new(File::create(output_path.as_path())?)
            .write_all(&linker_output.stdout.as_bytes())
            .chain_err(|| format!("Unable to write {}", output_path.to_str().unwrap()))?;

        Ok(TargetInfo { path: output_dir })
    }

    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn get_target_name(&self) -> &str {
        DEFAULT_TARGET_NAME
    }
}

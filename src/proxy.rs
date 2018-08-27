use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs::{create_dir_all, metadata, File};
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use error::*;
use project::{Crate, Project};

pub struct ProxyCrate<'a> {
    project: &'a Project,
    path: PathBuf,
}

const DEFAULT_MANIFEST_PREFIX: &str = r#"
[package]
name = "proxy"
version = "0.0.0"

[lib]
crate_type = ["dylib"]

[dependencies]
"#;

const DEFAULT_LIB_PREFIX: &str = r#"
#![feature(panic_handler)]
#![no_std]

"#;

const DEFAULT_LIB_SUFFIX: &str = r#"

// Needed because we compile `dylib`...
#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
    loop {}
}
"#;

impl<'a> ProxyCrate<'a> {
    pub fn new(project: &'a Project) -> Result<Self> {
        let mut path = env::temp_dir().join("ptx-builder-0.4");

        path.push(&project.get_rustc_name());
        path.push(format!("{:x}", Self::get_project_hash(project)));

        create_dir_all(&path)?;
        create_dir_all(&path.join("src"))?;

        Ok(ProxyCrate { project, path })
    }

    pub fn get_output_path(&self) -> PathBuf {
        self.path.join("target")
    }

    pub fn initialize(&mut self) -> Result<()> {
        if let Err(_) = metadata(self.path.join("Cargo.toml")) {
            let mut writer = BufWriter::new(File::create(self.path.join("Cargo.toml"))?);

            writer.write_all(DEFAULT_MANIFEST_PREFIX.as_bytes())?;
            writer.write_all(
                format!(
                    r#"{} = {{ path = {:?} }} "#,
                    self.project.get_name(),
                    self.project.get_path()
                ).as_bytes(),
            )?;
        }

        if let Err(_) = metadata(self.path.join("src/lib.rs")) {
            let mut writer = BufWriter::new(File::create(self.path.join("src/lib.rs"))?);

            writer.write_all(DEFAULT_LIB_PREFIX.as_bytes())?;
            writer.write_all(
                format!("extern crate {name};", name = self.project.get_rustc_name()).as_bytes(),
            )?;
            writer.write_all(DEFAULT_LIB_SUFFIX.as_bytes())?;
        }

        Ok(())
    }

    fn get_project_hash(project: &Project) -> u64 {
        let mut hasher = DefaultHasher::new();
        project.hash(&mut hasher);

        hasher.finish()
    }
}

impl<'a> Crate for ProxyCrate<'a> {
    fn get_path(&self) -> &Path {
        &self.path.as_path()
    }

    fn get_name(&self) -> &str {
        "proxy"
    }

    fn get_rustc_name(&self) -> &str {
        "proxy"
    }
}

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use error_chain::bail;
use toml;

use crate::error::*;

#[derive(Hash, Clone)]
pub enum CrateType {
    Library,
    Application,
}

#[derive(Hash, Clone)]
pub struct Crate {
    path: PathBuf,

    crate_type: CrateType,
    output_file_prefix: String,
    deps_file_prefix: String,
}

impl Crate {
    pub fn analyze<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = env::current_dir()?.join(&path);

        match fs::metadata(path.join("Cargo.toml")) {
            Ok(metadata) => {
                if metadata.is_dir() {
                    bail!(ErrorKind::InvalidCratePath(path.clone()));
                }
            }

            Err(_) => {
                bail!(ErrorKind::InvalidCratePath(path.clone()));
            }
        }

        let cargo_toml: toml::Value = {
            let mut reader = BufReader::new(fs::File::open(path.join("Cargo.toml"))?);
            let mut contents = String::new();

            reader.read_to_string(&mut contents)?;
            toml::from_str(&contents)?
        };

        let cargo_toml_name = match cargo_toml["package"]["name"].as_str() {
            Some(name) => name,
            None => {
                bail!(ErrorKind::InternalError(String::from(
                    "Cannot get crate name"
                )));
            }
        };

        let crate_type = if path.join("src").join("lib.rs").exists() {
            CrateType::Library
        } else {
            CrateType::Application
        };

        let output_file_prefix = cargo_toml_name.replace("-", "_");

        let deps_file_prefix = match crate_type {
            CrateType::Library => format!("lib{}", cargo_toml_name.replace("-", "_")),
            CrateType::Application => output_file_prefix.clone(),
        };

        Ok(Crate {
            path,
            crate_type,
            output_file_prefix,
            deps_file_prefix,
        })
    }

    pub fn get_output_file_prefix(&self) -> &str {
        &self.output_file_prefix
    }

    pub fn get_deps_file_prefix(&self) -> &str {
        &self.deps_file_prefix
    }

    pub fn get_path(&self) -> &Path {
        &self.path.as_path()
    }

    pub fn get_output_path(&self) -> Result<PathBuf> {
        let mut path = env::temp_dir().join("ptx-builder-0.5");

        path.push(&self.output_file_prefix);
        path.push(format!("{:x}", self.get_hash()));

        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        hasher.finish()
    }
}

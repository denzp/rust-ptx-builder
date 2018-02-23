use std::env;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use toml;

use error::*;
use executable::{Cargo, ExecutableRunner};
use proxy::ProxyCrate;

pub trait Crate {
    fn get_name(&self) -> &str;
    fn get_rustc_name(&self) -> &str;
    fn get_path(&self) -> &Path;
}

#[derive(Hash)]
pub struct Project {
    path: PathBuf,

    name: String,
    rustc_name: String,
}

impl Project {
    pub fn analyze<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = env::current_dir()?.join(&path);

        match fs::metadata(path.join("Cargo.toml")) {
            Ok(metadata) => if metadata.is_dir() {
                bail!(ErrorKind::InvalidCratePath(path.clone()));
            },
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

        let output = ExecutableRunner::new(Cargo)
            .with_args(&["rustc", "-q", "--", "--print", "crate-name"])
            .with_cwd(path.as_path())
            .with_env("CARGO_TARGET_DIR", env::temp_dir())
            .run()
            .chain_err(|| "Unable to get crate name with cargo")?;

        Ok(Project {
            path,
            name: String::from(cargo_toml_name),
            rustc_name: String::from(output.stdout.trim()),
        })
    }

    pub fn get_proxy_crate(&self) -> Result<ProxyCrate> {
        ProxyCrate::new(self)
    }
}

impl Crate for Project {
    fn get_rustc_name(&self) -> &str {
        &self.rustc_name
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_path(&self) -> &Path {
        &self.path.as_path()
    }
}

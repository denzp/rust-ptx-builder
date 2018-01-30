use std::env;
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use error::*;
use executable::{Cargo, ExecutableRunner};

#[derive(Hash)]
pub struct Project {
    path: PathBuf,
    name: String,
}

impl Project {
    pub fn analyze<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = env::current_dir()?.join(&path);

        let output = ExecutableRunner::new(Cargo)
            .with_args(&["rustc", "-q", "--", "--print", "crate-name"])
            .with_cwd(path.as_path())
            .run()?;

        Ok(Project {
            path,
            name: String::from(output.stdout.trim()),
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_crate_path(&self) -> &Path {
        &self.path.as_path()
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        hasher.finish()
    }

    pub fn get_output_path(&self) -> PathBuf {
        let mut path = env::temp_dir().join("ptx-builder");

        path.push(format!("{:x}", self.get_hash()));
        path.push(&self.name);

        path
    }
}

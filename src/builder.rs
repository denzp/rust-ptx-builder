use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, Read};
use std::env;

use error::*;
use project::{Crate, Project};
use target::TargetInfo;
use executable::{ExecutableRunner, Xargo};

pub struct Builder {
    project: Project,
    target: TargetInfo,

    colors: bool,

    is_rls_build: bool,
    is_recursive_build: bool,
}

pub struct Output {
    output_path: PathBuf,
    crate_name: String,
}

pub enum BuildStatus {
    Success(Output),
    NotNeeded,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Builder {
            project: Project::analyze(path).chain_err(|| "Unable to analyze project")?,
            target: TargetInfo::new().chain_err(|| "Unable to get target details")?,

            colors: true,

            is_rls_build: {
                env::var("CARGO").is_ok() && env::var("CARGO").unwrap().ends_with("rls")
            },
            is_recursive_build: {
                env::var("PTX_CRATE_BUILDING").is_ok()
                    && env::var("PTX_CRATE_BUILDING").unwrap() == "1"
            },
        })
    }

    pub fn disable_colors(&mut self) -> &mut Self {
        self.colors = false;
        self
    }

    pub fn build(&mut self) -> Result<BuildStatus> {
        if self.is_rls_build || self.is_recursive_build {
            return Ok(BuildStatus::NotNeeded);
        }

        let mut proxy = {
            self.project
                .get_proxy_crate()
                .chain_err(|| "Unable to create proxy crate")?
        };

        proxy
            .initialize()
            .chain_err(|| "Unable to initialize proxy crate")?;

        let mut xargo = ExecutableRunner::new(Xargo);

        xargo
            .with_args(&[
                "build",
                "--release",
                "--color",
                {
                    match self.colors {
                        true => "always",
                        false => "never",
                    }
                },
                "--target",
                self.target.get_target_name(),
            ])
            .with_cwd(proxy.get_path())
            .with_env("PTX_CRATE_BUILDING", "1")
            .with_env("CARGO_TARGET_DIR", proxy.get_output_path())
            .with_env("RUST_TARGET_PATH", self.target.get_path());

        xargo.run().map_err(|error| match error {
            Error(ErrorKind::CommandFailed(_, _, stderr), _) => {
                let lines = stderr
                    .trim_matches('\n')
                    .split("\n")
                    .map(|item| String::from(item))
                    .collect();

                ErrorKind::BuildFailed(lines).into()
            }

            _ => error,
        })?;

        Ok(BuildStatus::Success(Output::new(
            proxy.get_output_path(),
            proxy.get_name(),
        )))
    }
}

impl Output {
    fn new(output_path: PathBuf, crate_name: &str) -> Self {
        Output {
            output_path,
            crate_name: String::from(crate_name),
        }
    }

    pub fn get_assembly_path(&self) -> PathBuf {
        self.output_path.join(format!(
            "nvptx64-nvidia-cuda/release/{}.ptx",
            self.crate_name
        ))
    }

    pub fn source_files(&self) -> Result<Vec<PathBuf>> {
        let deps_contents = {
            self.get_deps_file_contents()
                .chain_err(|| "Unable to get crate deps")?
        };

        let mut deps_parts = deps_contents.split(":");

        match deps_parts.nth(0) {
            Some(path) => {
                if path != self.get_assembly_path().to_str().unwrap() {
                    bail!(ErrorKind::InternalError(String::from(
                        "Paths misalignment in deps file"
                    )));
                }
            }

            None => {
                bail!(ErrorKind::InternalError(String::from("Empty deps file")));
            }
        }

        match deps_parts.nth(0) {
            Some(pathes) => {
                let sources = pathes
                    .trim()
                    .split(" ")
                    .map(|item| PathBuf::from(item.trim()));

                Ok(sources.collect())
            }

            None => {
                bail!(ErrorKind::InternalError(String::from("Empty deps file")));
            }
        }
    }

    fn get_deps_file_contents(&self) -> Result<String> {
        let crate_deps_path = self.output_path
            .join(format!("nvptx64-nvidia-cuda/release/{}.d", self.crate_name));

        let mut crate_deps_reader = BufReader::new(File::open(crate_deps_path)?);
        let mut crate_deps_contents = String::new();

        crate_deps_reader.read_to_string(&mut crate_deps_contents)?;

        Ok(crate_deps_contents)
    }
}

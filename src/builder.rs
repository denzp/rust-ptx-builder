use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use error::*;
use executable::{ExecutableRunner, Xargo};
use project::{Crate, Project};
use target::TargetInfo;

pub struct Builder {
    project: Project,
    target: TargetInfo,

    profile: Profile,
    colors: bool,
}

pub struct Output {
    output_path: PathBuf,
    crate_name: String,
    profile: Profile,
}

pub enum BuildStatus {
    Success(Output),
    NotNeeded,
}

#[derive(PartialEq, Clone)]
pub enum Profile {
    Debug,
    Release,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Builder {
            project: Project::analyze(path).chain_err(|| "Unable to analyze project")?,
            target: TargetInfo::new().chain_err(|| "Unable to get target details")?,

            profile: Profile::Release, // TODO: choose automatically, e.g.: `env::var("PROFILE").unwrap_or("release".to_string())`
            colors: true,
        })
    }

    pub fn is_build_needed() -> bool {
        let cargo_env = env::var("CARGO");
        let recursive_env = env::var("PTX_CRATE_BUILDING");

        let is_rls_build = cargo_env.is_ok() && cargo_env.unwrap().ends_with("rls");
        let is_recursive_build = recursive_env.is_ok() && recursive_env.unwrap() == "1";

        !is_rls_build && !is_recursive_build
    }

    pub fn disable_colors(&mut self) -> &mut Self {
        self.colors = false;
        self
    }

    pub fn set_profile(&mut self, profile: Profile) -> &mut Self {
        self.profile = profile;
        self
    }

    pub fn build(&mut self) -> Result<BuildStatus> {
        if !Self::is_build_needed() {
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
        let mut args = Vec::new();

        args.push("build");

        if self.profile == Profile::Release {
            args.push("--release");
        }

        args.push("--color");
        args.push(match self.colors {
            true => "always",
            false => "never",
        });

        args.push("--target");
        args.push(self.target.get_target_name());

        xargo
            .with_args(&args)
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
            self.profile.clone(),
        )))
    }
}

impl Output {
    fn new(output_path: PathBuf, crate_name: &str, profile: Profile) -> Self {
        Output {
            output_path,
            crate_name: String::from(crate_name),
            profile,
        }
    }

    pub fn get_assembly_path(&self) -> PathBuf {
        self.output_path.join(format!(
            "nvptx64-nvidia-cuda/{}/{}.ptx",
            self.profile, self.crate_name,
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
        let crate_deps_path = self.output_path.join(format!(
            "nvptx64-nvidia-cuda/{}/{}.d",
            self.profile, self.crate_name,
        ));

        let mut crate_deps_reader = BufReader::new(File::open(crate_deps_path)?);
        let mut crate_deps_contents = String::new();

        crate_deps_reader.read_to_string(&mut crate_deps_contents)?;

        Ok(crate_deps_contents)
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Profile::Debug => write!(f, "debug"),
            Profile::Release => write!(f, "release"),
        }
    }
}

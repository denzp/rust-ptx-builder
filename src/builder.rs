use std::path::{Path, PathBuf};

use error::*;
use project::Project;
use target::TargetInfo;
use executable::{ExecutableRunner, Xargo};

pub struct Builder {
    project: Project,
    target: TargetInfo,
}

pub struct Output {
    assembly_path: PathBuf,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Builder {
            project: Project::analyze(path).chain_err(|| "Unable to initialize project")?,
            target: TargetInfo::new().chain_err(|| "Unable to get PTX target details")?,
        })
    }

    pub fn build(self) -> Result<Output> {
        let mut xargo = ExecutableRunner::new(Xargo);

        xargo
            .with_args(&[
                "build",
                "--release",
                "--target",
                self.target.get_target_name(),
            ])
            .with_cwd(self.project.get_crate_path())
            .with_env("CARGO_TARGET_DIR", self.project.get_output_path())
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

        Ok(Output {
            assembly_path: PathBuf::from(self.project.get_output_path().join(format!(
                "nvptx64-nvidia-cuda/release/{}.ptx",
                self.project.get_name()
            ))),
        })
    }
}

impl Output {
    pub fn get_assembly_path(&self) -> &Path {
        self.assembly_path.as_path()
    }
}

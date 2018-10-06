use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use error_chain::bail;
use lazy_static::*;
use regex::Regex;

use crate::error::*;
use crate::executable::{ExecutableRunner, Xargo};
use crate::source::Crate;
use crate::target::TargetInfo;

/// Core of the crate - PTX assembly build controller.
pub struct Builder {
    source_crate: Crate,
    target: TargetInfo,

    profile: Profile,
    colors: bool,
}

/// Successful build output.
pub struct BuildOutput<'a> {
    builder: &'a Builder,
    output_path: PathBuf,
    file_suffix: String,
}

/// Non-failed build status.
pub enum BuildStatus<'a> {
    /// The CUDA crate building was performed without errors.
    Success(BuildOutput<'a>),

    /// The CUDA crate building is not needed. Can happend in several cases:
    /// - `build.rs` script was called by **RLS**,
    /// - `build.rs` was called **recursively** (e.g. `build.rs` call for device crate in single-source setup)
    NotNeeded,
}

#[derive(PartialEq, Clone)]
/// Debug / Release profile.
pub enum Profile {
    /// Equivalent for `cargo-build` **without** `--release` flag.
    Debug,

    /// Equivalent for `cargo-build` **with** `--release` flag.
    Release,
}

impl Builder {
    /// Construct a builder for device crate at `path`.
    ///
    /// Can also be the same crate, for single-source mode:
    /// ```
    /// # use ptx_builder::prelude::*;
    /// # use ptx_builder::error::*;
    /// # fn main() -> Result<()> {
    /// # std::env::set_current_dir("tests/fixtures/sample-crate")?;
    /// match Builder::new(".")?.build()? {
    ///     BuildStatus::Success(output) => {
    ///         // do something with the output...
    ///     }
    ///
    ///     BuildStatus::NotNeeded => {
    ///         // ...
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Builder {
            source_crate: Crate::analyse(path).chain_err(|| "Unable to analyse source crate")?,
            target: TargetInfo::new().chain_err(|| "Unable to get target details")?,

            profile: Profile::Release, // TODO: choose automatically, e.g.: `env::var("PROFILE").unwrap_or("release".to_string())`
            colors: true,
        })
    }

    /// Returns bool indicating whether the actual build is needed.
    ///
    /// Behavior is consistent with
    /// [`BuildStatus::NotNeeded`](enum.BuildStatus.html#variant.NotNeeded).
    pub fn is_build_needed() -> bool {
        let cargo_env = env::var("CARGO");
        let recursive_env = env::var("PTX_CRATE_BUILDING");

        let is_rls_build = cargo_env.is_ok() && cargo_env.unwrap().ends_with("rls");
        let is_recursive_build = recursive_env.is_ok() && recursive_env.unwrap() == "1";

        !is_rls_build && !is_recursive_build
    }

    /// Disable colors for internal calls to `xargo` (and eventually `cargo`).
    pub fn disable_colors(&mut self) -> &mut Self {
        self.colors = false;
        self
    }

    /// Set build profile.
    pub fn set_profile(&mut self, profile: Profile) -> &mut Self {
        self.profile = profile;
        self
    }

    /// Performs an actual build: runs `xargo` with proper flags and environment.
    pub fn build(&self) -> Result<BuildStatus> {
        if !Self::is_build_needed() {
            return Ok(BuildStatus::NotNeeded);
        }

        let mut xargo = ExecutableRunner::new(Xargo);
        let mut args = Vec::new();

        args.push("rustc");

        if self.profile == Profile::Release {
            args.push("--release");
        }

        args.push("--color");
        args.push(if self.colors { "always" } else { "never" });

        args.push("--target");
        args.push(self.target.get_target_name());

        args.push("-v");
        args.push("--");
        args.push("--crate-type");
        args.push("dylib");

        let output_path = {
            self.source_crate
                .get_output_path()
                .chain_err(|| "Unable to create output path")?
        };

        xargo
            .with_args(&args)
            .with_cwd(self.source_crate.get_path())
            .with_env("PTX_CRATE_BUILDING", "1")
            .with_env("RUSTC", "rustc-dylib-wrapper")
            .with_env("CARGO_TARGET_DIR", output_path.clone())
            .with_env("RUST_TARGET_PATH", self.target.get_path());

        let xargo_output = xargo.run().map_err(|error| match error {
            Error(ErrorKind::CommandFailed(_, _, stderr), _) => {
                let lines = stderr
                    .trim_matches('\n')
                    .split('\n')
                    .map(String::from)
                    .collect();

                ErrorKind::BuildFailed(lines).into()
            }

            _ => error,
        })?;

        Ok(BuildStatus::Success(
            self.prepare_output(output_path, &xargo_output.stderr)?,
        ))
    }

    fn prepare_output(&self, output_path: PathBuf, xargo_stderr: &str) -> Result<BuildOutput> {
        lazy_static! {
            static ref SUFFIX_REGEX: Regex =
                Regex::new(r"-C extra-filename=([\S]+)").expect("Unable to parse regex...");
        }

        let file_suffix = match SUFFIX_REGEX.captures(xargo_stderr) {
            Some(caps) => caps[1].to_string(),

            None => {
                bail!(ErrorKind::InternalError(String::from(
                    "Unable to find `extra-filename` rustc flag"
                )));
            }
        };

        Ok(BuildOutput::new(self, output_path, file_suffix))
    }
}

impl<'a> BuildOutput<'a> {
    fn new(builder: &'a Builder, output_path: PathBuf, file_suffix: String) -> Self {
        BuildOutput {
            builder,
            output_path,
            file_suffix,
        }
    }

    /// Returns path to PTX assembly file.
    ///
    /// # Usage
    /// Can be used from `build.rs` script to provide Rust with the path
    /// via environment variable:
    /// ```
    /// # use ptx_builder::prelude::*;
    /// # use ptx_builder::error::*;
    /// # fn main() -> Result<()> {
    /// # if let BuildStatus::Success(output) = Builder::new("tests/fixtures/sample-crate")?.build()? {
    /// println!(
    ///     "cargo:rustc-env=KERNEL_PTX_PATH={}",
    ///     output.get_assembly_path().display()
    /// );
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_assembly_path(&self) -> PathBuf {
        self.output_path
            .join(self.builder.target.get_target_name())
            .join(self.builder.profile.to_string())
            .join("deps")
            .join(format!(
                "{}{}.ptx",
                self.builder.source_crate.get_output_file_prefix(),
                self.file_suffix,
            ))
    }

    /// Returns a list of crate dependencies.
    ///
    /// # Usage
    /// Can be used from `build.rs` script to notify Cargo the dependencies,
    /// so it can automatically rebuild on changes:
    /// ```
    /// # use ptx_builder::prelude::*;
    /// # use ptx_builder::error::*;
    /// # fn main() -> Result<()> {
    /// # if let BuildStatus::Success(output) = Builder::new("tests/fixtures/sample-crate")?.build()? {
    /// for path in output.dependencies()? {
    ///     println!("cargo:rerun-if-changed={}", path.display());
    /// }
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub fn dependencies(&self) -> Result<Vec<PathBuf>> {
        let mut deps_contents = {
            self.get_deps_file_contents()
                .chain_err(|| "Unable to get crate deps")?
        };

        if deps_contents.is_empty() {
            bail!(ErrorKind::InternalError(String::from("Empty deps file")));
        }

        deps_contents = deps_contents
            .chars()
            .skip_while(|c| *c != ':')
            .skip(1)
            .collect::<String>();

        let cargo_deps = vec![
            self.builder.source_crate.get_path().join("Cargo.toml"),
            self.builder.source_crate.get_path().join("Cargo.lock"),
        ];

        Ok(deps_contents
            .trim()
            .split(' ')
            .map(|item| PathBuf::from(item.trim()))
            .chain(cargo_deps.into_iter())
            .collect())
    }

    fn get_deps_file_contents(&self) -> Result<String> {
        let crate_deps_path = self
            .output_path
            .join(self.builder.target.get_target_name())
            .join(self.builder.profile.to_string())
            .join(format!(
                "{}.d",
                self.builder.source_crate.get_deps_file_prefix()
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

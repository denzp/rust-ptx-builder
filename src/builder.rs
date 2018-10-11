use std::env;
use std::fmt;
use std::fs::{read_to_string, write, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use error_chain::bail;
use lazy_static::*;
use regex::Regex;

use crate::error::*;
use crate::executable::{ExecutableRunner, Xargo};
use crate::source::Crate;
use crate::target::TargetInfo;

const LAST_BUILD_CMD: &str = ".last-build-command";

/// Core of the crate - PTX assembly build controller.
pub struct Builder {
    source_crate: Crate,
    target: TargetInfo,

    profile: Profile,
    colors: bool,
    crate_type: Option<CrateType>,
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
///
/// # Usage
/// ``` no_run
/// use ptx_builder::prelude::*;
/// # use ptx_builder::error::Result;
///
/// # fn main() -> Result<()> {
/// Builder::new(".")?
///     .set_profile(Profile::Debug)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub enum Profile {
    /// Equivalent for `cargo-build` **without** `--release` flag.
    Debug,

    /// Equivalent for `cargo-build` **with** `--release` flag.
    Release,
}

#[derive(Clone, Copy)]
/// Build specified crate type.
///
/// Mandatory for mixed crates - that have both `lib.rs` and `main.rs`,
/// otherwise Cargo won't know which to build:
/// ```text
/// error: extra arguments to `rustc` can only be passed to one target, consider filtering
/// the package by passing e.g. `--lib` or `--bin NAME` to specify a single target
/// ```
///
/// # Usage
/// ``` no_run
/// use ptx_builder::prelude::*;
/// # use ptx_builder::error::Result;
///
/// # fn main() -> Result<()> {
/// Builder::new(".")?
///     .set_crate_type(CrateType::Library)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub enum CrateType {
    Library,
    Binary,
}

impl Builder {
    /// Construct a builder for device crate at `path`.
    ///
    /// Can also be the same crate, for single-source mode:
    /// ``` no_run
    /// use ptx_builder::prelude::*;
    /// # use ptx_builder::error::Result;
    ///
    /// # fn main() -> Result<()> {
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
            crate_type: None,
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
    pub fn disable_colors(mut self) -> Self {
        self.colors = false;
        self
    }

    /// Set build profile.
    pub fn set_profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }

    /// Set crate type that needs to be built.
    ///
    /// Mandatory for mixed crates - that have both `lib.rs` and `main.rs`,
    /// otherwise Cargo won't know which to build:
    /// ```text
    /// error: extra arguments to `rustc` can only be passed to one target, consider filtering
    /// the package by passing e.g. `--lib` or `--bin NAME` to specify a single target
    /// ```
    pub fn set_crate_type(mut self, crate_type: CrateType) -> Self {
        self.crate_type = Some(crate_type);
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

        match self.crate_type {
            Some(CrateType::Binary) => {
                args.push("--bin");
                args.push(self.source_crate.get_name());
            }

            Some(CrateType::Library) => {
                args.push("--lib");
            }

            _ => {}
        }

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
                    .filter(Self::output_is_not_verbose)
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

        let crate_name = self.source_crate.get_output_file_prefix();

        let build_command = {
            xargo_stderr
                .trim_matches('\n')
                .split('\n')
                .find(|line| {
                    line.contains(&format!("--crate-name {}", crate_name))
                        && line.contains("--crate-type dylib")
                })
                .map(|line| BuildCommand::Realtime(line.to_string()))
                .or_else(|| Self::load_cached_build_command(&output_path))
                .ok_or_else(|| {
                    Error::from(ErrorKind::InternalError(String::from(
                        "Unable to find build command of the device crate",
                    )))
                })?
        };

        if let BuildCommand::Realtime(ref command) = build_command {
            Self::store_cached_build_command(&output_path, &command)?;
        }

        let file_suffix = match SUFFIX_REGEX.captures(&build_command) {
            Some(caps) => caps[1].to_string(),

            None => {
                bail!(ErrorKind::InternalError(String::from(
                    "Unable to find `extra-filename` rustc flag"
                )));
            }
        };

        Ok(BuildOutput::new(self, output_path, file_suffix))
    }

    fn output_is_not_verbose(line: &&str) -> bool {
        !line.starts_with("+ ")
            && !line.contains("Running")
            && !line.contains("Fresh")
            && !line.starts_with("Caused by:")
            && !line.starts_with("  process didn\'t exit successfully: ")
    }

    fn load_cached_build_command(output_path: &Path) -> Option<BuildCommand> {
        match read_to_string(output_path.join(LAST_BUILD_CMD)) {
            Ok(contents) => Some(BuildCommand::Cached(contents)),
            Err(_) => None,
        }
    }

    fn store_cached_build_command(output_path: &Path, command: &str) -> Result<()> {
        write(output_path.join(LAST_BUILD_CMD), command.as_bytes())?;

        Ok(())
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
    /// ```no_run
    /// use ptx_builder::prelude::*;
    /// # use ptx_builder::error::Result;
    ///
    /// # fn main() -> Result<()> {
    /// if let BuildStatus::Success(output) = Builder::new(".")?.build()? {
    ///     println!(
    ///         "cargo:rustc-env=KERNEL_PTX_PATH={}",
    ///         output.get_assembly_path().display()
    ///     );
    /// }
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
    /// ```no_run
    /// use ptx_builder::prelude::*;
    /// # use ptx_builder::error::Result;
    ///
    /// # fn main() -> Result<()> {
    /// if let BuildStatus::Success(output) = Builder::new(".")?.build()? {
    ///     for path in output.dependencies()? {
    ///         println!("cargo:rerun-if-changed={}", path.display());
    ///     }
    /// }
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
            .skip(3) // workaround for Windows paths starts wuth "[A-Z]:\"
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
                self.builder
                    .source_crate
                    .get_deps_file_prefix(self.builder.crate_type)?
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

enum BuildCommand {
    Realtime(String),
    Cached(String),
}

impl std::ops::Deref for BuildCommand {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            BuildCommand::Realtime(line) => &line,
            BuildCommand::Cached(line) => &line,
        }
    }
}

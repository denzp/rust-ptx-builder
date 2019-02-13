use std::env;
use std::fmt;
use std::fs::{read_to_string, write, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use failure::ResultExt;
use lazy_static::*;
use regex::Regex;

use crate::error::*;
use crate::executable::{Cargo, ExecutableRunner, Linker};
use crate::source::Crate;

const LAST_BUILD_CMD: &str = ".last-build-command";
const TARGET_NAME: &str = "nvptx64-nvidia-cuda";

/// Core of the crate - PTX assembly build controller.
#[derive(Debug)]
pub struct Builder {
    source_crate: Crate,

    profile: Profile,
    colors: bool,
    crate_type: Option<CrateType>,
}

/// Successful build output.
#[derive(Debug)]
pub struct BuildOutput<'a> {
    builder: &'a Builder,
    output_path: PathBuf,
    file_suffix: String,
}

/// Non-failed build status.
#[derive(Debug)]
pub enum BuildStatus<'a> {
    /// The CUDA crate building was performed without errors.
    Success(BuildOutput<'a>),

    /// The CUDA crate building is not needed. Can happend in several cases:
    /// - `build.rs` script was called by **RLS**,
    /// - `build.rs` was called **recursively** (e.g. `build.rs` call for device crate in single-source setup)
    NotNeeded,
}

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
#[derive(PartialEq, Clone, Debug)]
pub enum Profile {
    /// Equivalent for `cargo-build` **without** `--release` flag.
    Debug,

    /// Equivalent for `cargo-build` **with** `--release` flag.
    Release,
}

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
#[derive(Clone, Copy, Debug)]
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
            source_crate: Crate::analyse(path).context("Unable to analyse source crate")?,

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

    /// Disable colors for internal calls to `cargo`.
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

    /// Performs an actual build: runs `cargo` with proper flags and environment.
    pub fn build(&self) -> Result<BuildStatus> {
        if !Self::is_build_needed() {
            return Ok(BuildStatus::NotNeeded);
        }

        // Verify `ptx-linker` version.
        ExecutableRunner::new(Linker).with_args(vec!["-V"]).run()?;

        let mut cargo = ExecutableRunner::new(Cargo);
        let mut args = Vec::new();

        args.push("rustc");

        if self.profile == Profile::Release {
            args.push("--release");
        }

        args.push("--color");
        args.push(if self.colors { "always" } else { "never" });

        args.push("--target");
        args.push(TARGET_NAME);

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
        args.push("cdylib");
        args.push("-Zcrate-attr=no_main");

        let output_path = {
            self.source_crate
                .get_output_path()
                .context("Unable to create output path")?
        };

        cargo
            .with_args(&args)
            .with_cwd(self.source_crate.get_path())
            .with_env("PTX_CRATE_BUILDING", "1")
            .with_env("CARGO_TARGET_DIR", output_path.clone());

        let cargo_output = cargo.run().map_err(|error| match error.kind() {
            BuildErrorKind::CommandFailed { stderr, .. } => {
                let lines = stderr
                    .trim_matches('\n')
                    .split('\n')
                    .filter(Self::output_is_not_verbose)
                    .map(String::from)
                    .collect();

                Error::from(BuildErrorKind::BuildFailed(lines))
            }

            _ => error,
        })?;

        Ok(BuildStatus::Success(
            self.prepare_output(output_path, &cargo_output.stderr)?,
        ))
    }

    fn prepare_output(&self, output_path: PathBuf, cargo_stderr: &str) -> Result<BuildOutput> {
        lazy_static! {
            static ref SUFFIX_REGEX: Regex =
                Regex::new(r"-C extra-filename=([\S]+)").expect("Unable to parse regex...");
        }

        let crate_name = self.source_crate.get_output_file_prefix();

        // We need the build command to get real output filename.
        let build_command = {
            cargo_stderr
                .trim_matches('\n')
                .split('\n')
                .find(|line| {
                    line.contains(&format!("--crate-name {}", crate_name))
                        && line.contains("--crate-type cdylib")
                })
                .map(|line| BuildCommand::Realtime(line.to_string()))
                .or_else(|| Self::load_cached_build_command(&output_path))
                .ok_or_else(|| {
                    Error::from(BuildErrorKind::InternalError(String::from(
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
                bail!(BuildErrorKind::InternalError(String::from(
                    "Unable to find `extra-filename` rustc flag",
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
        write(output_path.join(LAST_BUILD_CMD), command.as_bytes())
            .context(BuildErrorKind::OtherError)?;

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
            .join(TARGET_NAME)
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
                .context("Unable to get crate deps")?
        };

        if deps_contents.is_empty() {
            bail!(BuildErrorKind::InternalError(String::from(
                "Empty deps file",
            )));
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
            .join(TARGET_NAME)
            .join(self.builder.profile.to_string())
            .join(format!(
                "{}.d",
                self.builder
                    .source_crate
                    .get_deps_file_prefix(self.builder.crate_type)?
            ));

        let mut crate_deps_reader =
            BufReader::new(File::open(crate_deps_path).context(BuildErrorKind::OtherError)?);

        let mut crate_deps_contents = String::new();

        crate_deps_reader
            .read_to_string(&mut crate_deps_contents)
            .context(BuildErrorKind::OtherError)?;

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

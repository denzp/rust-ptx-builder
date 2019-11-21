use std::fmt;
use std::path::PathBuf;

use colored::*;
use failure::{Backtrace, Context, Fail};
use semver::{Version, VersionReq};

#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    inner: Context<BuildErrorKind>,
}

#[derive(Debug, PartialEq, Fail, Clone)]
pub enum BuildErrorKind {
    CommandNotFound {
        command: String,
        hint: String,
    },

    CommandFailed {
        command: String,
        code: i32,
        stderr: String,
    },
    CommandVersionNotFulfilled {
        command: String,
        current: Version,
        required: VersionReq,
        hint: String,
    },

    InvalidCratePath(PathBuf),
    BuildFailed(Vec<String>),
    InvalidCrateType(String),
    MissingCrateType,
    InternalError(String),
    OtherError,
}

impl Fail for Error {
    fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, formatter)
    }
}

impl Error {
    pub fn kind(&self) -> BuildErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<BuildErrorKind> for Error {
    fn from(kind: BuildErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<BuildErrorKind>> for Error {
    fn from(inner: Context<BuildErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<Context<String>> for Error {
    fn from(inner: Context<String>) -> Error {
        Error {
            inner: inner.map(BuildErrorKind::InternalError),
        }
    }
}

impl<'a> From<Context<&'a str>> for Error {
    fn from(inner: Context<&'a str>) -> Error {
        Self::from(inner.map(String::from))
    }
}

impl fmt::Display for BuildErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use BuildErrorKind::*;

        match self {
            CommandNotFound { command, hint } => write!(
                formatter,
                "Command not found in PATH: '{}'. {}.",
                command.bold(),
                hint.underline()
            ),

            CommandFailed {
                command,
                code,
                stderr,
            } => write!(
                formatter,
                "Command failed: '{}' with code '{}' and output:\n{}",
                command.bold(),
                code,
                stderr.trim(),
            ),

            CommandVersionNotFulfilled {
                command,
                current,
                required,
                hint,
            } => write!(
                formatter,
                "Command version is not fulfilled: '{}' is currently '{}' but '{}' is required. {}.",
                command.bold(),
                current.to_string().underline(),
                required.to_string().underline(),
                hint.underline(),
            ),

            InvalidCratePath(path) => write!(
                formatter,
                "{}: {}",
                "Invalid device crate path".bold(),
                path.display()
            ),

            BuildFailed(lines) => write!(
                formatter,
                "{}\n{}",
                "Unable to build a PTX crate!".bold(),
                lines.join("\n")
            ),

            InvalidCrateType(crate_type) => write!(
                formatter,
                "{}: the crate cannot be build as '{}'",
                "Impossible CrateType".bold(),
                crate_type
            ),

            MissingCrateType => write!(
                formatter,
                "{}: it's mandatory for mixed-type crates",
                "Missing CrateType".bold()
            ),

            InternalError(message) => write!(formatter, "{}: {}", "Internal error".bold(), message),
            OtherError => write!(formatter, "Other error"),
        }
    }
}

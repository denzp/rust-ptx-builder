use semver::{Version, VersionReq};

use crate::error::*;

pub mod runner;
pub use self::runner::{ExecutableRunner, Output};

/// Details and requirements for executables.
pub trait Executable {
    /// Returns executable name in `PATH`.
    fn get_name(&self) -> String;

    /// Returns message about how to install missing executable.
    fn get_verification_hint(&self) -> String;

    /// Returns message about how to update outdated executable.
    fn get_version_hint(&self) -> String;

    /// Executable version constraint.
    fn get_required_version(&self) -> Option<VersionReq>;

    /// Returns the current version of the executable.
    fn get_current_version(&self) -> Result<Version>
    where
        Self: Sized,
    {
        self::runner::parse_executable_version(self)
    }
}

/// `cargo` command.
pub struct Cargo;

/// `xargo` command.
pub struct Xargo;

/// `ptx-linker` command.
pub struct Linker;

impl Executable for Cargo {
    fn get_name(&self) -> String {
        String::from("cargo")
    }

    fn get_verification_hint(&self) -> String {
        String::from("Please make sure you have it installed and in PATH")
    }

    fn get_version_hint(&self) -> String {
        String::from("Please update Rust and Cargo to latest nightly versions")
    }

    fn get_required_version(&self) -> Option<VersionReq> {
        Some(VersionReq::parse(">= 1.30.0-nightly").unwrap())
    }

    fn get_current_version(&self) -> Result<Version> {
        // Omit Rust channel name because it's not really semver-correct
        // https://github.com/steveklabnik/semver/issues/105

        self::runner::parse_executable_version(self).map(|mut version| {
            version.pre = vec![];
            version
        })
    }
}

impl Executable for Linker {
    fn get_name(&self) -> String {
        String::from("ptx-linker")
    }

    fn get_verification_hint(&self) -> String {
        String::from("You can install it with: 'cargo install ptx-linker'")
    }

    fn get_version_hint(&self) -> String {
        String::from("You can update it with: 'cargo install -f ptx-linker'")
    }

    fn get_required_version(&self) -> Option<VersionReq> {
        Some(VersionReq::parse(">= 0.8.1").unwrap())
    }
}

impl Executable for Xargo {
    fn get_name(&self) -> String {
        String::from("xargo")
    }

    fn get_verification_hint(&self) -> String {
        String::from("You can install it with: 'cargo install xargo'")
    }

    fn get_version_hint(&self) -> String {
        String::from("You can update it with: 'cargo install -f xargo'")
    }

    fn get_required_version(&self) -> Option<VersionReq> {
        Some(VersionReq::parse(">= 0.3.12").unwrap())
    }
}

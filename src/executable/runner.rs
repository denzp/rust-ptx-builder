use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use error_chain::bail;
use regex::Regex;
use semver::Version;

use super::Executable;
use crate::error::*;

pub struct ExecutableRunner<Ex: Executable> {
    command: Command,
    executable: Ex,
}

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

impl<Ex: Executable> ExecutableRunner<Ex> {
    pub fn new(executable: Ex) -> Self {
        ExecutableRunner {
            command: Command::new(executable.get_name()),
            executable,
        }
    }

    pub fn with_args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    pub fn with_env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }

    pub fn with_cwd<P>(&mut self, path: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.command.current_dir(path);
        self
    }

    pub fn run(&mut self) -> Result<Output> {
        self.check_version()?;

        let raw_output = {
            self.command.output().chain_err(|| {
                ErrorKind::InternalError(format!(
                    "Unable to execute command '{}'",
                    self.executable.get_name()
                ))
            })?
        };

        let output = Output {
            stdout: String::from_utf8(raw_output.stdout)?,
            stderr: String::from_utf8(raw_output.stderr)?,
        };

        if raw_output.status.success() {
            Ok(output)
        } else {
            bail!(ErrorKind::CommandFailed(
                self.executable.get_name(),
                raw_output.status.code().unwrap_or(-1),
                output.stderr,
            ))
        }
    }

    fn check_version(&self) -> Result<()> {
        let current = self.executable.get_current_version()?;
        let required = self.executable.get_required_version();

        match required {
            Some(ref required) if !required.matches(&current) => {
                bail!(ErrorKind::CommandVersionNotFulfilled(
                    self.executable.get_name(),
                    current,
                    required.clone(),
                    self.executable.get_version_hint()
                ));
            }

            _ => Ok(()),
        }
    }
}

pub(crate) fn parse_executable_version<E: Executable>(executable: &E) -> Result<Version> {
    let mut command = Command::new(executable.get_name());

    command.args(&["-V"]);

    let raw_output = {
        command.output().chain_err(|| {
            ErrorKind::CommandNotFound(executable.get_name(), executable.get_verification_hint())
        })?
    };

    let output = Output {
        stdout: String::from_utf8(raw_output.stdout)?,
        stderr: String::from_utf8(raw_output.stderr)?,
    };

    if !raw_output.status.success() {
        bail!(ErrorKind::CommandFailed(
            executable.get_name(),
            raw_output.status.code().unwrap_or(-1),
            output.stderr,
        ));
    }

    let version_regex = Regex::new(&format!(r"{}\s(\S+)", executable.get_name()))?;

    match version_regex.captures(&(output.stdout + &output.stderr)) {
        Some(captures) => Ok(Version::parse(&captures[1])?),

        None => bail!(ErrorKind::InternalError(
            "Unable to find executable version".into()
        )),
    }
}

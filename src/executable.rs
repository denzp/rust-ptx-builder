use std::path::Path;
use std::ffi::OsStr;
use std::process::Command;

use error::*;

pub trait Executable {
    fn get_name(&self) -> String;
    fn get_verification_hint(&self) -> String;

    fn populate_verification_args(&self, &mut Command);
}

pub struct Cargo;
pub struct Xargo;
pub struct Linker;

pub struct ExecutableRunner<Ex: Executable> {
    command: Command,
    executable: Ex,
}

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

    pub fn is_runnable(&self) -> bool {
        let mut command = Command::new(self.executable.get_name());

        self.executable.populate_verification_args(&mut command);

        match command.output() {
            Ok(_) => true,
            Err(_) => false,
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
        if !self.is_runnable() {
            bail!(ErrorKind::CommandNotFound(
                self.executable.get_name(),
                self.executable.get_verification_hint()
            ));
        }

        let raw_output = {
            self.command
                .output()
                .chain_err(|| "Unable to execute the command")?
        };

        let output = Output {
            stdout: String::from_utf8(raw_output.stdout)?,
            stderr: String::from_utf8(raw_output.stderr)?,
        };

        match raw_output.status.success() {
            true => Ok(output),

            false => bail!(ErrorKind::CommandFailed(
                self.executable.get_name(),
                raw_output.status.code().unwrap_or(-1),
                output.stderr,
            )),
        }
    }
}

impl Executable for Cargo {
    fn get_name(&self) -> String {
        String::from("cargo")
    }

    fn get_verification_hint(&self) -> String {
        String::from("Please make sure you have it installed and in PATH")
    }

    fn populate_verification_args(&self, command: &mut Command) {
        command.args(&["-V"]);
    }
}

impl Executable for Linker {
    fn get_name(&self) -> String {
        String::from("ptx-linker")
    }

    fn get_verification_hint(&self) -> String {
        String::from("You can install it with: 'cargo install ptx-linker'")
    }

    fn populate_verification_args(&self, command: &mut Command) {
        command.args(&["-V"]);
    }
}

impl Executable for Xargo {
    fn get_name(&self) -> String {
        String::from("xargo")
    }

    fn get_verification_hint(&self) -> String {
        String::from("You can install it with: 'cargo install xargo'")
    }

    fn populate_verification_args(&self, command: &mut Command) {
        command.args(&["-V"]);
    }
}

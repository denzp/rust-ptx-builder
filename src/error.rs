use std::path::PathBuf;

use colored::*;
use error_chain::*;
use semver::{Version, VersionReq};

error_chain! {
    errors {
        CommandNotFound(command: String, hint: String) {
            display("Command not found in PATH: '{}'. {}.", command.bold(), hint.underline()),
        }

        CommandFailed(command: String, code: i32, stderr: String) {
            display("Command failed: '{}' with code '{}' and output:\n{}", command.bold(), code, stderr.trim()),
        }

        CommandVersionNotFulfilled(command: String, current: Version, required: VersionReq, hint: String) {
            display(
                "Command version is not fulfilled: '{}' is currently '{}' but '{}' is required. {}.",
                command.bold(),
                current.to_string().underline(),
                required.to_string().underline(),
                hint.underline(),
            )
        }

        InvalidCratePath(path: PathBuf) {
            display("{}: {}", "Invalid device crate path".bold(), path.display()),
        }

        BuildFailed(diagnostics: Vec<String>) {
            display("{}\n{}", "Unable to build a PTX crate!".bold(), diagnostics.join("\n")),
        }

        InvalidCrateType(crate_type: String) {
            display("{}: the crate cannot be build as '{}'", "Impossible CrateType".bold(), crate_type),
        }

        MissingCrateType {
            display("{}: it's mandatory for mixed-type crates", "Missing CrateType".bold()),
        }

        InternalError(reason: String) {
            display("{}: {}", "Internal error".bold(), reason),
        }
    }

    foreign_links {
        Utf8Error(::std::string::FromUtf8Error);
        Io(::std::io::Error);
        TomlError(::toml::de::Error);
        RegexError(::regex::Error);
        SemVerError(::semver::SemVerError);
    }
}

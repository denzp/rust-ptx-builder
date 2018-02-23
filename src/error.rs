use std::path::PathBuf;
use colored::*;

error_chain! {
    errors {
        CommandNotFound(command: String, hint: String) {
            display("Command not found in PATH: '{}'. {}.", command.bold(), hint.underline()),
        }

        CommandFailed(command: String, code: i32, stderr: String) {
            display("Command failed: '{}' with code '{}' and output:\n{}", command.bold(), code, stderr.trim()),
        }

        InvalidCratePath(path: PathBuf) {
            display("{}: {}", "Invalid device crate path".bold(), path.to_str().unwrap()),
        }

        BuildFailed(diagnostics: Vec<String>) {
            display("{}\n{}", "Unable to build a PTX crate!".bold(), diagnostics.join("\n")),
        }

        InternalError(reason: String) {
            display("{}: {}", "Internal error".bold(), reason),
        }
    }

    foreign_links {
        Utf8Error(::std::string::FromUtf8Error);
        Io(::std::io::Error);
        TomlError(::toml::de::Error);
    }
}

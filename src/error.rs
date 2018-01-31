use colored::*;

error_chain! {
    errors {
        CommandNotFound(command: String, hint: String) {
            display("Command not found in PATH: '{}'. {}.", command.bold(), hint.underline()),
        }

        CommandFailed(command: String, code: i32, stderr: String) {
            display("Command failed: '{}' with code '{}' and output:\n{}", command.bold(), code, stderr.trim()),
        }

        BuildFailed(diagnostics: Vec<String>) {
            display("{}\n{}", "Unable to build a PTX crate!".bold(), diagnostics.join("\n")),
        }
    }

    foreign_links {
        Utf8Error(::std::string::FromUtf8Error);
        Io(::std::io::Error);
    }
}

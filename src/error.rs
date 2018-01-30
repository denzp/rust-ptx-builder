error_chain! {
    errors {
        CommandNotFound(command: String, hint: String) {
            display("Command not found in PATH: '{}'. {}.", command, hint),
        }

        CommandFailed(command: String, code: i32, stderr: String) {
            display("Command failed: '{}', code {}.", command, code),
        }

        BuildFailed(diagnostics: Vec<String>) {
            display("Build failed:\n{}", diagnostics.join("\n")),
        }
    }

    foreign_links {
        Utf8Error(::std::string::FromUtf8Error);
        Io(::std::io::Error);
    }
}

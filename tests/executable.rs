extern crate ptx_builder;

use std::process::Command;

use ptx_builder::executable::{Cargo, Executable, ExecutableRunner};
use ptx_builder::error::*;

mod cargo {
    use super::*;

    #[test]
    fn should_be_runnable() {
        assert_eq!(ExecutableRunner::new(Cargo).is_runnable(), true);
    }

    #[test]
    fn should_provide_output() {
        let output = ExecutableRunner::new(Cargo)
            .with_args(&["rustc", "-q", "--", "--print", "crate-name"])
            .with_cwd("tests/fixtures/sample-crate")
            .run();

        assert_eq!(output.is_ok(), true);
        assert_eq!(output.unwrap().stdout, String::from("sample_ptx_crate\n"));
    }

    #[test]
    fn should_check_exit_code() {
        let output = ExecutableRunner::new(Cargo)
            .with_args(&["rustc", "-q", "--unknow-flag"])
            .with_cwd("tests/fixtures/sample-crate")
            .run();

        match output {
            Err(Error(ErrorKind::CommandFailed(command, code, stderr), _)) => {
                assert_eq!(command, String::from("cargo"));
                assert_eq!(code, 1);
                assert_eq!(stderr, String::from("error: Unknown flag: '--unknow-flag'\n\nUsage:\n    cargo rustc [options] [--] [<opts>...]\n"));
            }

            Ok(_) => unreachable!("it should fail"),
            Err(_) => unreachable!("it should fail with proper error"),
        }
    }
}

mod non_existing_command {
    use super::*;

    struct NonExistingCommand;

    impl Executable for NonExistingCommand {
        fn get_name(&self) -> String {
            String::from("almost-unique-name")
        }

        fn get_verification_hint(&self) -> String {
            String::from("Some useful hint")
        }

        fn populate_verification_args(&self, command: &mut Command) {
            command.args(&["-V"]);
        }
    }

    #[test]
    fn should_not_be_runnable() {
        assert_eq!(
            ExecutableRunner::new(NonExistingCommand).is_runnable(),
            false
        );
    }

    #[test]
    fn should_not_provide_output() {
        let output = ExecutableRunner::new(NonExistingCommand).run();

        match output {
            Err(Error(ErrorKind::CommandNotFound(command, hint), _)) => {
                assert_eq!(command, String::from("almost-unique-name"));
                assert_eq!(hint, String::from("Some useful hint"));
            }

            Ok(_) => unreachable!("it should fail"),
            Err(_) => unreachable!("it should fail with proper error"),
        }
    }

}

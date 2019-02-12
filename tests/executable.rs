use semver::VersionReq;

use ptx_builder::error::*;
use ptx_builder::executable::{Cargo, Executable, ExecutableRunner};

mod cargo {
    use super::*;

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
            .with_args(&["rustc", "-q", "--unknown-flag"])
            .with_cwd("tests/fixtures/sample-crate")
            .run();

        match output.unwrap_err().kind() {
            BuildErrorKind::CommandFailed {
                command,
                code,
                stderr,
            } => {
                assert_eq!(command, String::from("cargo"));
                assert_eq!(code, 1);

                assert!(stderr.contains("argument '--unknown-flag'"));
            }

            _ => unreachable!("it should fail with proper error"),
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

        fn get_version_hint(&self) -> String {
            String::from("Some useful hint about version")
        }

        fn get_required_version(&self) -> Option<VersionReq> {
            None
        }
    }

    #[test]
    fn should_not_provide_output() {
        let output = ExecutableRunner::new(NonExistingCommand).run();

        match output.unwrap_err().kind() {
            BuildErrorKind::CommandNotFound { command, hint } => {
                assert_eq!(command, String::from("almost-unique-name"));
                assert_eq!(hint, String::from("Some useful hint"));
            }

            _ => unreachable!("it should fail with proper error"),
        }
    }
}

mod unrealistic_version_requirement {
    use super::*;

    struct UnrealisticCommand;

    impl Executable for UnrealisticCommand {
        fn get_name(&self) -> String {
            String::from("cargo")
        }

        fn get_verification_hint(&self) -> String {
            String::from("Some useful hint")
        }

        fn get_version_hint(&self) -> String {
            String::from("Some useful hint about version")
        }

        fn get_required_version(&self) -> Option<VersionReq> {
            Some(VersionReq::parse("> 100.0.0").unwrap())
        }
    }

    #[test]
    fn should_not_provide_output() {
        let output = ExecutableRunner::new(UnrealisticCommand).run();

        match output.unwrap_err().kind() {
            BuildErrorKind::CommandVersionNotFulfilled {
                command,
                required,
                hint,
                ..
            } => {
                assert_eq!(command, String::from("cargo"));
                assert_eq!(required, VersionReq::parse("> 100.0.0").unwrap());
                assert_eq!(hint, String::from("Some useful hint about version"));
            }

            _ => unreachable!("it should fail with proper error"),
        }
    }
}

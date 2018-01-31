extern crate colored;
extern crate ptx_builder;

use ptx_builder::error::*;
use ptx_builder::reporter::CargoBuildReporter;

#[test]
fn should_report_in_cargo_style() {
    let original_error: Result<()> = Err(ErrorKind::CommandFailed(
        String::from("some_name"),
        0,
        String::from("some\nmultiline\noutput"),
    ).into());

    let chained_error = original_error.chain_err(|| {
        ErrorKind::BuildFailed(vec![
            String::from("error[E0425]: cannot find function `external_fn` in this scope"),
            String::from(" --> src/lib.rs:6:20"),
            String::from("  |"),
            String::from("6 |     *y.offset(0) = external_fn(*x.offset(0)) * a;"),
            String::from("  |                    ^^^^^^^^^^^ not found in this scope"),
        ])
    });

    let mut reporter = CargoBuildReporter::report(chained_error.unwrap_err());

    assert_eq!(
        reporter.disable_colors().to_string(),
        "cargo:warning=\r              \r[PTX] Unable to build a PTX crate!
cargo:warning=\r              \r[PTX] error[E0425]: cannot find function `external_fn` in this scope
cargo:warning=\r              \r[PTX]  --> src/lib.rs:6:20
cargo:warning=\r              \r[PTX]   |
cargo:warning=\r              \r[PTX] 6 |     *y.offset(0) = external_fn(*x.offset(0)) * a;
cargo:warning=\r              \r[PTX]   |                    ^^^^^^^^^^^ not found in this scope
cargo:warning=\r              \r[PTX]
cargo:warning=\r              \r[PTX] caused by:
cargo:warning=\r              \r[PTX]   Command failed: 'some_name' with code '0' and output:
cargo:warning=\r              \r[PTX]   some
cargo:warning=\r              \r[PTX]   multiline
cargo:warning=\r              \r[PTX]   output
"
    );
}

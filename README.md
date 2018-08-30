# Rust PTX Builder
[![Build Status](https://travis-ci.org/denzp/rust-ptx-builder.svg?branch=master)](https://travis-ci.org/denzp/rust-ptx-builder)
[![Build status](https://ci.appveyor.com/api/projects/status/5m0du8548xh1fjph/branch/master?svg=true)](https://ci.appveyor.com/project/denzp/rust-ptx-builder/branch/master)
[![Current Version](https://img.shields.io/crates/v/ptx-builder.svg)](https://crates.io/crates/ptx-builder)

## Purpose
The library should facilitate CUDA development with Rust.
It can be used in a [cargo build script](http://doc.crates.io/build-script.html) of a host crate, and take responsibility for building device crates.

## Features
1. Obviously, device crates building.
2. Announcing device crates sources to cargo, so it can automatically rebuild after changes.
3. Reporting about missing tools, for example:
```
[PTX] Unable to get target details
[PTX]
[PTX] caused by:
[PTX]   Command not found in PATH: 'ptx-linker'. You can install it with: 'cargo install ptx-linker'.
```

## Prerequirements
The library depends on [ptx-linker](https://crates.io/crates/ptx-linker) and [xargo](https://crates.io/crates/xargo).
Both can be installed from crates.io:
```
cargo install xargo
cargo install ptx-linker
```

## Usage
First, you need to specify a build script in host crate's `Cargo.toml` and declare the library as a *build-dependency*:
``` toml
[package]
build = "build.rs"

[build-dependencies]
ptx-builder = "0.3"
```

Then, typical `build.rs` might look like:
``` rust
extern crate ptx_builder;

use std::process::exit;
use ptx_builder::prelude::*;

fn main() {
    if let Err(error) = build() {
        eprintln!("{}", BuildReporter::report(error));
        exit(1);
    }
}

fn build() -> Result<()> {
    let status = Builder::new(".")?.build()?;

    match status {
        BuildStatus::Success(output) => {
            // Provide the PTX Assembly location via env variable
            println!(
                "cargo:rustc-env=KERNEL_PTX_PATH={}",
                output.get_assembly_path().to_str().unwrap()
            );

            // Observe changes in kernel sources
            for path in output.source_files()? {
                println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
            }
        }

        BuildStatus::NotNeeded => {
            println!("cargo:rustc-env=KERNEL_PTX_PATH=/dev/null");
        }
    };

    Ok(())
}

```

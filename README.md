# Rust PTX Builder
[![Build Status](https://travis-ci.org/denzp/rust-ptx-builder.svg?branch=master)](https://travis-ci.org/denzp/rust-ptx-builder)
[![Current Version](https://img.shields.io/crates/v/ptx-builder.svg)](https://crates.io/crates/ptx-builder)
[![Docs](https://docs.rs/ptx-builder/badge.svg)](https://docs.rs/ptx-builder)

## New Release: 0.5 🎉
### Say goodbye to proxy crate approach
This allows us to use single-source CUDA in **binary**-only crates (ones without `lib.rs`).

### Development breaking changes
The crate does not provide a default `panic_handler` anymore.
From now on, it either up to a user, or other crates (e.g. coming soon [`ptx-support` crate](https://github.com/denzp/rust-ptx-support)).

Next workaround should work in common cases,
although it doesn't provide any panic details in runtime:
``` rust
#![feature(core_intrinsics)]

#[panic_handler]
unsafe fn breakpoint_panic_handler(_: &::core::panic::PanicInfo) -> ! {
    core::intrinsics::breakpoint();
    core::hint::unreachable_unchecked();
}
```

### API Breaking Changes - less boilerplate code
`build.rs` script was never so compact and clear before:
``` rust
use ptx_builder::error::Result;
use ptx_builder::prelude::*;

fn main() -> Result<()> {
    let builder = Builder::new(".")?;
    CargoAdapter::with_env_var("KERNEL_PTX_PATH").build(builder);
}
```

### Documentation improvements
This release comes with a significant documentation improvement! [Check it out](https://docs.rs/ptx-builder) :)

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
[PTX]   Command not found in PATH: 'rust-ptx-linker'. You can install it with: 'cargo install ptx-linker'.
```

## Prerequirements
The library depends on a fresh Nightly and [ptx-linker](https://crates.io/crates/ptx-linker).
The latter can be installed from crates.io:
```
cargo install ptx-linker
```

## Usage
First, you need to specify a build script in host crate's `Cargo.toml` and declare the library as a *build-dependency*:
``` toml
[build-dependencies]
ptx-builder = "0.5"
```

Then, typical `build.rs` might look like:
``` rust
use ptx_builder::error::Result;
use ptx_builder::prelude::*;

fn main() -> Result<()> {
    let builder = Builder::new(".")?;
    CargoAdapter::with_env_var("KERNEL_PTX_PATH").build(builder);
}
```

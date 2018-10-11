# Rust PTX Builder
[![Build Status](https://travis-ci.org/denzp/rust-ptx-builder.svg?branch=master)](https://travis-ci.org/denzp/rust-ptx-builder)
[![Build status](https://ci.appveyor.com/api/projects/status/5m0du8548xh1fjph/branch/master?svg=true)](https://ci.appveyor.com/project/denzp/rust-ptx-builder/branch/master)
[![Current Version](https://img.shields.io/crates/v/ptx-builder.svg)](https://crates.io/crates/ptx-builder)
[![Docs](https://img.shields.io/badge/docs-master-blue.svg)](https://denzp.github.io/rust-ptx-builder/master/ptx_builder/index.html)

## New Release: 0.5 🎉
### Say goodbye to proxy crate approach
This allows us to use single-source CUDA in **binary**-only crates (ones without `lib.rs`).
New approach might seem a bit hacky with overriding Cargo behavior and enforcing `--crate-type dylib`, but in the end, development workflow became much more convinient.

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
    CargoAdapter::with_env_var("KERNEL_PTX_PATH").build(Builder::new(".")?);
}
```

### Documentation improvements
This release comes with a significant documentation improvement! [Check it out](https://denzp.github.io/rust-ptx-builder/master/ptx_builder/index.html) :)

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

### Windows users!
Unfortunately, due to [rustc-llvm-proxy#1](/denzp/rustc-llvm-proxy/issues/1) **MSVS** targets are not supported yet.

You might face similar errors:
```
Unable to find symbol 'LLVMContextCreate' in the LLVM shared lib
```

For now the only solution is to use **GNU** targets.

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
    CargoAdapter::with_env_var("KERNEL_PTX_PATH").build(Builder::new(".")?);
}
```

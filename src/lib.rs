#![deny(warnings)]
#![warn(clippy::all)]

//! `build.rs` helper crate for your CUDA experiments.
//!
//! It helps to automatically build device crate in both *single-source* and *separated-source* projects.
//!
//! Features the crate provide:
//! * Automatically notify Cargo about device crate sources, so it can reuild on changes,
//! * Provide output PTX assembly path to Rust via environment variable,
//! * Rich reporting of device crate errors,
//! * Hints and troubleshooting for missing tools.
//!
//! # Usage
//! Simply add the crate as `build-dependency`:
//! ```text
//! [build-dependencies]
//! ptx-builder = "0.5"
//! ```
//!
//! And start using it in `build.rs` script:
//! ```no_run
//! use ptx_builder::error::Result;
//! use ptx_builder::prelude::*;
//!
//! fn main() -> Result<()> {
//!     let builder = Builder::new(".")?;
//!     CargoAdapter::with_env_var("KERNEL_PTX_PATH").build(builder);
//! }
//! ```
//!
//! Now, on the host-side, the PTX assembly can be loaded and used with your favorite CUDA driver crate:
//! ```ignore
//! use std::ffi::CString;
//!
//! let ptx = CString::new(include_str!(env!("KERNEL_PTX_PATH")))?;
//!
//! // use the assembly contents ...
//! ```

/// Error handling.
#[macro_use]
pub mod error;

/// External executables that are needed to build CUDA crates.
pub mod executable;

/// Build helpers.
pub mod builder;

/// Build reporting helpers.
pub mod reporter;

mod source;

/// Convenient re-exports of mostly used types.
pub mod prelude {
    pub use crate::builder::{BuildStatus, Builder, CrateType, Profile};
    pub use crate::reporter::{CargoAdapter, ErrorLogPrinter};
}

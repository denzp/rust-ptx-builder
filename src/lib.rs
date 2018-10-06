#![deny(warnings)]

/// Error handling.
pub mod error;

/// External executables that are needed to build CUDA crates.
pub mod executable;

/// Build helpers.
pub mod builder;

/// Build reporting helpers.
pub mod reporter;

mod source;
mod target;

/// Convenient re-exports of mostly used types.
pub mod prelude {
    pub use crate::builder::{BuildStatus, Builder, Profile};
    pub use crate::error::Result;
    pub use crate::reporter::BuildReporter;
}

#![deny(warnings)]

pub mod error;
pub mod executable;

pub mod builder;
pub mod reporter;
pub mod source;
pub mod target;

pub mod prelude {
    pub use crate::builder::{BuildStatus, Builder};
    pub use crate::error::Result;
    pub use crate::reporter::BuildReporter;
}

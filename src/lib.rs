#![deny(warnings)]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate colored;
extern crate regex;
extern crate semver;
extern crate toml;

pub mod error;
pub mod executable;

pub mod builder;
pub mod reporter;
pub mod source;
pub mod target;

pub mod prelude {
    pub use builder::{BuildStatus, Builder};
    pub use error::Result;
    pub use reporter::BuildReporter;
}

#![deny(warnings)]

#[macro_use]
extern crate error_chain;

extern crate colored;
extern crate toml;

pub mod error;
pub mod executable;

pub mod project;
pub mod builder;
pub mod target;
pub mod proxy;
pub mod reporter;

pub mod prelude {
    pub use builder::Builder;
    pub use reporter::BuildReporter;
    pub use error::Result;
}

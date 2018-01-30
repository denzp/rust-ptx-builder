#![deny(warnings)]

#[macro_use]
extern crate error_chain;

pub mod error;
pub mod executable;

pub mod project;
pub mod builder;
pub mod target;

pub mod prelude {
    pub use builder::Builder;
}

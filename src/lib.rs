#![deny(unused_must_use)]
#![allow(clippy::ptr_arg)]

pub mod cli;
mod core;
pub mod utils;

// Exports
pub use core::install::{default_install_dir, EnvConfig, InstallConfiguration};
pub use core::parser::manifest;
pub use core::try_it::try_it;

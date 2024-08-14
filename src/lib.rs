#![deny(unused_must_use)]

pub mod cli;
mod core;
pub mod utils;

// Exports
pub use core::install::{default_install_dir, InstallConfiguration};
pub use core::parser::manifest;
pub use core::EnvConfig;

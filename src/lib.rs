#![deny(unused_must_use)]
#![allow(clippy::ptr_arg)]

#[macro_use]
extern crate rust_i18n;

pub mod cli;
mod core;
pub mod utils;

// Exports
pub use core::components::{get_component_list_from_manifest, Component};
pub use core::install::{default_install_dir, EnvConfig, InstallConfiguration};
pub use core::parser::manifest;
pub use core::try_it::try_it;

i18n!("locales", fallback = "en");

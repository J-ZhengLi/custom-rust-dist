#![deny(unused_must_use)]
#![allow(clippy::ptr_arg)]

#[macro_use]
extern crate rust_i18n;

pub mod cli;
mod core;
pub mod utils;

// Exports
pub use core::components;
pub use core::install::{default_install_dir, EnvConfig, InstallConfiguration};
pub use core::parser::{fingerprint, get_installed_dir, toolset_manifest};
pub use core::try_it::try_it;
pub use core::uninstall::UninstallConfiguration;

i18n!("locales", fallback = "en");

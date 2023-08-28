// TODO: warn unused before release
#![allow(unused)]

mod defaults;
mod parser;
mod steps;
mod utils;

#[cfg(feature = "iced-gui")]
pub mod app;
#[cfg(feature = "cli")]
pub mod cli;
pub mod mini_rustup;

mod applog {
    pub use logger::{debug, err, info, trace, warn};
}

use anyhow::Result;
use cfg_if::cfg_if;
use parser::Configuration;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use logger::Logger;

pub(crate) const APPNAME: &str = env!("CARGO_PKG_NAME");
static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn config_path() -> &'static Path {
    CONFIG_PATH.get_or_init(|| utils::home_dir().join(format!(".{APPNAME}-config")))
}

//! Core functionalities of this program
//!
//! Including configuration, toolchain, toolset management.

mod os;

use std::path::PathBuf;

use anyhow::Result;
use url::Url;

pub(crate) trait Preinstallation {
    /// Configure environment variables for `rustup`.
    ///
    /// This will set persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`.
    fn config_rustup_env_vars() -> Result<()>;
    /// Configuration options for `cargo`.
    ///
    /// This is basically configuring replaced source of `crates-io` for now.
    fn config_cargo() -> Result<()>;
}

#[derive(Debug, Default)]
pub(crate) struct Configuration {
    install_dir: Option<PathBuf>,
    rustup_dist_server: Option<Url>,
    rustup_update_root: Option<Url>,
    cargo_registry_url: Option<Url>,
}

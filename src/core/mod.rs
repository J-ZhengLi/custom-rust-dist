//! Core functionalities of this program
//!
//! Including configuration, toolchain, toolset management.

mod custom_instructions;
pub mod install;
mod os;
pub(crate) mod parser;
pub(crate) mod rustup;
pub(crate) mod try_it;
pub(crate) mod uninstall;

use anyhow::Result;

macro_rules! declare_env_vars {
    ($($key:ident),+) => {
        $(pub(crate) const $key: &str = stringify!($key);)*
        #[cfg(windows)]
        pub(crate) static ALL_VARS: &[&str] = &[$($key),+];
    };
}

declare_env_vars!(
    CARGO_HOME,
    RUSTUP_HOME,
    RUSTUP_DIST_SERVER,
    RUSTUP_UPDATE_ROOT
);

/// Contains definition of installation steps, including pre-install configs.
///
/// Make sure to always call `init()` as it creates essential folders to
/// hold the installation files.
pub trait EnvConfig {
    /// Configure environment variables for `rustup`.
    ///
    /// This will set persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`.
    fn config_rustup_env_vars(&self) -> Result<()>;
}

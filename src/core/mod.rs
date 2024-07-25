//! Core functionalities of this program
//!
//! Including configuration, toolchain, toolset management.

mod cargo_config;
mod os;

use std::path::PathBuf;

use anyhow::{Context, Result};
use url::Url;

/// Contains definition of installation steps, including pre-install configs.
///
/// Make sure to always call `init()` as it creates essential folders to
/// hold the installation files.
pub(crate) trait Installation {
    fn init(&self) -> Result<()>;
    /// Configure environment variables for `rustup`.
    ///
    /// This will set persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`.
    fn config_rustup_env_vars(&self) -> Result<()>;
    /// Configuration options for `cargo`.
    ///
    /// This will write a `config.toml` file to `CARGO_HOME`.
    fn config_cargo(&self) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct InstallConfiguration {
    pub(crate) cargo_registry: Option<(String, Url)>,
    /// Path to install everything.
    ///
    /// Note that this folder will includes `.cargo` and `.rustup` folders as well.
    /// And the default location will be `$HOME` directory (`%USERPROFILE%` on windows).
    /// So, even if the user didn't specify any install path, a pair of env vars will still
    /// be written (CARGO_HOME and RUSTUP_HOME), as they will be located in a sub folder of `$HOME`,
    /// which is [`installer_home`](utils::installer_home).
    pub(crate) install_dir: PathBuf,
    pub(crate) rustup_dist_server: Option<Url>,
    pub(crate) rustup_update_root: Option<Url>,
}

impl InstallConfiguration {
    pub(crate) fn cargo_home(&self) -> PathBuf {
        self.install_dir.join(".cargo")
    }

    pub(crate) fn rustup_home(&self) -> PathBuf {
        self.install_dir.join(".rustup")
    }

    pub(crate) fn env_vars(&self) -> Result<Vec<(&'static str, String)>> {
        let cargo_home = self
            .cargo_home()
            .to_str()
            .map(ToOwned::to_owned)
            .context("`install-dir` cannot contains invalid unicodes")?;
        // This `unwrap` is safe here because we've already make sure the `install_dir`'s path can be
        // converted to string with the `cargo_home` variable.
        let rustup_home = self.rustup_home().to_str().unwrap().to_string();
        // Clippy suggest removing `into_iter`, which might be a bug, as removing it prevent
        // `.chain` method being used.
        #[allow(clippy::useless_conversion)]
        let mut env_vars: Vec<(&str, String)> = self
            .rustup_dist_server
            .clone()
            .map(|s| ("RUSTUP_DIST_SERVER", s.to_string()))
            .into_iter()
            .chain(
                self.rustup_update_root
                    .clone()
                    .map(|s| ("RUSTUP_UPDATE_ROOT", s.to_string()))
                    .into_iter(),
            )
            .collect();
        env_vars.push(("CARGO_HOME", cargo_home));
        env_vars.push(("RUSTUP_HOME", rustup_home));

        Ok(env_vars)
    }
}

/// Contains definition of uninstallation steps.
pub(crate) trait Uninstallation {
    /// Remove persistent environment variables for `rustup`.
    ///
    /// This will remove persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`.
    fn remove_rustup_env_vars(&self) -> Result<()>;
    /// The last step of uninstallation, this will remove the binary itself, along with
    /// the folder it's in.
    fn remove_self(&self) -> Result<()>;
}

/// Configurations to use when installing.
// NB: Currently, there's no uninstall configurations, this struct is only
// used for abstract purpose.
pub(crate) struct UninstallConfiguration;

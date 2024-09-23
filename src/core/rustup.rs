use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use url::Url;

use super::directories::RimDir;
use super::install::InstallConfiguration;
use super::parser::toolset_manifest::ToolsetManifest;
use super::uninstall::UninstallConfiguration;
use super::CARGO_HOME;
use super::RUSTUP_DIST_SERVER;
use super::RUSTUP_HOME;
use crate::toolset_manifest::Proxy;
use crate::utils;
use crate::utils::execute_with_env;
use crate::utils::{download, execute, force_url_join, set_exec_permission};

#[cfg(windows)]
pub(crate) const RUSTUP_INIT: &str = "rustup-init.exe";
#[cfg(not(windows))]
pub(crate) const RUSTUP_INIT: &str = "rustup-init";

#[cfg(windows)]
const RUSTUP: &str = "rustup.exe";
#[cfg(not(windows))]
const RUSTUP: &str = "rustup";

pub struct ToolchainInstaller;

impl ToolchainInstaller {
    pub(crate) fn init() -> Self {
        std::env::remove_var("RUSTUP_TOOLCHAIN");
        Self
    }

    fn download_rustup_init(&self, dest: &Path, server: &Url, proxy: Option<&Proxy>) -> Result<()> {
        let download_url =
            force_url_join(server, &format!("dist/{}/{RUSTUP_INIT}", env!("TARGET")))
                .context("Failed to init rustup download url.")?;
        download(RUSTUP_INIT, &download_url, dest, proxy).context("Failed to download rustup.")
    }

    fn install_rustup(&self, rustup_init: &PathBuf) -> Result<()> {
        let args = [
            // tell rustup not to add `. $HOME/.cargo/env` because we are writing one for them.
            "--no-modify-path",
            "--default-toolchain",
            "none",
            "--default-host",
            env!("TARGET"),
            "-vy",
        ];
        execute(rustup_init, &args)
    }

    fn install_toolchain_via_rustup(
        &self,
        rustup: &Path,
        manifest: &ToolsetManifest,
        components: Vec<&str>,
    ) -> Result<()> {
        // TODO: check local manifest.
        let version = manifest.rust.version.clone();
        let mut args = vec!["toolchain", "install", &version, "--no-self-update"];
        if let Some(profile) = &manifest.rust.profile {
            args.extend(["--profile", &profile.name]);
        }
        if !components.is_empty() {
            args.push("--component");
            args.extend(components);
        }
        if let Some(local_server) = manifest.offline_dist_server()? {
            execute_with_env(rustup, &args, [(RUSTUP_DIST_SERVER, local_server.as_str())])
        } else {
            execute(rustup, &args)
        }
    }

    pub(crate) fn install(
        &self,
        config: &InstallConfiguration,
        manifest: &ToolsetManifest,
        optional_components: &[String],
    ) -> Result<()> {
        let (rustup_init, maybe_temp_dir) = if let Some(bundled_rustup) = &manifest.rustup_bin()? {
            (bundled_rustup.to_path_buf(), None)
        } else {
            // We are putting the binary here so that it will be deleted automatically after done.
            let temp_dir = config.create_temp_dir("rustup-init")?;
            let rustup_init = temp_dir.path().join(RUSTUP_INIT);
            // Download rustup-init.
            self.download_rustup_init(
                &rustup_init,
                &config.rustup_update_root,
                manifest.proxy.as_ref(),
            )?;
            (rustup_init, Some(temp_dir))
        };

        // File permission
        set_exec_permission(&rustup_init)?;
        self.install_rustup(&rustup_init)?;

        // We don't need the rustup-init anymore, drop the whole temp dir containing it.
        drop(maybe_temp_dir);

        // Install rust toolchain & components via rustup.
        let rustup = config.cargo_bin().join(RUSTUP);
        let components_to_install = manifest
            .rust
            .components
            .iter()
            .map(|s| s.as_str())
            .chain(optional_components.iter().map(|s| s.as_str()))
            .collect();
        self.install_toolchain_via_rustup(&rustup, manifest, components_to_install)?;

        // Remove the `rustup` uninstall entry on windows, because we don't want
        // uses to accidently uninstall `rustup` thus removing the installed binary of this program.
        #[cfg(windows)]
        super::os::windows::do_remove_from_programs(
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Rustup",
        )?;

        Ok(())
    }

    // Rustup self uninstall all the components and toolchains.
    pub(crate) fn remove_self(&self, config: &UninstallConfiguration) -> Result<()> {
        let rustup = config.cargo_bin().join(RUSTUP);
        let args = vec!["self", "uninstall", "-y"];
        execute_with_env(
            rustup,
            &args,
            [
                (CARGO_HOME, utils::path_to_str(config.cargo_home())?),
                (RUSTUP_HOME, utils::path_to_str(config.rustup_home())?),
            ],
        )?;
        Ok(())
    }
}

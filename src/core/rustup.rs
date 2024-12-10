use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::info;
use url::Url;

use super::directories::RimDir;
use super::install::InstallConfiguration;
use super::parser::toolset_manifest::ToolsetManifest;
use super::uninstall::UninstallConfiguration;
use super::CARGO_HOME;
use super::RUSTUP_DIST_SERVER;
use super::RUSTUP_HOME;
use crate::toolset_manifest::Proxy;
use crate::utils::{self, download_with_proxy, set_exec_permission, url_join};

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

    async fn install_toolchain_via_rustup(
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
            utils::Command::new(rustup)
                .args(&args)
                .env(RUSTUP_DIST_SERVER, local_server.as_str())
                .run()
                .await
        } else {
            utils::Command::new(rustup).args(&args).run().await
        }
    }

    /// Install rust toolchain & components via rustup.
    pub(crate) async fn install(
        &self,
        config: &InstallConfiguration<'_>,
        manifest: &ToolsetManifest,
        optional_components: &[String],
    ) -> Result<()> {
        let rustup = ensure_rustup(config, manifest).await?;

        let components_to_install = manifest
            .rust
            .components
            .iter()
            .map(|s| s.as_str())
            .chain(optional_components.iter().map(|s| s.as_str()))
            .collect();
        self.install_toolchain_via_rustup(&rustup, manifest, components_to_install).await?;

        // Remove the `rustup` uninstall entry on windows, because we don't want users to
        // accidently uninstall `rustup` thus removing the tools installed by this program.
        #[cfg(windows)]
        super::os::windows::do_remove_from_programs(
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Rustup",
        )?;

        Ok(())
    }

    /// Update rust toolchain by invoking `rustup toolchain add`, then `rustup default`
    pub(crate) async fn update(
        &self,
        config: &InstallConfiguration<'_>,
        manifest: &ToolsetManifest,
    ) -> Result<()> {
        let rustup = ensure_rustup(config, manifest).await?;

        let tc_ver = manifest.rust_version();
        utils::Command::new(&rustup)
            .args(&["toolchain", "add", tc_ver])
            .run().await?;
        utils::Command::new(&rustup)
            .args(&["default", tc_ver])
            .run().await?;
        Ok(())
    }

    // Rustup self uninstall all the components and toolchains.
    pub(crate) async fn remove_self(&self, config: &UninstallConfiguration<'_>) -> Result<()> {
        let rustup = config.cargo_bin().join(RUSTUP);
        utils::Command::new(rustup)
            .args(&["self", "uninstall", "-y"])
            .env(CARGO_HOME, config.cargo_home())
            .env(RUSTUP_HOME, config.rustup_home())
            .run()
            .await
    }
}

async fn ensure_rustup(config: &InstallConfiguration<'_>, manifest: &ToolsetManifest) -> Result<PathBuf> {
    let rustup_bin = config.cargo_bin().join(RUSTUP);
    if rustup_bin.exists() {
        return Ok(rustup_bin);
    }

    // Run the bundled rustup-init or download it from server.
    // NOTE: When running updates, the manifest we cached might states that it has a bundled
    // rustup-init, but in reality it might not exists, therefore we need to check if that file
    // exists and download it otherwise.
    let (rustup_init, maybe_temp_dir) =
        if let Some(bundled_rustup) = &manifest.rustup_bin()?.filter(|p| p.is_file()) {
            (bundled_rustup.to_path_buf(), None)
        } else {
            // We are putting the binary here so that it will be deleted automatically after done.
            let temp_dir = config.create_temp_dir("rustup-init")?;
            let rustup_init = temp_dir.path().join(RUSTUP_INIT);
            // Download rustup-init.
            download_rustup_init(
                &rustup_init,
                &config.rustup_update_root,
                manifest.proxy.as_ref(),
            )?;
            (rustup_init, Some(temp_dir))
        };

    install_rustup(&rustup_init).await?;
    // We don't need the rustup-init anymore, drop the whole temp dir containing it.
    drop(maybe_temp_dir);

    Ok(rustup_bin)
}

fn download_rustup_init(dest: &Path, server: &Url, proxy: Option<&Proxy>) -> Result<()> {
    info!("{}", t!("downloading_rustup_init"));

    let download_url = url_join(server, &format!("dist/{}/{RUSTUP_INIT}", env!("TARGET")))
        .context("Failed to init rustup download url.")?;
    download_with_proxy(RUSTUP_INIT, &download_url, dest, proxy)
        .context("Failed to download rustup.")
}

async fn install_rustup(rustup_init: &PathBuf) -> Result<()> {
    // make sure it can be executed
    set_exec_permission(rustup_init)?;

    let args = [
        // tell rustup not to add `. $HOME/.cargo/env` because we already wrote one for them.
        "--no-modify-path",
        "--default-toolchain",
        "none",
        "--default-host",
        env!("TARGET"),
        "-vy",
    ];
    utils::Command::new(rustup_init).args(&args).run().await
}

use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use url::Url;

use super::install::InstallConfiguration;
use super::parser::manifest::ToolsetManifest;
use crate::utils;
use crate::utils::create_executable_file;
use crate::utils::download_from_start;
use crate::utils::execute;

#[cfg(windows)]
pub(crate) const RUSTUP_INIT: &str = "rustup-init.exe";
#[cfg(not(windows))]
pub(crate) const RUSTUP_INIT: &str = "rustup-init";

#[cfg(windows)]
const RUSTUP: &str = "rustup.exe";
#[cfg(not(windows))]
const RUSTUP: &str = "rustup";

pub struct Rustup;

impl Rustup {
    pub(crate) fn init() -> Self {
        std::env::remove_var("RUSTUP_TOOLCHAIN");
        Self
    }

    pub(crate) fn download_rustup_init(&self, dest: &Path, server: &Url) -> Result<()> {
        let download_url =
            utils::force_url_join(server, &format!("dist/{}/{RUSTUP_INIT}", env!("TARGET")))
                .context("Failed to init rustup download url.")?;
        download_from_start(RUSTUP_INIT, &download_url, dest).context("Failed to download rustup.")
    }

    pub(crate) fn generate_rustup(&self, rustup_init: &PathBuf) -> Result<()> {
        let args = [
            "--default-toolchain",
            "none",
            "--default-host",
            env!("TARGET"),
            "-vy",
        ];
        execute(rustup_init, &args)
    }

    fn download_rust_toolchain(&self, rustup: &Path, manifest: &ToolsetManifest) -> Result<()> {
        // TODO: check local manifest.
        let version = manifest.rust.version.clone();
        let mut args = vec!["toolchain", "install", &version, "--no-self-update"];
        if let Some(profile) = &manifest.rust.profile {
            args.extend(["--profile", profile]);
        }
        execute(rustup, &args)
    }

    fn download_rust_components(&self, rustup: &Path, components: Vec<&str>) -> Result<()> {
        if components.is_empty() {
            return Ok(());
        }
        let mut args = vec!["component", "add"];
        args.extend(components);
        execute(rustup, &args)
    }

    pub(crate) fn download_toolchain(
        &self,
        config: &InstallConfiguration,
        manifest: &ToolsetManifest,
        optional_components: Option<&[String]>,
    ) -> Result<()> {
        // We are putting the binary here so that it will be deleted automatically after done.
        let temp_dir = config.create_temp_dir("rustup-init")?;
        let rustup_init = temp_dir.path().join(RUSTUP_INIT);
        // Download rustup-init.
        self.download_rustup_init(&rustup_init, &config.rustup_update_root)?;
        // File permission
        create_executable_file(&rustup_init)?;
        // Install rustup.
        self.generate_rustup(&rustup_init)?;
        // Install rust toolchain via rustup.
        let rustup = config.cargo_bin().join(RUSTUP);
        self.download_rust_toolchain(&rustup, manifest)?;

        // Install extra rust components via rustup.
        // NOTE: that the `component` field in manifest is essential
        let components_to_install = if let Some(opt) = optional_components {
            manifest
                .rust
                .components
                .iter()
                .map(|s| s.as_str())
                .chain(opt.iter().map(|s| s.as_str()))
                .collect()
        } else {
            manifest
                .rust
                .components
                .iter()
                .map(|s| s.as_str())
                .collect()
        };
        self.download_rust_components(&rustup, components_to_install)?;

        // Remove the `rustup` uninstall entry on windows, because we don't want
        // uses to accidently uninstall `rustup` thus removing the installed binary of this program.
        #[cfg(windows)]
        super::os::windows::do_remove_from_programs(
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Rustup",
        )?;

        Ok(())
    }
}

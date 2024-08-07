use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::install::InstallConfiguration;
use super::parser::manifest::ToolsetManifest;
use crate::utils::cmd_output;
use crate::utils::cmd_output_with_input;
use crate::utils::create_executable_file;
use crate::utils::download_from_start;
use crate::utils::HostTriple;

// FIXME: remove this `allow` before 0.1.0 release.
#[allow(unused)]
const RUSTUP_DIST_SERVER: &str = "https://mirrors.tuna.tsinghua.edu.cn/rustup";
const RUSTUP_UPDATE_ROOT: &str = "https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup";

#[cfg(windows)]
const RUSTUP_INIT: &str = "rustup-init.exe";
#[cfg(not(windows))]
const RUSTUP_INIT: &str = "rustup-init";

#[cfg(windows)]
const RUSTUP: &str = "rustup.exe";
#[cfg(not(windows))]
const RUSTUP: &str = "rustup";

pub struct Rustup {
    triple: HostTriple,
}

impl Default for Rustup {
    fn default() -> Self {
        let host_triple = match HostTriple::from_host() {
            Some(host_triple) => host_triple,
            None => panic!("Failed to get local host triple."),
        };
        Self {
            triple: host_triple,
        }
    }
}

impl Rustup {
    pub fn init() -> Self {
        std::env::remove_var("RUSTUP_TOOLCHAIN");
        Self::default()
    }

    fn download_rustup_init(&self, dest: &Path) -> Result<()> {
        let download_url = url::Url::parse(&format!(
            "{}/{}/{}/{}",
            RUSTUP_UPDATE_ROOT, "dist", self.triple, RUSTUP_INIT
        ))
        .context("Failed to init rustup download url.")?;
        download_from_start(RUSTUP_INIT, &download_url, dest).context("Failed to download rustup.")
    }

    fn generate_rustup(&self, rustup_init: &PathBuf) -> Result<()> {
        let args = ["--default-toolchain", "none"];
        let input = b"\n";
        cmd_output_with_input(rustup_init, &args, input)
    }

    fn download_rust_toolchain(&self, rustup: &Path, manifest: &ToolsetManifest) -> Result<()> {
        // TODO: check local manifest.
        let version = manifest.rust.version.clone();
        let args = ["toolchain", "install", &version, "--no-self-update"];
        cmd_output(rustup, &args)
    }

    fn download_rust_component(&self, rustup: &Path, compoent: &String) -> Result<()> {
        let args = ["component", "add", compoent];
        cmd_output(rustup, &args)
    }

    pub(crate) fn download_toolchain(
        &self,
        config: &InstallConfiguration,
        manifest: &ToolsetManifest,
    ) -> Result<()> {
        let rustup_init = config.install_dir.join(RUSTUP_INIT);
        // Download rustup-init.
        self.download_rustup_init(&rustup_init)?;
        // File permission
        create_executable_file(&rustup_init)?;
        // Install rustup.
        self.generate_rustup(&rustup_init)?;
        // Install rust toolchain via rustup.
        let rustup = config.cargo_home().join("bin").join(RUSTUP);
        self.download_rust_toolchain(&rustup, manifest)?;
        // Install extral rust component via rustup.
        if let Some(compoents) = &manifest.rust.components {
            for cpt in compoents {
                self.download_rust_component(&rustup, cpt)?;
            }
        }
        Ok(())
    }
}

use env::consts::EXE_SUFFIX;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use anyhow::{bail, Result};
use cfg_if::cfg_if;

use crate::common::{self, *};

pub const DIST_HELP: &str = r#"
Generate release binaries

Usage: cargo dev dist [OPTIONS]

Options:
        --cli       Generate release binary for CLI mode only
        --gui       Generate release binary for GUI mode only
    -b, --binary-only
                    Build binary only (net-installer), skip offline package generation
    -h, -help       Print this help message
"#;

#[derive(Debug, Clone, Copy)]
pub enum DistMode {
    Both,
    Gui,
    Cli,
}

/// A dist worker has two basic jobs:
///
/// 1. Run build command to create binaries.
/// 2. Collect built binaries and move them into specific folder.
#[derive(Debug)]
struct DistWorker<'a> {
    build_args: &'a [&'static str],
    source_bin: PathBuf,
    dest_bin_name: String,
}

impl DistWorker<'_> {
    fn cli(src_dir: &Path) -> Self {
        Self {
            build_args: &["build", "--release", "--locked"],
            source_bin: src_dir.join(format!("rim-cli{EXE_SUFFIX}")),
            dest_bin_name: format!("{}-installer-cli{EXE_SUFFIX}", t!("vendor_en")),
        }
    }
    fn gui(src_dir: &Path) -> Self {
        Self {
            build_args: &["tauri", "build", "-b", "none", "--", "--locked"],
            source_bin: src_dir.join(format!("rim-gui{EXE_SUFFIX}")),
            dest_bin_name: format!("{}-installer{EXE_SUFFIX}", t!("vendor_en")),
        }
    }
    fn dist_common_(&self, target: &str, noweb: bool) -> Result<PathBuf> {
        // Get the dest directory and create one if it does not exist
        let mut dest_dir = PathBuf::from(
            env::var("RIM_WORKSPACE_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR").to_string()),
        )
        .with_file_name("dist");
        dest_dir.push(target);

        let mut cmd = Command::new("cargo");
        cmd.env("HOST_TRIPPLE", target);
        cmd.args(self.build_args);

        if noweb {
            dest_dir.push(format!("{}-{target}", t!("vendor_en")));

            cmd.args(["--features", "no-web"]);
        }
        fs::create_dir_all(&dest_dir)?;

        let status = cmd.status()?;
        if status.success() {
            // copy and rename the binary with vendor name
            let to = dest_dir.join(&self.dest_bin_name);
            copy(&self.source_bin, to)?;
        } else {
            bail!("build failed with code: {}", status.code().unwrap_or(-1));
        }
        Ok(dest_dir)
    }

    fn dist_net_installer(&self, specific_target: Option<&str>) -> Result<()> {
        let target = specific_target.unwrap_or(env!("TARGET"));
        self.dist_common_(target, false)?;
        Ok(())
    }

    fn dist_noweb_installer(&self, specific_target: Option<&str>) -> Result<()> {
        let target = specific_target.unwrap_or(env!("TARGET"));
        let dest_pkg_dir = self.dist_common_(target, true)?.join("packages");
        ensure_dir(&dest_pkg_dir)?;

        // Copy packages to dest dir as well
        // TODO: download from web instead
        let src_pkg_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .with_file_name("resources")
            .join("packages");
        // Step 1: easiest, copy the folder with `target` as name
        let target_specific_pkgs_dir = src_pkg_dir.join(target);
        if target_specific_pkgs_dir.exists() {
            copy_into(target_specific_pkgs_dir, &dest_pkg_dir)?;
        }
        // Step 2: copy OS "common" packages
        let (target_no_abi, _abi) = target.rsplit_once('-').expect("invalid target format");
        let common_pkgs_dir = src_pkg_dir.join(target_no_abi);
        if common_pkgs_dir.exists() {
            copy_into(common_pkgs_dir, &dest_pkg_dir)?;
        }
        // Step 3: copy only relavent packages in the `dist` folder
        let dist_dir = src_pkg_dir.join("dist");
        if dist_dir.exists() {
            let dist_entries = walk_dir(&dist_dir, true)?;
            for entry in dist_entries {
                let relpath = entry.strip_prefix(&src_pkg_dir)?;
                let dest_path = dest_pkg_dir.join(relpath);
                if entry.is_dir() {
                    fs::create_dir_all(&dest_path)?;
                } else if let Some(filename) = entry.file_name().and_then(|s| s.to_str()) {
                    if filename.contains(target)
                        || filename.starts_with("rust-src")
                        || filename.starts_with("channel-rust")
                    {
                        copy_as(entry, dest_path)?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn dist(mode: DistMode, binary_only: bool) -> Result<()> {
    // Get the target dir
    let dev_bin = env::current_exe()?;
    let release_dir = dev_bin.parent().unwrap().with_file_name("release");

    let mut workers = vec![];

    match mode {
        DistMode::Cli => {
            workers.push(DistWorker::cli(&release_dir));
        }
        DistMode::Gui => {
            workers.push(DistWorker::gui(&release_dir));
        }
        DistMode::Both => {
            workers.push(DistWorker::cli(&release_dir));
            workers.push(DistWorker::gui(&release_dir));
        }
    };

    if !matches!(mode, DistMode::Cli) {
        common::install_gui_deps();
    }

    for worker in workers {
        cfg_if! {
            if #[cfg(all(windows, target_arch = "x86_64"))] {
                let msvc_target = "x86_64-pc-windows-msvc";
                let gnu_target = "x86_64-pc-windows-gnu";

                worker.dist_net_installer(Some(msvc_target))?;
                worker.dist_net_installer(Some(gnu_target))?;
                if !binary_only {
                    worker.dist_noweb_installer(Some(msvc_target))?;
                    worker.dist_noweb_installer(Some(gnu_target))?;
                }
            } else {
                worker.dist_net_installer(None)?;
                if !binary_only {
                    worker.dist_noweb_installer(None)?;
                }
            }
        }
    }

    Ok(())
}

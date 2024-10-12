use env::consts::EXE_SUFFIX;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

use anyhow::{anyhow, bail, Context, Result};
use cfg_if::cfg_if;

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

impl<'a> DistWorker<'a> {
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
        let mut dest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("dist");
        dest_dir.push(target);

        let mut cmd = Command::new("cargo");
        cmd.env("HOST_TRIPPLE", target);
        cmd.args(self.build_args);

        if noweb {
            dest_dir.push("offline-package");
            dest_dir.push(format!("{}-{target}", t!("vendor_en")));

            cmd.args(["--features", "no-web"]);
        } else {
            dest_dir.push("net-installer");
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
        fs::create_dir_all(&dest_pkg_dir)?;

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
        pnpm_install();
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

fn pnpm_install() {
    println!("running `pnpm i`");
    let fail_msg = "unable to run `pnpm i`, \
            please manually cd to `installer/` then run the command manually";

    let gui_crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("installer");
    assert!(gui_crate_dir.exists());

    cfg_if! {
        if #[cfg(windows)] {
            let mut status = Command::new("cmd.exe");
            status.args(["/C", "pnpm", "i"]);
        } else {
            let mut status = Command::new("pnpm");
            status.arg("i");
        }
    }
    status
        .current_dir(gui_crate_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let Ok(st) = status.status() else {
        println!("{fail_msg}");
        return;
    };

    if !st.success() {
        println!("{fail_msg}: {}", st.code().unwrap_or(-1));
    }
}

// NB: If we end up using too many util functions from `rim`,
// consider separate the `utils` module as a separated crate.
/// Copy file or directory into an existing directory.
///
/// Similar to [`copy_file_to`], except this will recursively copy directory as well.
pub fn copy_into<P, Q>(from: P, to: Q) -> Result<PathBuf>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    if !to.as_ref().is_dir() {
        bail!("'{}' is not a directory", to.as_ref().display());
    }

    let dest = to.as_ref().join(from.as_ref().file_name().ok_or_else(|| {
        anyhow!(
            "path '{}' does not have a file name",
            from.as_ref().display()
        )
    })?);

    copy_as(from, &dest)?;
    Ok(dest)
}

/// Copy file or directory to a specified path.
pub fn copy_as<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    fn copy_dir_(src: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(dest)?;
        for maybe_entry in src.read_dir()? {
            let entry = maybe_entry?;
            let src = entry.path();
            let dest = dest.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_dir_(&src, &dest)?;
            } else {
                copy(src, dest)?;
            }
        }
        Ok(())
    }

    if !from.as_ref().exists() {
        bail!(
            "failed to copy '{}': path does not exist",
            from.as_ref().display()
        );
    }

    if from.as_ref().is_file() {
        copy(from, to)
    } else {
        copy_dir_(from.as_ref(), to.as_ref()).with_context(|| {
            format!(
                "could not copy directory '{}' to '{}'",
                from.as_ref().display(),
                to.as_ref().display()
            )
        })
    }
}

fn copy<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    // Make sure no redundent work is done
    if let (Ok(src_modif_time), Ok(dest_modif_time)) = (
        fs::metadata(&from).and_then(|m| m.modified()),
        fs::metadata(&to).and_then(|m| m.modified()),
    ) {
        if src_modif_time == dest_modif_time {
            return Ok(());
        }
    }

    fs::copy(&from, &to).with_context(|| {
        format!(
            "could not copy file '{}' to '{}'",
            from.as_ref().display(),
            to.as_ref().display()
        )
    })?;
    Ok(())
}

/// Attempts to read a directory path, then return a list of paths
/// that are inside the given directory, may or may not including sub folders.
pub fn walk_dir(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    fn collect_paths_(dir: &Path, paths: &mut Vec<PathBuf>, recursive: bool) -> Result<()> {
        for dir_entry in dir.read_dir()?.flatten() {
            paths.push(dir_entry.path());
            if recursive && matches!(dir_entry.file_type(), Ok(ty) if ty.is_dir()) {
                collect_paths_(&dir_entry.path(), paths, true)?;
            }
        }
        Ok(())
    }
    let mut paths = vec![];
    collect_paths_(dir, &mut paths, recursive)?;
    Ok(paths)
}

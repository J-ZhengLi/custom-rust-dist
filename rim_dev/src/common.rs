use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};

// NB: If we end up using too many util functions from `rim`,
// consider separate the `utils` module as a separated crate.
/// Copy file or directory into an existing directory.
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

/// An [`fs::copy`] wrapper that only copies a file if:
///
/// - `to` does not exist yet.
/// - `to` exists but have different modified date.
///
/// Also, this function make sure the parent directory of `to` exists by creating one if not.
pub fn copy<P, Q>(from: P, to: Q) -> Result<()>
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

    ensure_parent_dir(&to)?;
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

pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().is_dir() {
        fs::create_dir_all(path.as_ref()).with_context(|| {
            format!(
                "unable to create specified directory '{}'",
                path.as_ref().display()
            )
        })?;
    }
    Ok(())
}

pub fn ensure_parent_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(p) = path.as_ref().parent() {
        ensure_dir(p)?;
    }
    Ok(())
}

pub fn install_gui_deps() {
    println!("running `pnpm i`");
    let fail_msg = "unable to run `pnpm i`, \
            please manually cd to `rim_gui/` then run the command manually";

    let gui_crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("rim_gui");
    assert!(gui_crate_dir.exists());

    cfg_if::cfg_if! {
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

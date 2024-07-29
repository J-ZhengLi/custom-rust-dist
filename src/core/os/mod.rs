//! This module contains implementations of core functionalities,
//! each submodule must implement traits defined in [`core`](crate::core).
//! such as [`Installation`](crate::core::Installation).
//!
//! Note: If you add/remove sub mods here to add/remove support for certain OS,
//! make sure to update `build.rs` as well.

#[cfg(unix)]
pub(crate) mod unix;
#[cfg(windows)]
pub(crate) mod windows;

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Indicate whether if the [`Installation::init`] was called.
///
/// If calling `.get()` on this lock returns a `None`, meaning it hasn't been
/// initialized, which leads to a conclusion that `init` was indeed not called.
static INIT_ONCE: OnceLock<()> = OnceLock::new();

/// Try getting the installation root judging be current executable path.
//
// This program should be installed under `{install_dir}/.cargo/bin/`,
// we should be able to track the installation dir by going up three parents.
// We should also make sure it is indeed the installation dir by checking if
// the folder fits the characteristic.
// FIXME: There might be risks involved, resulting unintended directory being removed
// after uninstallation.
fn install_dir_from_exe_path() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().context("cannot locate current executable")?;
    let comp_count = exe_path.components().count();
    let maybe_install_dir: PathBuf = exe_path
        .components()
        .take(comp_count.saturating_sub(3))
        .collect();

    if !maybe_install_dir.is_dir() {
        // Check if it exists, this could fail if comp_count was less then `3`,
        // meaning the current exe was put into root dir, or any folder that are not deep enough.
        bail!(
            "install directory does not exist, \
        make sure this binary is in its original location before running uninstall."
        );
    }
    if !maybe_install_dir.ends_with(env!("CARGO_PKG_NAME")) {
        // Check if the install dir's name is correct. This could fail if someone has
        // put this binary in an arbitrary folder, then run uninstallation, which
        // resulted in removing unintended directories.
        bail!(
            "directory '{}' does not seems like the currect install root, \
        make sure this binary is in its original location before running uninstall.",
            maybe_install_dir.display()
        );
    }

    Ok(maybe_install_dir)
}

fn ensure_init_call() {
    assert!(
        INIT_ONCE.get().is_some(),
        "Internal Error: `Installation::init` should be called first"
    );
}

//! This module contains OS specific implementations of core functionalities,
//! each submodule must implement the below traits:
//!
//! 1. [`EnvConfig`](crate::core::install::EnvConfig)
//! 2. [`UninstallConfiguration`](crate::core::uninstall::UninstallConfiguration)

#[cfg(unix)]
pub(crate) mod unix;
#[cfg(windows)]
pub(crate) mod windows;

use anyhow::Result;
use std::path::Path;

use super::GlobalOpts;

/// Add a given path to OS's `PATH` variable.
///
/// Note this will do nothing if either
/// [`no_modify_path`](GlobalOpts::no_modify_path) or [`no_modify_env`](GlobalOpts::no_modify_env)
/// was set to true.
pub(crate) fn add_to_path(path: &Path) -> Result<()> {
    let g_opt = GlobalOpts::get();
    if g_opt.no_modify_path || g_opt.no_modify_env {
        // skip PATH modification of user specified not to modify it
        return Ok(());
    }

    #[cfg(windows)]
    windows::add_to_path(path)?;

    #[cfg(unix)]
    unix::add_to_path(path)?;

    Ok(())
}

/// Remove a given path from OS's `PATH` variable.
///
/// Note this will do nothing if either
/// [`no_modify_path`](GlobalOpts::no_modify_path) or [`no_modify_env`](GlobalOpts::no_modify_env)
/// was set to true, or if the path is not in the `PATH` variable.
pub(crate) fn remove_from_path(path: &Path) -> Result<()> {
    let g_opt = GlobalOpts::get();
    if g_opt.no_modify_path || g_opt.no_modify_env {
        // skip PATH modification of user specified not to modify it
        return Ok(());
    }

    #[cfg(windows)]
    windows::remove_from_path(path)?;

    #[cfg(not(windows))]
    unix::remove_from_path(path)?;

    Ok(())
}

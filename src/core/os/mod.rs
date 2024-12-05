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

pub(crate) fn add_to_path(path: &Path) -> Result<()> {
    #[cfg(windows)]
    windows::add_to_path(path)?;

    #[cfg(unix)]
    unix::add_to_path(path)?;

    Ok(())
}

pub(crate) fn remove_from_path(path: &Path) -> Result<()> {
    #[cfg(windows)]
    windows::remove_from_path(path)?;

    #[cfg(not(windows))]
    unix::remove_from_path(path)?;

    Ok(())
}

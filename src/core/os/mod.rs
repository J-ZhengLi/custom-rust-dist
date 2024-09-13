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

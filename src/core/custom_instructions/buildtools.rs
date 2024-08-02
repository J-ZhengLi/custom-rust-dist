use std::path::Path;
use anyhow::Result;
use crate::core::InstallConfiguration;

#[cfg(windows)]
pub(super) fn install(_path: &Path, _config: &InstallConfiguration) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn install(_path: &Path, _config: &InstallConfiguration) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn uninstall() -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn uninstall() -> Result<()> {
    Ok(())
}

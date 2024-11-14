pub(crate) mod cargo_config;
pub mod dist_manifest;
pub mod fingerprint;
pub(crate) mod release_info;
pub mod toolset_manifest;

use anyhow::{bail, Context, Result};
use fingerprint::InstallationRecord;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};
use toml::{de, ser};

use crate::utils;

static INSTALL_DIR_ONCE: OnceLock<PathBuf> = OnceLock::new();

#[allow(unused)]
pub(crate) trait TomlParser {
    const FILENAME: &str;

    /// Deserialize a certain type from [`str`] value.
    fn from_str(from: &str) -> Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        Ok(de::from_str(from)?)
    }

    /// Serialize data of a type into [`String`].
    fn to_toml(&self) -> Result<String>
    where
        Self: Sized + Serialize,
    {
        Ok(ser::to_string(self)?)
    }

    /// Load TOML data directly from a certain file path.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let raw = utils::read_to_string("toml", path)?;
        Self::from_str(&raw)
    }
}

/// Try guessing the installation directory base on current exe path, and return the path.
///
/// This program should be installed directly under `install_dir`,
/// but in case someone accidentally put this binary into some other locations such as
/// the root, we should definitely NOT remove the parent dir after installation.
/// Therefor we need some checks:
/// 1. Make sure the parent directory is not root.
/// 2. Make sure there is a `.fingerprint` file alongside current binary.
/// 3. Make sure the parent directory matches the recorded `root` path in the fingerprint file.
///
/// # Panic
/// This function will panic if any of the above check fails.
///
/// # Note
/// This function should only be used in **manager** mode.
pub fn get_installed_dir() -> &'static Path {
    fn inner_() -> Result<PathBuf> {
        let maybe_install_dir = utils::parent_dir_of_cur_exe()?;

        // the first check
        if maybe_install_dir.parent().is_none() {
            bail!("it appears that this program was mistakenly installed in root directory");
        }
        // the second check
        if !maybe_install_dir
            .join(InstallationRecord::FILENAME)
            .is_file()
        {
            bail!("installation record cannot be found");
        }
        // the third check
        let fp = InstallationRecord::load(&maybe_install_dir)
            .context("'.fingerprint' file exists but cannot be loaded")?;
        if fp.root != maybe_install_dir {
            bail!(
                "`.fingerprint` file exists but the installation root in it \n\
                does not match the one its in"
            );
        }

        Ok(maybe_install_dir.to_path_buf())
    }

    INSTALL_DIR_ONCE.get_or_init(|| inner_().expect("unable to determine install dir"))
}

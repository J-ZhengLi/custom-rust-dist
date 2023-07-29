mod configuration;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fs::{self, read_to_string};
use std::path::Path;
use toml_edit::{de, ser};

// Re-exporting essential api, don't lint `wildcard_import` on this
#[allow(clippy::wildcard_imports)]
pub(crate) use configuration::*;

pub(crate) trait TomlTable {
    /// Deserialize from [`str`] value.
    fn from_toml(toml: &str) -> Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        Ok(de::from_str(toml)?)
    }

    /// Serialize this data into [`String`].
    fn to_toml(&self, pretty: bool) -> Result<String>
    where
        Self: Sized + Serialize,
    {
        if pretty {
            Ok(ser::to_string_pretty(self)?)
        } else {
            Ok(ser::to_string(self)?)
        }
    }

    /// Load TOML data directly from a certain file path.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized + DeserializeOwned,
    {
        let raw = read_to_string(path)?;
        Self::from_toml(&raw)
    }
}

/// Convenient function to load [`Configuration`] from a certain path.
pub(crate) fn load_config(path: &Path) -> Result<Configuration> {
    Configuration::load(path).context("unable to load configuration file.")
}

pub(crate) fn write_config(path: &Path, cfg: &Configuration, pretty: bool) -> Result<()> {
    let toml = cfg.to_toml(pretty)?;
    fs::write(path, toml)?;
    Ok(())
}

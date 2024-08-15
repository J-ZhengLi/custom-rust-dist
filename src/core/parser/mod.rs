pub(crate) mod cargo_config;
pub mod manifest;

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use toml::{de, ser};

use crate::utils;

#[allow(unused)]
pub(crate) trait TomlParser {
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
        let raw = utils::read_to_string(path)?;
        Self::from_str(&raw)
    }
}

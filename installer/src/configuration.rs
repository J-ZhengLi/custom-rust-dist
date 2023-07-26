use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs::read_to_string;
use toml_edit::{de, ser};

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Configuration {
    pub settings: Settings,
}

impl Configuration {
    pub fn from_toml(toml: &str) -> Result<Self> {
        Ok(de::from_str(toml)?)
    }
    pub fn to_toml(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(ser::to_string_pretty(self)?)
        } else {
            Ok(ser::to_string(self)?)
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Settings {
    pub cargo_home: Option<String>,
    pub rustup_home: Option<String>,
    pub rustup_dist_server: Option<String>,
    pub rustup_update_root: Option<String>,
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub cargo: Option<CargoSettings>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CargoSettings {
    pub git_fetch_with_cli: bool,
    pub check_revoke: bool,
    pub default_registry: Option<String>,
    pub registries: HashMap<String, CargoRegistry>,
}

impl Default for CargoSettings {
    fn default() -> Self {
        Self {
            git_fetch_with_cli: false,
            check_revoke: true,
            default_registry: None,
            registries: HashMap::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CargoRegistry {
    pub index: String,
}

/// A convenient function to load [`Configuration`] from a `toml` file.
pub(crate) fn load(path: &Path) -> Result<Configuration> {
    let conf_str = read_to_string(path)?;
    Ok(Configuration::from_toml(&conf_str)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if all fields of [`Configuration`] can be deserialized as expected.
    ///
    /// Remember to update this test case after adding new struct member(s).
    #[test]
    fn fields_de_intact() {
        let input = r#"
        [settings]
        cargo_home = "cargo"
        rustup_home = "rustup"
        rustup_dist_server = "rustup_dist_server"
        rustup_update_root = "rustup_update_root"
        http_proxy = "http_proxy"
        https_proxy = "https_proxy"
        no_proxy = "no_proxy"
        
        [settings.cargo]
        git_fetch_with_cli = true
        check_revoke = true
        default_registry = "registry"
        
        [settings.cargo.registries.registry]
        index = "index"
        "#;
        let opt = |s: &str| -> Option<String> { Some(s.into()) };
        let deserialized = Configuration::from_toml(input).expect("fail to deserialize");
        assert_eq!(
            deserialized,
            Configuration {
                settings: Settings {
                    cargo_home: opt("cargo"),
                    rustup_home: opt("rustup"),
                    rustup_dist_server: opt("rustup_dist_server"),
                    rustup_update_root: opt("rustup_update_root"),
                    http_proxy: opt("http_proxy"),
                    https_proxy: opt("https_proxy"),
                    no_proxy: opt("no_proxy"),
                    cargo: Some(CargoSettings {
                        git_fetch_with_cli: true,
                        check_revoke: true,
                        default_registry: opt("registry"),
                        registries: HashMap::from([(
                            "registry".to_string(),
                            CargoRegistry {
                                index: "index".to_string()
                            }
                        )]),
                    })
                }
            }
        )
    }

    #[test]
    fn serialize_default() {
        let input = Configuration::default();
        let toml = input.to_toml(false).expect("fail to serialize");
        let pretty_toml = input.to_toml(true).expect("fail to serialize");
        assert_eq!(toml.trim(), "settings = {}");
        assert_eq!(pretty_toml.trim(), "[settings]");
    }
}

//! Configurations for this application, including information about
//! installed tools and toolchain.

use super::TomlTable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Configuration {
    pub settings: Settings,
    pub installation: Option<Installation>,
}

impl TomlTable for Configuration {}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Settings {
    pub cargo_home: Option<String>,
    pub rustup_home: Option<String>,
    pub rustup_dist_server: Option<Url>,
    pub rustup_update_root: Option<Url>,
    pub proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub cargo: Option<CargoSettings>,
}

impl Settings {
    /// Return true if self is default settings.
    pub fn is_default(&self) -> bool {
        self == &Settings::default()
    }
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

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Installation {
    #[serde(rename = "rustup")]
    rustup_ver: String,
    toolchain: Toolchain,
    tool: Tool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Toolchain {
    default: String,
    #[serde(flatten)]
    installed: HashMap<String, ToolchainTarget>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ToolchainTarget {
    version: String,
    components: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Tool {
    keep_package: bool,
    package_dir: Option<String>,
    tools_dir: Option<String>,
    #[serde(flatten)]
    installed: HashMap<String, ToolDetail>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ToolDetail {
    version: String,
    installed_from_source: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if all fields of [`Configuration`] can be deserialized as expected.
    ///
    /// Remember to update this test case after adding new struct member(s).
    #[test]
    fn fields_de_intact() {
        let input = r#"[settings]
cargo_home = "cargo"
rustup_home = "rustup"
rustup_dist_server = "https://example.com/"
rustup_update_root = "https://example.com/"
proxy = "proxy"
no_proxy = "no_proxy"

[settings.cargo]
git_fetch_with_cli = true
check_revoke = true
default_registry = "registry"

[settings.cargo.registries.registry]
index = "index"

[installation]
rustup = "1.26.0"

[installation.toolchain]
default = "default-toolchain"

[installation.toolchain.default-toolchain]
version = "version"
components = ["component-A"]

[installation.tool]
keep_package = true
package_dir = "package_dir"
tools_dir = "tools_dir"

[installation.tool.tool-A]
version = "version"
installed_from_source = false
"#;
        let deserialized = Configuration::from_toml(input).expect("fail to deserialize");
        let serialized = deserialized.to_toml(true).expect("fail to serialize");

        assert_eq!(input, serialized);
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

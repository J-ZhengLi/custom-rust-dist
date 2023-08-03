//! Configurations for this application, including information about
//! installed tools and toolchain.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

use super::{crates::CargoInstallTrackerLite, TomlTable};
use crate::utils;

pub trait TryFromEnv {
    /// Attempt to load data from runtime environment.
    ///
    /// Typically, data can be loaded from environment variables, but could also
    /// from filesystem, such as reading config files etc.
    ///
    /// # Errors
    ///
    /// Errors will occur when the required data is in-accessable,
    /// such as when the program does not have enough permissions, or some
    /// simply does not exist thus cannot be fetched.
    fn try_from_env() -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Configuration {
    pub settings: Settings,
    pub installation: Option<Installation>,
}

impl TomlTable for Configuration {}

impl TryFromEnv for Configuration {
    fn try_from_env() -> Result<Self> {
        Ok(Self {
            settings: Settings::try_from_env()?,
            installation: Some(Installation::try_from_env()?),
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub(crate) struct Settings {
    pub cargo_home: Option<String>,
    pub rustup_home: Option<String>,
    pub rustup_dist_server: Option<Url>,
    pub rustup_update_root: Option<Url>,
    pub proxy: Option<String>,
    pub no_proxy: Option<String>,
    pub cargo: Option<CargoSettings>,
}

impl TryFromEnv for Settings {
    fn try_from_env() -> Result<Self> {
        Ok(Settings::default())
    }
}

impl Settings {
    /// Return true if self is default settings.
    pub fn is_default(&self) -> bool {
        self == &Settings::default()
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub(crate) struct CargoSettings {
    pub git_fetch_with_cli: Option<bool>,
    pub check_revoke: Option<bool>,
    pub default_registry: Option<String>,
    pub registries: HashMap<String, CargoRegistry>,
}

impl CargoSettings {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub(crate) struct CargoRegistry {
    pub index: String,
}

impl From<String> for CargoRegistry {
    fn from(value: String) -> Self {
        Self { index: value }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Installation {
    #[serde(rename = "rustup")]
    rustup_ver: Option<String>,
    toolchain: Toolchain,
    tool: Tool,
}

impl TryFromEnv for Installation {
    fn try_from_env() -> Result<Self> {
        let rustup_ver = get_single_line_from_stdout("rustup", &["-V"]);

        Ok(Installation {
            rustup_ver,
            toolchain: Toolchain::try_from_env()?,
            tool: Tool::try_from_env()?,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Toolchain {
    default: Option<String>,
    #[serde(flatten)]
    installed: HashMap<String, ToolchainInfo>,
}

impl TryFromEnv for Toolchain {
    fn try_from_env() -> Result<Self> {
        let default = get_single_line_from_stdout("rustup", &["default"])
            .map(|line| line.trim_end_matches(" (default)").to_owned());

        let toolchain_list = utils::standard_output("rustup", &["toolchain", "list", "-v"])
            .with_context(|| "unable to get list of installed toolchain")?;
        let mut installed = HashMap::new();

        for line in toolchain_list.lines() {
            let mut splited = line.split_whitespace();
            // skip empty lines if there's one
            let Some(tc_name) = splited.next() else { continue };
            // if `rustup toolchain list -v` doesn't give us a toolchain name with path,
            // then there might be a bug, we need to be cautious about it.
            let tc_path = splited.last().unwrap_or_else(|| {
                panic!("got invalid output '{line}' when trying to gather toolchain list")
            });
            let tc_info = read_toolchain_info(tc_path)?;
            installed.insert(tc_name.into(), tc_info);
        }

        Ok(Toolchain { default, installed })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ToolchainInfo {
    version: String,
    components: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Tool {
    keep_package: bool,
    package_dir: Option<PathBuf>,
    tools_dir: Option<PathBuf>,
    #[serde(flatten)]
    installed: HashMap<String, ToolDetail>,
}

impl Default for Tool {
    fn default() -> Self {
        let root = utils::installer_home();
        Self {
            keep_package: true,
            package_dir: Some(root.join("packages")),
            tools_dir: Some(root.join("tools")),
            installed: HashMap::new(),
        }
    }
}

impl TryFromEnv for Tool {
    fn try_from_env() -> Result<Self> {
        let mut installed = HashMap::new();

        // attemp to read cargo installation record
        let crates_toml = home::cargo_home()?.join(".crates.toml");
        if let Ok(cargo_ins_rec) = super::load_toml::<CargoInstallTrackerLite>(&crates_toml) {
            for crate_info in cargo_ins_rec.v1.keys() {
                installed.insert(
                    crate_info.name.clone(),
                    ToolDetail {
                        version: crate_info.version.clone(),
                        installed_from_source: None,
                    },
                );
            }
        }

        // TODO: load tool installation record

        Ok(Tool {
            installed,
            ..Default::default()
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct ToolDetail {
    version: String,
    installed_from_source: Option<bool>,
}

fn get_single_line_from_stdout(p: &str, args: &[&str]) -> Option<String> {
    let output = utils::standard_output(p, args).ok()?;
    Some(output.lines().next()?.into())
}

/// Gather information from a toolchain folder.
fn read_toolchain_info<P: AsRef<Path>>(toolchain_folder: P) -> Result<ToolchainInfo> {
    // maybe there's no need to add extension?
    let rustc_exe = toolchain_folder.as_ref().join("bin").join("rustc");
    let components_file = toolchain_folder
        .as_ref()
        .join("lib")
        .join("rustlib")
        .join("components");
    let rustc_version = utils::standard_output_first_line_only(rustc_exe, &["-V"])
        .with_context(|| "unable to determine rustc version")?;
    let components: Vec<String> = utils::read_to_string(components_file)?
        .lines()
        .map(ToOwned::to_owned)
        .collect();
    Ok(ToolchainInfo {
        version: rustc_version,
        components,
    })
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

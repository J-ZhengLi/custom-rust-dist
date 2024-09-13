use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::utils;

use super::TomlParser;

pub(crate) const FILENAME: &str = ".fingerprint";

/// Re-load fingerprint file just to get the list of installed tools,
/// therefore we can use this list to uninstall, while avoiding race condition.
pub(crate) fn installed_tools_fresh(root: &Path) -> Result<IndexMap<String, ToolRecord>> {
    Ok(InstallationRecord::load(root)?.tools)
}

/// Holds Installation record.
///
/// This tracks what tools/components we have installed, and where they are installed.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InstallationRecord {
    pub(crate) root: PathBuf,
    rust: Option<RustRecord>,
    #[serde(default)]
    pub(crate) tools: IndexMap<String, ToolRecord>,
}

impl TomlParser for InstallationRecord {
    /// Load fingerprint from a given root.
    ///
    /// Note that the fingerprint filename is fixed, as defined as [`FILENAME`],
    /// hense why the parameter of this function is a root directory rather than dest path.
    fn load<P: AsRef<Path>>(root: P) -> Result<InstallationRecord>
    where
        Self: Sized + serde::de::DeserializeOwned,
    {
        assert!(root.as_ref().is_dir());

        let fp_path = root.as_ref().join(FILENAME);
        if fp_path.is_file() {
            let raw = utils::read_to_string("installation fingerprint", &fp_path)?;
            Self::from_str(&raw)
        } else {
            let default = InstallationRecord {
                root: root.as_ref().to_path_buf(),
                rust: None,
                tools: IndexMap::default(),
            };
            default.write()?;
            Ok(default)
        }
    }
}

impl InstallationRecord {
    pub(crate) fn write(&self) -> Result<()> {
        let path = self.root.join(FILENAME);
        let content = self
            .to_toml()
            .context("unable to serialize installation fingerprint")?;
        utils::write_bytes(&path, content.as_bytes(), false).with_context(|| {
            anyhow!(
                "unable to write fingerprint file to the given location: '{}'",
                path.display()
            )
        })
    }

    pub(crate) fn add_rust_record(&mut self, version: &str, components: &[String]) {
        self.rust = Some(RustRecord {
            version: version.to_string(),
            components: components.to_vec(),
        });
    }

    pub(crate) fn add_tool_record(&mut self, name: &str, record: ToolRecord) {
        self.tools.insert(name.into(), record);
    }

    pub fn remove_rust_record(&mut self) {
        self.rust = None;
    }

    #[allow(unused)]
    pub fn remove_component_record(&mut self, component: &str) {
        let Some(rust) = self.rust.as_mut() else {
            return;
        };
        let Some(target_idx) = rust.components.iter().position(|c| c == component) else {
            // Nothing to remove
            return;
        };
        rust.components.swap_remove(target_idx);
    }

    pub fn remove_tool_record(&mut self, tool_name: &str) {
        self.tools.shift_remove(tool_name);
    }

    pub(crate) fn print_installation(&self) -> String {
        let mut installed = String::new();
        if let Some(rust) = &self.rust {
            installed.push_str(&rust.print_rust_info());
        }
        for tool in self.tools.iter() {
            installed.push_str(&format!("tools: {:?} \n", tool.0));
        }
        installed
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct RustRecord {
    version: String,
    #[serde(default)]
    pub(crate) components: Vec<String>,
}

impl RustRecord {
    pub(crate) fn print_rust_info(&self) -> String {
        format!(
            "rust-version: {}\ncomponents: {:?}\n",
            self.version, self.components
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ToolRecord {
    #[serde(default)]
    pub(crate) use_cargo: bool,
    #[serde(default)]
    pub(crate) paths: Vec<PathBuf>,
    // version: String,
}

impl ToolRecord {
    pub(crate) fn cargo_tool() -> Self {
        ToolRecord {
            use_cargo: true,
            paths: vec![],
        }
    }

    pub(crate) fn with_paths(paths: Vec<PathBuf>) -> Self {
        ToolRecord {
            use_cargo: false,
            paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_local_install_info() {
        let install_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
        let mut fp = InstallationRecord::load(&install_dir).unwrap();
        let rust_components = vec![String::from("rustfmt"), String::from("cargo")];

        fp.add_rust_record("stable", &rust_components);
        fp.add_tool_record("aaa", ToolRecord::with_paths(vec![install_dir.join("aaa")]));

        let v0 = format!(
            "\
root = \"{}\"

[rust]
version = \"stable\"
components = [\"rustfmt\", \"cargo\"]

[tools.aaa]
use-cargo = false
paths = [\"{}\"]
",
            install_dir.display(),
            install_dir.join("aaa").display()
        );
        assert_eq!(v0, fp.to_toml().unwrap());
    }
}

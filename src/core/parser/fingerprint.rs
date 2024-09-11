use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::utils;

use super::TomlParser;

/// A simple struct representing the fields in `config.toml`.
///
/// Only covers a small range of options we need to configurate.
/// Fwiw, the full set of configuration options can be found
/// in the [Cargo Configuration Book](https://doc.rust-lang.org/cargo/reference/config.html).
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct FingerPrint {
    rust: RustInstallInfo,
    tools: IndexMap<String, ToolDetailInfo>,
}

impl TomlParser for FingerPrint {}

#[allow(unused)]
impl FingerPrint {
    pub(crate) fn load_fingerprint(install_dir: &PathBuf) -> Self {
        let fp_path = install_dir.join(".fingerprint");
        if fp_path.exists() {
            FingerPrint::load(fp_path).expect("Failed to find fingerprint file")
        } else {
            let mut fingerprint = Self::default();
            let fingerprint_content = fingerprint.to_toml().expect("Init new fingerprint content");
            utils::write_file(fp_path, fingerprint_content.as_str(), false);
            fingerprint
        }
    }

    pub(crate) fn record_rust(&mut self, version: String, components: Vec<String>) -> &mut Self {
        self.rust = RustInstallInfo {
            version,
            components,
        };
        self
    }

    pub(crate) fn record_tool(
        &mut self,
        use_cargo: bool,
        name: String,
        paths: Option<Vec<PathBuf>>,
    ) -> &mut Self {
        self.tools
            .entry(name)
            .and_modify(|tool| {
                if use_cargo {
                    tool.paths = Vec::new();
                } else if let Some(p) = &paths {
                    for pp in p.iter() {
                        if !tool.paths.contains(pp) {
                            tool.paths.push(pp.to_path_buf());
                        }
                    }
                }
            })
            .or_insert(ToolDetailInfo {
                use_cargo,
                paths: {
                    if use_cargo {
                        Vec::new()
                    } else if let Some(pp) = &paths {
                        pp.to_vec()
                    } else {
                        /// FIXME: should throw error if path is not found.
                        Vec::new()
                    }
                },
            });

        self
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct RustInstallInfo {
    version: String,
    #[serde(default)]
    pub(crate) components: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ToolDetailInfo {
    use_cargo: bool,
    paths: Vec<PathBuf>,
}

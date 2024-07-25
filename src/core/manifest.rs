#![allow(unused)]

use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;
use url::Url;

use super::TomlParser;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ToolsetManifest {
    pub(crate) rust: RustToolchain,
    pub(crate) target: BTreeMap<String, TargetTools>,
    pub(crate) tools: BTreeMap<String, Tool>,
}

impl TomlParser for ToolsetManifest {}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct RustToolchain {
    pub(crate) version: String,
    pub(crate) profile: Option<String>,
    pub(crate) components: Option<Vec<String>>,
}

impl RustToolchain {
    pub(crate) fn new(ver: &str) -> Self {
        Self {
            version: ver.to_string(),
            profile: None,
            components: None,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct TargetTools {
    pub(crate) tools: Vec<String>,
}

impl From<Vec<String>> for TargetTools {
    fn from(value: Vec<String>) -> Self {
        Self { tools: value }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum Tool {
    Normal {
        version: String,
    },
    Git {
        git: Url,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
    },
    Path {
        path: PathBuf,
    },
    Url {
        url: Url,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_minimal_manifest() {
        let input = r#"
[rust]
version = "1.0.0"
[tools]
[target]
"#;
        assert_eq!(
            ToolsetManifest::from_str(input).unwrap(),
            ToolsetManifest {
                rust: RustToolchain::new("1.0.0"),
                target: BTreeMap::default(),
                tools: BTreeMap::default(),
            }
        )
    }

    #[test]
    fn deserialize_complicated_manifest() {
        let input = r#"
[rust]
version = "1.0.0"
profile = "minimal"
components = ["clippy-preview", "llvm-tools-preview"]

[target.x86_64-pc-windows-msvc]
tools = ["t1", "t2", "t3"]

[target.x86_64-unknown-linux-gnu]
tools = ["t1", "t2"]

[target.aarch64-unknown-linux-gnu]
tools = ["t1", "t4"]

[tools]
t1 = { version = "0.1.0" } # use cargo install
t2 = { path = "/path/to/local" }
t3 = { url = "https://example.com/path/to/tool" }
t4 = { git = "https://git.example.com/org/tool", branch = "stable" }
"#;

        let expected = ToolsetManifest {
            rust: RustToolchain {
                version: "1.0.0".into(),
                profile: Some("minimal".into()),
                components: Some(vec!["clippy-preview".into(), "llvm-tools-preview".into()]),
            },
            target: BTreeMap::from_iter([
                (
                    "x86_64-pc-windows-msvc".into(),
                    vec!["t1".into(), "t2".into(), "t3".into()].into(),
                ),
                (
                    "x86_64-unknown-linux-gnu".into(),
                    vec!["t1".into(), "t2".into()].into(),
                ),
                (
                    "aarch64-unknown-linux-gnu".into(),
                    vec!["t1".into(), "t4".into()].into(),
                ),
            ]),
            tools: BTreeMap::from_iter([
                (
                    "t1".into(),
                    Tool::Normal {
                        version: "0.1.0".into(),
                    },
                ),
                (
                    "t2".into(),
                    Tool::Path {
                        path: PathBuf::from("/path/to/local"),
                    },
                ),
                (
                    "t3".into(),
                    Tool::Url {
                        url: Url::parse("https://example.com/path/to/tool").unwrap(),
                    },
                ),
                (
                    "t4".into(),
                    Tool::Git {
                        git: Url::parse("https://git.example.com/org/tool").unwrap(),
                        branch: Some("stable".into()),
                        tag: None,
                        rev: None,
                    },
                ),
            ]),
        };

        assert_eq!(ToolsetManifest::from_str(input).unwrap(), expected);
    }
}

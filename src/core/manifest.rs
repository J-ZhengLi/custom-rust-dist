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

impl ToolsetManifest {
    /// Get a map of [`Tool`] that are available only in current target.
    pub(crate) fn current_target_tools(&self) -> BTreeMap<&String, &ToolInfo> {
        let cur_target = env!("TARGET");
        self.target
            .get(cur_target)
            .map(|tools| {
                tools
                    .tools
                    .iter()
                    .filter_map(|toolname| {
                        self.tools
                            .get_key_value(toolname)
                            .and_then(|(name, tool)| match tool {
                                Tool::General(toolinfo) => Some((name, toolinfo)),
                                Tool::WithTarget { target } => {
                                    target.get(cur_target).map(|toolinfo| (name, toolinfo))
                                }
                            })
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default()
    }
}

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
    WithTarget { target: BTreeMap<String, ToolInfo> },
    General(ToolInfo),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum ToolInfo {
    Version(String),
    Git {
        git: Url,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
    },
    Path {
        path: PathBuf,
        version: Option<String>,
    },
    Url {
        url: Url,
        version: Option<String>,
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
t1 = "0.1.0" # use cargo install
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
                    Tool::General(ToolInfo::Version("0.1.0".into())),
                ),
                (
                    "t2".into(),
                    Tool::General(ToolInfo::Path {
                        path: PathBuf::from("/path/to/local"),
                        version: None,
                    }),
                ),
                (
                    "t3".into(),
                    Tool::General(ToolInfo::Url {
                        url: Url::parse("https://example.com/path/to/tool").unwrap(),
                        version: None,
                    }),
                ),
                (
                    "t4".into(),
                    Tool::General(ToolInfo::Git {
                        git: Url::parse("https://git.example.com/org/tool").unwrap(),
                        branch: Some("stable".into()),
                        tag: None,
                        rev: None,
                    }),
                ),
            ]),
        };

        assert_eq!(ToolsetManifest::from_str(input).unwrap(), expected);
    }

    #[test]
    fn deserialize_realworld_manifest() {
        let input = include_str!("../../tests/data/toolset_manifest.toml");
        let expected = ToolsetManifest {
            rust: RustToolchain {
                version: "1.80.0".into(),
                profile: Some("minimal".into()),
                components: Some(vec!["clippy-preview".into(), "rustfmt".into()]),
            },
            target: BTreeMap::from_iter([
                (
                    "x86_64-pc-windows-msvc".into(),
                    vec!["buildtools".into(), "cargo-llvm-cov".into(), "vscode".into(), "vscode-rust-analyzer".into(), "cargo-expand".into()].into(),
                ),
                (
                    "x86_64-pc-windows-gnu".into(),
                    vec!["mingw".into(), "vscode".into(), "vscode-rust-analyzer".into(), "cargo-expand".into()].into(),
                ),
                (
                    "x86_64-unknown-linux-gnu".into(),
                    vec!["cargo-llvm-cov".into(), "flamegraph".into(), "cargo-expand".into()].into(),
                ),
                (
                    "aarch64-apple-darwin".into(),
                    vec!["cargo-llvm-cov".into(), "flamegraph".into(), "cargo-expand".into()].into(),
                ),
            ]),
            tools: BTreeMap::from_iter([
                (
                    "buildtools".into(),
                    Tool::General(ToolInfo::Path {
                        path: "tests/cache/BuildTools-With-SDK.zip".into(),
                        version: Some("1".into()),
                    }),
                ),
                (
                    "mingw".into(),
                    Tool::General(ToolInfo::Path {
                        path: PathBuf::from("tests/cache/x86_64-13.2.0-release-posix-seh-msvcrt-rt_v11-rev1.7z"),
                        version: Some("13.2.0".into()),
                    }),
                ),
                (
                    "cargo-expand".into(),
                    Tool::General(ToolInfo::Version("1.0.88".into())),
                ),
                (
                    "flamegraph".into(),
                    Tool::General(ToolInfo::Git {
                        git: Url::parse("https://github.com/flamegraph-rs/flamegraph").unwrap(),
                        branch: None,
                        tag: Some("v0.6.5".into()),
                        rev: None,
                    }),
                ),
                (
                    "vscode".into(),
                    Tool::WithTarget { target: BTreeMap::from([
                        (
                            "x86_64-pc-windows-msvc".into(),
                            ToolInfo::Path {
                                path: PathBuf::from("tests/cache/VSCode-win32-x64-1.91.1.zip"),
                                version: Some("1.91.1".into()),
                            }
                        ),
                        (
                            "x86_64-pc-windows-gnu".into(),
                            ToolInfo::Path {
                                path: PathBuf::from("tests/cache/VSCode-win32-x64-1.91.1.zip"),
                                version: Some("1.91.1".into()),
                            }
                        )
                    ])},
                ),
                (
                    "vscode-rust-analyzer".into(),
                    Tool::WithTarget { target: BTreeMap::from([
                        (
                            "x86_64-pc-windows-msvc".into(),
                            ToolInfo::Path {
                                path: PathBuf::from("tests/cache/rust-lang.rust-analyzer-0.4.2054@win32-x64.vsix"),
                                version: Some("0.4.2054".into()),
                            }
                        ),
                        (
                            "x86_64-pc-windows-gnu".into(),
                            ToolInfo::Path {
                                path: PathBuf::from("tests/cache/rust-lang.rust-analyzer-0.4.2054@win32-x64.vsix"),
                                version: Some("0.4.2054".into()),
                            }
                        )
                    ])},
                ),
                (
                    "cargo-llvm-cov".into(),
                    Tool::WithTarget { target: BTreeMap::from([
                        (
                            "x86_64-pc-windows-msvc".into(),
                            ToolInfo::Url {
                                url: Url::parse("https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-pc-windows-msvc.zip").unwrap(),
                                version: Some("0.6.11".into()),
                            }
                        ),
                        (
                            "x86_64-unknown-linux-gnu".into(),
                            ToolInfo::Url {
                                url: Url::parse("https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz").unwrap(),
                                version: Some("0.6.11".into()),
                            }
                        ),
                        (
                            "aarch64-apple-darwin".into(),
                            ToolInfo::Url {
                                url: Url::parse("https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-aarch64-apple-darwin.tar.gz").unwrap(),
                                version: Some("0.6.11".into()),
                            }
                        ),
                    ])},
                ),
            ]),
        };
        assert_eq!(ToolsetManifest::from_str(input).unwrap(), expected);
    }

    #[test]
    fn current_target_tools_are_correct() {
        let input = include_str!("../../tests/data/toolset_manifest.toml");
        let manifest = ToolsetManifest::from_str(input).unwrap();
        let tools = manifest.current_target_tools();

        #[cfg(all(windows, target_env = "gnu"))]
        assert_eq!(
            tools,
            BTreeMap::from([
                (
                    &"mingw".into(),
                    &ToolInfo::Path {
                        path: PathBuf::from(
                            "tests/cache/x86_64-13.2.0-release-posix-seh-msvcrt-rt_v11-rev1.7z"
                        ),
                        version: Some("13.2.0".into()),
                    }
                ),
                (
                    &"vscode".into(),
                    &ToolInfo::Path {
                        path: PathBuf::from("tests/cache/VSCode-win32-x64-1.91.1.zip"),
                        version: Some("1.91.1".into()),
                    }
                ),
                (
                    &"vscode-rust-analyzer".into(),
                    &ToolInfo::Path {
                        path: "tests/cache/rust-lang.rust-analyzer-0.4.2054@win32-x64.vsix".into(),
                        version: Some("0.4.2054".into()),
                    }
                ),
                (&"cargo-expand".into(), &ToolInfo::Version("1.0.88".into())),
            ])
        );

        #[cfg(all(windows, target_env = "msvc"))]
        assert_eq!(
            tools,
            BTreeMap::from([
                (
                    &"buildtools".into(),
                    &ToolInfo::Path {
                        path: PathBuf::from(
                            "tests/cache/BuildTools-With-SDK.zip"
                        ),
                        version: Some("1".into()),
                    }
                ),
                (
                    &"cargo-llvm-cov".into(),
                    &ToolInfo::Url {
                        url: "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz".parse().unwrap(),
                        version: Some("0.6.11".into())
                    }
                ),
                (
                    &"vscode".into(),
                    &ToolInfo::Path {
                        path: PathBuf::from("tests/cache/VSCode-win32-x64-1.91.1.zip"),
                        version: Some("1.91.1".into()),
                    }
                ),
                (
                    &"vscode-rust-analyzer".into(),
                    &ToolInfo::Path {
                        path: "tests/cache/rust-lang.rust-analyzer-0.4.2054@win32-x64.vsix".into(),
                        version: Some("0.4.2054".into()),
                    }
                ),
                (
                    &"cargo-expand".into(),
                    &ToolInfo::Version("1.0.88".into()),
                ),
            ])
        );

        #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
        assert_eq!(tools, BTreeMap::from([
            (&"cargo-llvm-cov".into(), &ToolInfo::Url {
                url: "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz".parse().unwrap(),
                version: Some("0.6.11".into())
            }),
            (&"flamegraph".into(), &ToolInfo::Git { git: "https://github.com/flamegraph-rs/flamegraph".parse().unwrap(), tag: Some("v0.6.5".into()), branch: None, rev: None }),
            (&"cargo-expand".into(), &ToolInfo::Version("1.0.88".into())),
        ]));

        // TODO: Add test for macos.
    }
}

#![allow(unused)]

use std::{collections::BTreeMap, path::PathBuf};

use serde::Deserialize;
use url::Url;

use super::TomlParser;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ToolsetManifest {
    pub(crate) rust: RustToolchain,
    #[serde(default)]
    pub(crate) tools: TargetedTools,
}

impl TomlParser for ToolsetManifest {}

impl ToolsetManifest {
    /// Get a map of [`Tool`] that are available only in current target.
    pub(crate) fn current_target_tools(&self) -> BTreeMap<&String, &ToolInfo> {
        let cur_target = env!("TARGET");
        // Clippy bug, the `map(|(k, v)| (k, v))` cannot be removed
        #[allow(clippy::map_identity)]
        self.tools
            .target
            .get(cur_target)
            .map(|map| map.iter().map(|(k, v)| (k, v)).collect())
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

#[derive(Debug, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct TargetedTools {
    #[serde(default)]
    target: BTreeMap<String, BTreeMap<String, ToolInfo>>,
}

impl FromIterator<(String, BTreeMap<String, ToolInfo>)> for TargetedTools {
    fn from_iter<T: IntoIterator<Item = (String, BTreeMap<String, ToolInfo>)>>(iter: T) -> Self {
        Self {
            target: BTreeMap::from_iter(iter),
        }
    }
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

    macro_rules! tool_info {
        ($version:literal) => {
            ToolInfo::Version($version.into())
        };
        ($url_str:literal, $version:expr) => {
            ToolInfo::Url {
                version: $version.map(ToString::to_string),
                url: $url_str.parse().unwrap(),
            }
        };
        ($git:literal, $branch:expr, $tag:expr, $rev:expr) => {
            ToolInfo::Git {
                git: $git.parse().unwrap(),
                branch: $branch.map(ToString::to_string),
                tag: $tag.map(ToString::to_string),
                rev: $rev.map(ToString::to_string),
            }
        };
        ($path:expr, $version:expr) => {
            ToolInfo::Path {
                version: $version.map(ToString::to_string),
                path: $path,
            }
        };
    }

    #[test]
    fn deserialize_minimal_manifest() {
        let input = r#"
[rust]
version = "1.0.0"
"#;
        assert_eq!(
            ToolsetManifest::from_str(input).unwrap(),
            ToolsetManifest {
                rust: RustToolchain::new("1.0.0"),
                tools: TargetedTools::default(),
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

[tools.target.x86_64-pc-windows-msvc]
t1 = "0.1.0" # use cargo install
t2 = { path = "/path/to/local" }
t3 = { url = "https://example.com/path/to/tool" }

[tools.target.x86_64-unknown-linux-gnu]
t1 = "0.1.0" # use cargo install
t2 = { path = "/path/to/local" }

[tools.target.aarch64-unknown-linux-gnu]
t1 = "0.1.0"
t4 = { git = "https://git.example.com/org/tool", branch = "stable" }
"#;

        let mut x86_64_windows_msvc_tools = BTreeMap::new();
        x86_64_windows_msvc_tools.insert("t1".to_string(), tool_info!("0.1.0"));
        x86_64_windows_msvc_tools.insert(
            "t2".to_string(),
            tool_info!(PathBuf::from("/path/to/local"), None::<&str>),
        );
        x86_64_windows_msvc_tools.insert(
            "t3".to_string(),
            tool_info!("https://example.com/path/to/tool", None::<&str>),
        );

        let mut x86_64_linux_gnu_tools = BTreeMap::new();
        x86_64_linux_gnu_tools.insert("t1".to_string(), tool_info!("0.1.0"));
        x86_64_linux_gnu_tools.insert(
            "t2".to_string(),
            tool_info!(PathBuf::from("/path/to/local"), None::<&str>),
        );

        let mut aarch64_linux_gnu_tools = BTreeMap::new();
        aarch64_linux_gnu_tools.insert("t1".to_string(), tool_info!("0.1.0"));
        aarch64_linux_gnu_tools.insert(
            "t4".to_string(),
            tool_info!(
                "https://git.example.com/org/tool",
                Some("stable"),
                None::<&str>,
                None::<&str>
            ),
        );

        let expected = ToolsetManifest {
            rust: RustToolchain {
                version: "1.0.0".into(),
                profile: Some("minimal".into()),
                components: Some(vec!["clippy-preview".into(), "llvm-tools-preview".into()]),
            },
            tools: TargetedTools::from_iter([
                (
                    "x86_64-pc-windows-msvc".to_string(),
                    x86_64_windows_msvc_tools,
                ),
                (
                    "x86_64-unknown-linux-gnu".to_string(),
                    x86_64_linux_gnu_tools,
                ),
                (
                    "aarch64-unknown-linux-gnu".to_string(),
                    aarch64_linux_gnu_tools,
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
            tools: TargetedTools::from_iter([
                (
                    "x86_64-pc-windows-msvc".into(),
                    BTreeMap::from_iter([
                        ("buildtools".to_string(), tool_info!(PathBuf::from("tests/cache/BuildTools-With-SDK.zip"), Some("1"))),
                        ("cargo-llvm-cov".to_string(), tool_info!("https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-pc-windows-msvc.zip", Some("0.6.11"))),
                        ("vscode".to_string(), tool_info!(PathBuf::from("tests/cache/VSCode-win32-x64-1.91.1.zip"), Some("1.91.1"))),
                        ("vscode-rust-analyzer".to_string(), tool_info!(PathBuf::from("tests/cache/rust-lang.rust-analyzer-0.4.2054@win32-x64.vsix"), Some("0.4.2054"))),
                        ("cargo-expand".to_string(), tool_info!("1.0.88")),
                    ]),
                ),
                (
                    "x86_64-pc-windows-gnu".into(),
                    BTreeMap::from_iter([
                        ("mingw".to_string(), tool_info!(PathBuf::from("tests/cache/x86_64-13.2.0-release-posix-seh-msvcrt-rt_v11-rev1.7z"), Some("13.2.0"))),
                        ("vscode".to_string(), tool_info!(PathBuf::from("tests/cache/VSCode-win32-x64-1.91.1.zip"), Some("1.91.1"))),
                        ("vscode-rust-analyzer".to_string(), tool_info!(PathBuf::from("tests/cache/rust-lang.rust-analyzer-0.4.2054@win32-x64.vsix"), Some("0.4.2054"))),
                        ("cargo-expand".to_string(), tool_info!("1.0.88")),
                    ]),
                ),
                (
                    "x86_64-unknown-linux-gnu".into(),
                    BTreeMap::from_iter([
                        ("cargo-llvm-cov".to_string(), tool_info!("https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz", Some("0.6.11"))),
                        ("flamegraph".to_string(), tool_info!("https://github.com/flamegraph-rs/flamegraph", None::<&str>, Some("v0.6.5"), None::<&str>)),
                        ("cargo-expand".to_string(), tool_info!("1.0.88")),
                    ]),
                ),
                (
                    "aarch64-apple-darwin".into(),
                    BTreeMap::from_iter([
                        ("cargo-llvm-cov".to_string(), tool_info!("https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-aarch64-apple-darwin.tar.gz", Some("0.6.11"))),
                        ("flamegraph".to_string(), tool_info!("https://github.com/flamegraph-rs/flamegraph", None::<&str>, Some("v0.6.5"), None::<&str>)),
                        ("cargo-expand".to_string(), tool_info!("1.0.88")),
                    ]),
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
                        url: "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.6.11/cargo-llvm-cov-x86_64-pc-windows-msvc.zip".parse().unwrap(),
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

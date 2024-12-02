use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use log::debug;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

use crate::{core::tools::ToolKind, setter, utils};

use super::{toolset_manifest::ToolsetManifest, TomlParser};

/// Re-load fingerprint file just to get the list of installed tools,
/// therefore we can use this list to uninstall, while avoiding race condition.
pub(crate) fn installed_tools_fresh(root: &Path) -> Result<IndexMap<String, ToolRecord>> {
    Ok(InstallationRecord::load(root)?.tools)
}

/// Holds Installation record.
///
/// This tracks what tools/components we have installed, and where they are installed.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct InstallationRecord {
    /// Name of the bundle, such as `my-rust-stable`
    pub name: Option<String>,
    pub version: Option<String>,
    pub root: PathBuf,
    pub rust: Option<RustRecord>,
    #[serde(default)]
    pub tools: IndexMap<String, ToolRecord>,
}

impl TomlParser for InstallationRecord {
    const FILENAME: &str = ".fingerprint.toml";

    /// Load fingerprint from a given root.
    ///
    /// Note that the fingerprint filename is fixed, as defined as [`FILENAME`],
    /// hense why the parameter of this function is a root directory rather than dest path.
    fn load<P: AsRef<Path>>(root: P) -> Result<InstallationRecord>
    where
        Self: Sized + serde::de::DeserializeOwned,
    {
        assert!(
            root.as_ref().is_dir(),
            "install record needs to be loaded from a directory"
        );

        let fp_path = root.as_ref().join(Self::FILENAME);
        if fp_path.is_file() {
            let raw = utils::read_to_string("installation fingerprint", &fp_path)?;
            Self::from_str(&raw)
        } else {
            let default = InstallationRecord {
                root: root.as_ref().to_path_buf(),
                ..Default::default()
            };
            default.write()?;
            Ok(default)
        }
    }
}

impl InstallationRecord {
    /// Used to detect whether a fingerprint file exists in parent directory.
    ///
    /// This is useful when you want to know it without causing
    /// the program to panic using [`get_installed_dir`](super::get_installed_dir).
    pub fn exists() -> Result<bool> {
        let parent_dir = utils::parent_dir_of_cur_exe()?;
        Ok(parent_dir.join(Self::FILENAME).is_file())
    }

    /// Load installation record from a presumed install directory,
    /// which is typically the parent directory of the current executable.
    ///
    /// # Note
    /// Use this instead of [`InstallationRecord::load`] in **manager** mod.
    // TODO: Cache the result using a `Cell` or `RwLock` or combined.
    pub fn load_from_install_dir() -> Result<Self> {
        let root = super::get_installed_dir();
        Self::load(root)
    }

    pub(crate) fn write(&self) -> Result<()> {
        let path = self.root.join(Self::FILENAME);
        let content = self
            .to_toml()
            .context("unable to serialize installation fingerprint")?;
        debug!("writing installation record into '{}'", path.display());
        utils::write_bytes(&path, content.as_bytes(), false).with_context(|| {
            anyhow!(
                "unable to write fingerprint file to the given location: '{}'",
                path.display()
            )
        })
    }

    pub(crate) fn clone_toolkit_meta_from_manifest(&mut self, manifest: &ToolsetManifest) {
        self.name.clone_from(&manifest.name);
        self.version.clone_from(&manifest.version);
    }

    pub(crate) fn remove_toolkit_meta(&mut self) {
        self.name = None;
        self.version = None;
    }

    pub(crate) fn add_rust_record(&mut self, version: &str, components: &[String]) {
        self.rust = Some(RustRecord {
            version: version.to_string(),
            components: components.to_vec(),
        });
    }

    pub(crate) fn update_rust(&mut self, version: &str) {
        if let Some(rust) = self.rust.as_mut() {
            rust.version = version.into();
            debug!("toolchain installation record was updated to '{version}'");
        }
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

    /// Return an iterator of installed tools' names.
    pub fn installed_tools(&self) -> impl Iterator<Item = &str> {
        self.tools.keys().map(|k| k.as_str())
    }

    /// Returns the rust toolchain channel name (such as `stable`, `nightly`, `1.80.1`, etc.),
    /// and an iterator of installed components.
    pub fn installed_toolchain(&self) -> Option<(&str, impl Iterator<Item = &str>)> {
        self.rust.as_ref().map(|rr| {
            (
                rr.version.as_str(),
                rr.components.iter().map(|s| s.as_str()),
            )
        })
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

    pub fn get_tool_version(&self, name: &str) -> Option<&str> {
        self.tools.get(name).and_then(|rec| rec.version.as_deref())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RustRecord {
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

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ToolRecord {
    #[deprecated(since = "0.3.1", note = "use `.tool_kind()` instead")]
    #[serde(
        default,
        skip_serializing,
        deserialize_with = "de_deprecated_use_cargo"
    )]
    use_cargo: Option<ToolKind>,
    #[serde(default)]
    kind: ToolKind,
    version: Option<String>,
    #[serde(default)]
    pub(crate) paths: Vec<PathBuf>,
}

impl ToolRecord {
    pub(crate) fn cargo_tool() -> Self {
        ToolRecord {
            kind: ToolKind::CargoTool,
            ..Default::default()
        }
    }

    pub(crate) fn new(kind: ToolKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }

    pub(crate) fn tool_kind(&self) -> ToolKind {
        #[allow(deprecated)]
        self.use_cargo.unwrap_or(self.kind)
    }

    setter!(paths(self, Vec<PathBuf>));
    setter!(version(self, ver: Option<impl Into<String>>) { ver.map(Into::into) });
}

// `use-cargo = true/false` was used during [0.2.0, 0.3.0], in order not to break
// the compatibility for those versions, we need to deserialize it to the new api.
fn de_deprecated_use_cargo<'de, D>(deserializer: D) -> Result<Option<ToolKind>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ToolKindVisitor;

    impl Visitor<'_> for ToolKindVisitor {
        type Value = Option<ToolKind>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a `true` or `false` string")
        }

        fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.then_some(ToolKind::CargoTool))
        }
    }

    deserializer.deserialize_bool(ToolKindVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    // there is an inconsistency between OSs when serialize paths
    #[cfg(not(windows))]
    const QUOTE: &str = "\"";
    #[cfg(windows)]
    const QUOTE: &str = "'";

    #[test]
    fn create_local_install_info() {
        let install_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
        let mut fp = InstallationRecord::load(&install_dir).unwrap();
        let rust_components = vec![String::from("rustfmt"), String::from("cargo")];

        fp.add_rust_record("stable", &rust_components);
        fp.add_tool_record(
            "aaa",
            ToolRecord::new(ToolKind::Custom).paths(vec![install_dir.join("aaa")]),
        );

        let v0 = format!(
            "\
root = {QUOTE}{}{QUOTE}

[rust]
version = \"stable\"
components = [\"rustfmt\", \"cargo\"]

[tools.aaa]
kind = \"custom\"
paths = [{QUOTE}{}{QUOTE}]
",
            install_dir.display(),
            install_dir.join("aaa").display()
        );
        assert_eq!(v0, fp.to_toml().unwrap());
    }

    #[test]
    fn with_name_and_ver() {
        let input = r#"
name = "rust bundle (experimental)"
version = "0.1"
root = '/path/to/something'"#;

        let expected = InstallationRecord::from_str(input).unwrap();
        assert_eq!(expected.name.unwrap(), "rust bundle (experimental)");
        assert_eq!(expected.version.unwrap(), "0.1");
        assert_eq!(expected.root, PathBuf::from("/path/to/something"));
    }

    #[test]
    fn all_tool_kinds() {
        let input = r#"
root = '/path/to/something'

[tools]
a = { kind = 'cargo-tool', paths = [] }
b = { kind = 'custom', paths = [] }
c = { kind = 'dir-with-bin', paths = [] }
d = { kind = 'executables', paths = []}
e = { kind = 'plugin', paths = []}
"#;

        let kinds = &[
            ToolKind::CargoTool,
            ToolKind::Custom,
            ToolKind::DirWithBin,
            ToolKind::Executables,
            ToolKind::Plugin,
        ];
        let expected = InstallationRecord::from_str(input).unwrap();
        let all_kinds = expected
            .tools
            .values()
            .map(|rec| rec.kind)
            .collect::<Vec<_>>();

        assert_eq!(all_kinds, kinds);
    }

    #[test]
    fn de_use_cargo_and_default_toolkind() {
        let input = r#"
root = '/path/to/something'

[tools]
a = { use-cargo = true, paths = [] }
b = { use-cargo = false, paths = ['some/path'] }
c = { paths = ['some/other/path'] }"#;

        let expected = InstallationRecord::from_str(input).unwrap();
        let mut it = expected.tools.values().map(|rec| rec.tool_kind());

        assert_eq!(it.next(), Some(ToolKind::CargoTool));
        assert_eq!(it.next(), Some(ToolKind::Unknown));
        assert_eq!(it.next(), Some(ToolKind::Unknown));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn do_not_ser_use_cargo() {
        let record = InstallationRecord {
            root: "/some/path".into(),
            tools: IndexMap::from([("a".into(), ToolRecord::cargo_tool())]),
            ..Default::default()
        };
        let ser = record.to_toml().unwrap();
        let expected = r#"root = "/some/path"

[tools.a]
kind = "cargo-tool"
paths = []
"#;
        assert_eq!(ser, expected);
    }

    #[test]
    fn with_tool_version() {
        let input = r#"
root = '/path/to/something'

[tools]
a = { kind = "custom", version = "1.2.0", paths = ["/some/path"] }
[tools.b]
kind = "executables"
paths = ["/some/other/path"]"#;

        let rec = InstallationRecord::from_str(input).unwrap();
        let mut tools = rec.tools.values().map(|r| r.version.as_deref());
        assert_eq!(tools.next(), Some(Some("1.2.0")));
        assert_eq!(tools.next(), Some(None));
        assert_eq!(tools.next(), None);
    }
}

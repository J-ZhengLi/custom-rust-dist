use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    fingerprint::InstallationRecord,
    setter,
    toolset_manifest::{ToolInfo, ToolMap, ToolsetManifest},
};

static COMPONENTS_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub id: u32,
    pub group_name: Option<String>,
    pub name: String,
    pub version: Option<String>,
    pub desc: String,
    pub required: bool,
    pub optional: bool,
    pub tool_installer: Option<ToolInfo>,
    pub is_toolchain_component: bool,
    /// Indicates whether this component was already installed or not.
    pub installed: bool,
}

impl Component {
    #[must_use]
    pub fn new(name: &str, desc: &str) -> Self {
        let comp = Component {
            id: COMPONENTS_COUNTER.load(Ordering::Relaxed),
            group_name: None,
            name: name.into(),
            version: None,
            desc: desc.into(),
            required: false,
            optional: false,
            tool_installer: None,
            is_toolchain_component: false,
            installed: false,
        };
        COMPONENTS_COUNTER.fetch_add(1, Ordering::SeqCst);

        comp
    }

    setter!(required(self, bool));
    setter!(optional(self, bool));
    setter!(installed(self, bool));
    setter!(is_toolchain_component(self, bool));
    setter!(group_name(self, group: Option<&str>) { group.map(ToOwned::to_owned) });
    setter!(tool_installer(self, installer: &ToolInfo) { Some(installer.clone()) });
    setter!(version(self, version: Option<&str>) { version.map(ToOwned::to_owned) });
}

/// Get a combined list of tools and toolchain components in Vec<[Component]> format,
/// whether it's installed or not.
///
/// A toolset manifest located under installation dir (`toolset-manifest.toml`)
/// will be loaded in order to retrieve component's full info.
///
/// # Panic
/// This should only be called in manager mode, otherwise it will panic.
pub(crate) fn all_components_from_installation(
    record: &InstallationRecord,
) -> Result<Vec<Component>> {
    let mut full_components =
        ToolsetManifest::load_from_install_dir()?.current_target_components(false)?;

    // components that are installed by rim previously.
    let installed_toolchain = record.installed_toolchain().map(|(name, _)| name);
    let installed_tools: HashSet<&str> = record.installed_tools().collect();

    for comp in &mut full_components {
        if comp.is_toolchain_component {
            if let Some(tc) = installed_toolchain {
                comp.version = Some(tc.into());
                comp.installed = true;
            }
            continue;
        }
        // third-party tools
        if installed_tools.contains(comp.name.as_str()) {
            comp.installed = true;
            if let Some(ver) = record.get_tool_version(&comp.name) {
                comp.version = Some(ver.into());
            }
        }
    }

    Ok(full_components)
}

pub fn component_list_to_tool_map(list: Vec<&Component>) -> ToolMap {
    list.iter()
        .filter_map(|c| {
            c.tool_installer
                .as_ref()
                .map(|tool_info| (c.name.clone(), tool_info.clone()))
        })
        .collect()
}

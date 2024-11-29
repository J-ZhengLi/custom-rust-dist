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
    fn new(name: &str, desc: &str) -> Self {
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

struct InstalledComponents<'a> {
    toolchain_channel: Option<&'a str>,
    all: HashSet<&'a String>,
}

impl<'a> InstalledComponents<'a> {
    fn from_record(record: Option<&'a InstallationRecord>) -> Self {
        let mut all = HashSet::new();
        let mut toolchain_channel = None;

        if let Some(installed_tools) = record.map(|r| r.installed_tools()) {
            all.extend(installed_tools);
        }
        if let Some((tc_channel, installed_comps)) = record.and_then(|r| r.installed_toolchain()) {
            toolchain_channel = Some(tc_channel);
            all.extend(installed_comps);
        }

        Self {
            all,
            toolchain_channel,
        }
    }
    fn toolchain_installed(&self) -> bool {
        self.toolchain_channel.is_some()
    }
}

/// Get a components from manifest that are available in the current target.
///
/// If the `record` is present, this will read the installation record and the
/// returned components will also contain [`installed`](Component::installed) info as well.
pub fn get_component_list_from_manifest(
    manifest: &ToolsetManifest,
    record: Option<&InstallationRecord>,
) -> Result<Vec<Component>> {
    // components that are installed by rim previously.
    let installed_by_rim = InstalledComponents::from_record(record);

    let tc_channel = installed_by_rim
        .toolchain_channel
        .unwrap_or(manifest.rust_version());

    let profile = manifest.toolchain_profile().cloned().unwrap_or_default();
    let profile_name = profile.verbose_name.as_deref().unwrap_or(&profile.name);
    // Add a component that represents rust toolchain
    let mut components = vec![Component::new(
        profile_name,
        profile.description.as_deref().unwrap_or_default(),
    )
    .group_name(Some(manifest.toolchain_group_name()))
    .is_toolchain_component(true)
    .required(true)
    .installed(installed_by_rim.toolchain_installed())
    .version(Some(tc_channel))];

    for component in manifest.optional_toolchain_components() {
        components.push(
            Component::new(
                component,
                manifest.get_tool_description(component).unwrap_or_default(),
            )
            .group_name(Some(manifest.toolchain_group_name()))
            .optional(true)
            .is_toolchain_component(true)
            .installed(installed_by_rim.all.contains(component))
            .version(Some(tc_channel)),
        );
    }

    if let Some(tools) = manifest.current_target_tools() {
        // components that are already installed in user's machine, such as vscode, or mingw.
        let installed_by_user = manifest.already_installed_tools();

        for (tool_name, tool_info) in tools {
            let installed =
                installed_by_rim.all.contains(tool_name) || installed_by_user.contains(&tool_name);
            components.push(
                Component::new(
                    tool_name,
                    manifest.get_tool_description(tool_name).unwrap_or_default(),
                )
                .group_name(manifest.group_name(tool_name))
                .tool_installer(tool_info)
                .required(tool_info.is_required())
                .optional(tool_info.is_optional())
                .installed(installed)
                .version(tool_info.version()),
            );
        }
    }

    Ok(components)
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

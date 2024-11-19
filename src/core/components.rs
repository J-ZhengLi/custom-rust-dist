use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    fingerprint::InstallationRecord,
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

macro_rules! setter {
    ($name:ident ($self_arg:ident, $t:ty)) => {
        #[allow(clippy::wrong_self_convention)]
        fn $name(mut $self_arg, val: $t) -> Self {
            $self_arg.$name = val;
            $self_arg
        }
    };
    ($name:ident ($self_arg:ident, $val:ident : $t:ty) { $init_val:expr }) => {
        fn $name(mut $self_arg, $val: $t) -> Self {
            $self_arg.$name = $init_val;
            $self_arg
        }
    };
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
    // Try getting these record first in order to determine `is_installed` status
    let installed = InstalledComponents::from_record(record);
    let tc_channel = installed
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
    .installed(installed.toolchain_installed())
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
            .installed(installed.all.contains(component))
            .version(Some(tc_channel)),
        );
    }

    if let Some(tools) = manifest.current_target_tools() {
        for (tool_name, tool_info) in tools {
            components.push(
                Component::new(
                    tool_name,
                    manifest.get_tool_description(tool_name).unwrap_or_default(),
                )
                .group_name(manifest.group_name(tool_name))
                .tool_installer(tool_info)
                .required(tool_info.is_required())
                .optional(tool_info.is_optional())
                .installed(installed.all.contains(tool_name))
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

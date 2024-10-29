use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::toolset_manifest::{ToolInfo, ToolsetManifest};

static COMPONENTS_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub fn get_component_list_from_manifest(
    manifest: &ToolsetManifest,
    ignore_installed: bool,
) -> Result<Vec<Component>> {
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
    .version(Some(manifest.rust_version()))];

    for component in manifest.optional_toolchain_components() {
        components.push(
            Component::new(
                component,
                manifest.get_tool_description(component).unwrap_or_default(),
            )
            .group_name(Some(manifest.toolchain_group_name()))
            .optional(true)
            .is_toolchain_component(true)
            .version(Some(manifest.rust_version())),
        );
    }

    let already_installed_tools = if !ignore_installed {
        manifest.already_installed_tools()
    } else {
        vec![]
    };
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
                .installed(already_installed_tools.contains(&tool_name))
                .version(tool_info.version()),
            );
        }
    }

    Ok(components)
}

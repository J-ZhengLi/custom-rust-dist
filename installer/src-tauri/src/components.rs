use crate::Result;
use custom_rust_dist::manifest;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

static COMPONENTS_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Debug)]
pub struct Component {
    pub id: u32,
    pub group_name: Option<String>,
    pub name: String,
    pub desc: String,
    pub required: bool,
    pub optional: bool,
    pub tool_installer: Option<manifest::ToolInfo>,
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
        COMPONENTS_COUNTER.fetch_add(1, Ordering::SeqCst);
        Component {
            id: COMPONENTS_COUNTER.load(Ordering::Relaxed),
            group_name: None,
            name: name.into(),
            desc: desc.into(),
            required: false,
            optional: false,
            tool_installer: None,
            is_toolchain_component: false,
            installed: false,
        }
    }

    setter!(required(self, bool));
    setter!(optional(self, bool));
    setter!(installed(self, bool));
    setter!(is_toolchain_component(self, bool));
    setter!(group_name(self, group: Option<&str>) { group.map(ToOwned::to_owned) });
    setter!(tool_installer(self, installer: &manifest::ToolInfo) { Some(installer.clone()) });
}

pub fn get_component_list_from_manifest() -> Result<Vec<Component>> {
    // TODO: Download manifest form remote server for online build
    let mut manifest = manifest::baked_in_manifest()?;
    manifest.adjust_paths()?;

    let profile = manifest.toolchain_profile().cloned().unwrap_or_default();
    let profile_name = profile.verbose_name.as_deref().unwrap_or(&profile.name);
    let mut components = vec![Component::new(
        profile_name,
        profile.description.as_deref().unwrap_or_default(),
    )
    .group_name(Some(manifest.toolchain_group_name()))
    .is_toolchain_component(true)
    .required(true)];

    for component in manifest.optional_toolchain_components() {
        components.push(
            Component::new(
                component,
                manifest.get_tool_description(component).unwrap_or_default(),
            )
            .group_name(Some(manifest.toolchain_group_name()))
            .optional(true)
            .is_toolchain_component(true),
        );
    }

    let already_installed_tools = manifest.already_installed_tools();
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
                .installed(already_installed_tools.contains(&tool_name)),
            );
        }
    }

    Ok(components)
}

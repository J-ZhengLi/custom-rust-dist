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
    pub tool_installer: Option<manifest::ToolInfo>,
    pub is_toolchain_component: bool,
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
            tool_installer: None,
            is_toolchain_component: false,
        }
    }
    fn required(mut self) -> Self {
        self.required = true;
        self
    }
    fn toolchain_component(mut self) -> Self {
        self.is_toolchain_component = true;
        self
    }
    fn with_group_name(mut self, group: &str) -> Self {
        self.group_name = Some(group.into());
        self
    }
    fn with_installer(mut self, installer: &manifest::ToolInfo) -> Self {
        self.tool_installer = Some(installer.clone());
        self
    }
}

pub fn get_component_list_from_manifest() -> Result<Vec<Component>> {
    let mut components = vec![
        Component::new("Rust", "Basic set of tools to run Rust compiler")
            .with_group_name("Rust toolchain")
            .toolchain_component()
            .required(),
    ];
    let manifest = manifest::baked_in_manifest()?;

    for toolchain_components in manifest.toolchain_components() {
        components.push(
            Component::new(
                toolchain_components,
                manifest
                    .get_tool_description(toolchain_components)
                    .unwrap_or_default(),
            )
            .with_group_name("Rust toolchain")
            .toolchain_component(),
        );
    }

    for (tool_name, tool_info) in manifest.current_target_tools() {
        let comp = Component::new(
            tool_name,
            manifest.get_tool_description(tool_name).unwrap_or_default(),
        )
        .with_group_name("Third-party tools")
        .with_installer(tool_info);
        components.push(comp);
    }

    Ok(components)
}

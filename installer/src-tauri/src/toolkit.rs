use std::collections::HashSet;

use crate::Result;
use rim::{components, fingerprint::InstallationRecord, toolset_manifest::ToolsetManifest};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Toolkit {
    name: String,
    version: String,
    desc: Option<String>,
    #[serde(alias = "notes")]
    info: Option<String>,
    #[serde(rename = "manifestURL")]
    manifest_url: Option<String>,
    components: Vec<components::Component>,
}

impl Toolkit {
    /// Try getting the toolkit from installation record and the original manifest for installed toolset.
    ///
    /// We need the manifest because it contains the details of the toolkit along with
    /// what components it has.
    pub fn from_installed() -> Result<Option<Self>> {
        if !InstallationRecord::exists()? {
            // No toolkit installed, return None
            return Ok(None);
        }

        let mut tk = Self {
            name: t!("unknown_toolkit").to_string(),
            version: "N/A".to_string(),
            desc: None,
            info: None,
            manifest_url: None,
            components: vec![],
        };
        let fp = InstallationRecord::load_from_install_dir()?;
        let manifest = ToolsetManifest::load_from_install_dir()?;

        if let Some(name) = &fp.name {
            name.clone_into(&mut tk.name);
        }
        if let Some(ver) = &fp.version {
            ver.clone_into(&mut tk.version);
        }
        // TODO: We might deprecate `name` and `version` fields in `ToolsetManifest`,
        // but until so, this check is only used to ensure the components can be installed using
        // that toolset manifest.
        if matches!(&manifest.name, Some(name) if *name == tk.name)
            && matches!(&manifest.version, Some(ver) if *ver == tk.version)
        {
            let installed_comps = fp.installed_components();
            let installed_tools = fp.installed_tools();
            let installed_set: HashSet<&&str> =
                HashSet::from_iter(installed_comps.iter().chain(installed_tools.iter()));
            let mut components = components::get_component_list_from_manifest(&manifest, true)?;
            for c in &mut components {
                if installed_set.contains(&c.name.as_str()) {
                    c.installed = true;
                }
            }
            tk.components = components;
        }

        Ok(Some(tk))
    }
}

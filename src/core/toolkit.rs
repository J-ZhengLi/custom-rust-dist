use std::collections::HashSet;
use std::sync::OnceLock;

use crate::core::parser::dist_manifest::DistManifest;
use crate::core::parser::TomlParser;
use crate::{components, utils};
use crate::{fingerprint::InstallationRecord, toolset_manifest::ToolsetManifest};
use anyhow::Result;
use log::{info, warn};
use serde::Serialize;
use url::Url;

use super::parser::dist_manifest::DistPackage;

pub(crate) const DIST_MANIFEST_TOML: &str = "distribution-manifest.toml";

/// A cached installed [`Toolkit`] struct to prevent the program doing
/// excessive IO operations as in [`from_installed`](Toolkit::from_installed).
static INSTALLED_KIT: OnceLock<Toolkit> = OnceLock::new();

#[derive(Clone, Debug, Serialize)]
pub struct Toolkit {
    pub name: String,
    pub version: String,
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
        if let Some(cached) = INSTALLED_KIT.get() {
            return Ok(Some(cached.clone()));
        }

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

        // Make a clone and cache the final result
        INSTALLED_KIT
            .set(tk.clone())
            .unwrap_or_else(|_| warn!("unable to cache the installed toolkit"));

        Ok(Some(tk))
    }
}

impl From<DistPackage> for Toolkit {
    fn from(value: DistPackage) -> Self {
        Self {
            name: value.name,
            version: value.version,
            desc: value.desc,
            info: value.info,
            manifest_url: Some(value.manifest_url.to_string()),
            components: vec![],
        }
    }
}

pub fn get_available_kits_from_server() -> Result<Vec<Toolkit>> {
    let dist_server_env_ovr = std::env::var("RIM_DIST_SERVER");
    let dist_server = dist_server_env_ovr
        .as_deref()
        .unwrap_or(super::RIM_DIST_SERVER);

    // download dist manifest from server
    info!("{} {DIST_MANIFEST_TOML}", t!("fetching"));
    let dist_m_url = Url::parse(&format!("{dist_server}/dist/{DIST_MANIFEST_TOML}"))?;
    let dist_m_file = utils::make_temp_file("dist-manifest-", None)?;
    utils::DownloadOpt::<()>::new("distribution manifest")?.download_file(
        &dist_m_url,
        dist_m_file.path(),
        false,
    )?;

    // process place-holder text in debug mode
    #[cfg(debug_assertions)]
    {
        use std::fs;
        let mut dist_m_content = fs::read_to_string(dist_m_file.path())?;
        dist_m_content = dist_m_content.replace("{{SERVER}}", Url::parse(dist_server)?.as_str());
        fs::write(dist_m_file.path(), dist_m_content)?;
    }

    // load dist "pacakges" then convert them into `toolkit`s
    let packages = DistManifest::load(dist_m_file.path())?.packages;
    Ok(packages.into_iter().rev().map(Toolkit::from).collect())
}

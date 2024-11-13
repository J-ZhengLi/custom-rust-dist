use std::collections::HashSet;
use std::sync::OnceLock;

use crate::core::parser::dist_manifest::DistManifest;
use crate::core::parser::TomlParser;
use crate::{components, utils};
use crate::{fingerprint::InstallationRecord, toolset_manifest::ToolsetManifest};
use anyhow::Result;
use log::info;
use semver::Version;
use serde::Serialize;
use url::Url;

use super::parser::dist_manifest::DistPackage;

pub(crate) const DIST_MANIFEST_TOML: &str = "distribution-manifest.toml";

/// A cached installed [`Toolkit`] struct to prevent the program doing
/// excessive IO operations as in [`installed`](Toolkit::installed).
static INSTALLED_KIT: OnceLock<Toolkit> = OnceLock::new();
/// Cache the list of toolkit provided by the server, this will save the number of times
/// that we need to make server request, in the exchange of memory usage.
static ALL_TOOLKITS: OnceLock<Vec<Toolkit>> = OnceLock::new();

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Toolkit {
    pub name: String,
    pub version: String,
    desc: Option<String>,
    #[serde(alias = "notes")]
    info: Option<String>,
    #[serde(rename = "manifestURL")]
    pub manifest_url: Option<String>,
    pub components: Vec<components::Component>,
}

impl PartialEq for Toolkit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.version == other.version
            && self.manifest_url == other.manifest_url
    }
}

impl Toolkit {
    /// Try getting the toolkit from installation record and the original manifest for installed toolset.
    ///
    /// We need the manifest because it contains the details of the toolkit along with
    /// what components it has.
    pub fn installed() -> Result<Option<&'static Self>> {
        if let Some(cached) = INSTALLED_KIT.get() {
            return Ok(Some(cached));
        }

        if !InstallationRecord::exists()? {
            // No toolkit installed, return None
            return Ok(None);
        }

        // Right now we already know user has installed a toolkit,
        // just don't know the details yet, so let's create a "blank" installed toolkit
        // then try to collect the missing details later.
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

        let installed_comps = fp.installed_components();
        let installed_tools = fp.installed_tools();
        let installed_set: HashSet<&&str> =
            HashSet::from_iter(installed_comps.iter().chain(installed_tools.iter()));
        let components = components::get_component_list_from_manifest(&manifest, true)?;
        tk.components = components;

        for c in &mut tk.components {
            if installed_set.contains(&c.name.as_str()) {
                c.installed = true;
            }
        }

        // Make a clone and cache the final result
        let cached = INSTALLED_KIT.get_or_init(|| tk);
        Ok(Some(cached))
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

/// Download the dist manifest from server to get the list of all provided toolkits.
///
/// Note the retrieved list will be reversed so that the newest toolkit will always be on top.
fn toolkits_from_server() -> Result<&'static [Toolkit]> {
    if let Some(cached) = ALL_TOOLKITS.get() {
        return Ok(cached);
    }

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
    let cached =
        ALL_TOOLKITS.get_or_init(|| packages.into_iter().map(Toolkit::from).rev().collect());
    Ok(cached)
}

/// Return a list of all toolkits that are not currently installed.
pub fn installable_toolkits() -> Result<Vec<&'static Toolkit>> {
    let all_toolkits = toolkits_from_server()?;
    let installable = if let Some(installed) = Toolkit::installed()? {
        all_toolkits.iter().filter(|tk| *tk != installed).collect()
    } else {
        all_toolkits.iter().collect()
    };
    Ok(installable)
}

/// Return the latest available toolkit if it's not already installed.
pub fn latest_installable_toolkit() -> Result<Option<&'static Toolkit>> {
    let all_toolkits = toolkits_from_server()?;
    if let Some(installed) = Toolkit::installed()? {
        let Some(maybe_latest) = all_toolkits
            .iter()
            // make sure they are the same **product**
            .find(|tk| tk.name == installed.name)
        else {
            return Ok(None);
        };
        // For some reason, the version might contains prefixes such as "stable 1.80.1",
        // therefore we need to trim them so that `semver` can be used to parse the actual
        // version string.
        // NB (J-ZhengLi): We might need another version field... one for display,
        // one for the actual version.
        let cur_ver = installed
            .version
            .trim_start_matches(|c| !char::is_ascii_digit(&c));
        let target_ver = maybe_latest
            .version
            .trim_start_matches(|c| !char::is_ascii_digit(&c));
        let cur_version: Version = cur_ver.parse()?;
        let target_version: Version = target_ver.parse()?;

        if target_version > cur_version {
            Ok(Some(maybe_latest))
        } else {
            Ok(None)
        }
    } else {
        Ok(all_toolkits.first())
    }
}

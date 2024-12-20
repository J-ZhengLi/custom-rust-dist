use std::sync::{Mutex, OnceLock};

use crate::core::parser::dist_manifest::DistManifest;
use crate::core::parser::TomlParser;
use crate::fingerprint::InstallationRecord;
use crate::{components, utils};
use anyhow::Result;
use log::{debug, info};
use semver::Version;
use serde::Serialize;
use url::Url;

use super::parser::dist_manifest::DistPackage;

/// A cached installed [`Toolkit`] struct to prevent the program doing
/// excessive IO operations as in [`installed`](Toolkit::installed).
static INSTALLED_KIT: OnceLock<Mutex<Toolkit>> = OnceLock::new();
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
        self.name == other.name && self.version == other.version
    }
}

impl Toolkit {
    /// Try getting the toolkit from installation record and the original manifest for installed toolset.
    ///
    /// The installed kit will be cached to reduce the number of IO operations.
    /// However, if `reload_cache` is `true`, the cache will be ignored, and will be
    /// updated once installed kit is being reloaded.
    pub fn installed(reload_cache: bool) -> Result<Option<&'static Mutex<Self>>> {
        if !reload_cache {
            return Ok(INSTALLED_KIT.get());
        }

        if !InstallationRecord::exists()? {
            // No toolkit installed, return None
            return Ok(None);
        }

        let fp = InstallationRecord::load_from_install_dir()?;
        let components = components::all_components_from_installation(&fp)?;

        let tk = Self {
            name: fp
                .name
                .clone()
                .unwrap_or_else(|| t!("unknown_toolkit").to_string()),
            version: fp.version.as_deref().unwrap_or("N/A").to_string(),
            desc: None,
            info: None,
            manifest_url: None,
            components,
        };

        if let Some(existing) = INSTALLED_KIT.get() {
            // If we already have a cache, update the inner value of it.
            let mut guard = existing.lock().unwrap();
            *guard = tk;
            drop(guard);
            Ok(Some(existing))
        } else {
            // If we are creating a fresh cache, just return the inner mutex guard.
            let mutex = INSTALLED_KIT.get_or_init(|| Mutex::new(tk));
            Ok(Some(mutex))
        }
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
///
/// The collection will always be cached to reduce the number of server requests.
fn toolkits_from_server(insecure: bool) -> Result<&'static [Toolkit]> {
    if let Some(cached) = ALL_TOOLKITS.get() {
        return Ok(cached);
    }

    let dist_server_env_ovr = std::env::var("RIM_DIST_SERVER");
    let dist_server = dist_server_env_ovr
        .as_deref()
        .unwrap_or(super::RIM_DIST_SERVER);

    // download dist manifest from server
    let dist_m_filename = DistManifest::FILENAME;
    info!("{} {dist_m_filename}", t!("fetching"));
    let dist_m_url = Url::parse(&format!("{dist_server}/dist/{dist_m_filename}"))?;
    let dist_m_file = utils::make_temp_file("dist-manifest-", None)?;
    utils::DownloadOpt::new("distribution manifest")
        .insecure(insecure)
        .download_file(&dist_m_url, dist_m_file.path(), false)?;
    debug!("distribution manifest file successfully downloaded!");

    // load dist "pacakges" then convert them into `toolkit`s
    let packages = DistManifest::load(dist_m_file.path())?.packages;
    let cached =
        ALL_TOOLKITS.get_or_init(|| packages.into_iter().map(Toolkit::from).rev().collect());
    debug!(
        "detected {} available toolkits by accessing server",
        cached.len()
    );
    Ok(cached)
}

/// Return a list of all toolkits that are not currently installed.
pub fn installable_toolkits(reload_cache: bool, insecure: bool) -> Result<Vec<&'static Toolkit>> {
    let all_toolkits = toolkits_from_server(insecure)?;
    let installable = if let Some(installed) = Toolkit::installed(reload_cache)? {
        all_toolkits
            .iter()
            .filter(|tk| *tk != &*installed.lock().unwrap())
            .collect()
    } else {
        all_toolkits.iter().collect()
    };
    Ok(installable)
}

/// Return the latest available toolkit if it's not already installed.
pub fn latest_installable_toolkit(
    reload_cache: bool,
    insecure: bool,
) -> Result<Option<&'static Toolkit>> {
    info!("{}", t!("checking_toolkit_updates"));

    let all_toolkits = toolkits_from_server(insecure)?;
    if let Some(installed) = Toolkit::installed(reload_cache)? {
        let installed = &*installed.lock().unwrap();
        let Some(maybe_latest) = all_toolkits
            .iter()
            // make sure they are the same **product**
            .find(|tk| tk.name == installed.name)
        else {
            info!("{}", t!("no_available_updates"));
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
            info!(
                "{}",
                t!(
                    "latest_toolkit_installed",
                    name = installed.name,
                    version = cur_version
                )
            );
            Ok(None)
        }
    } else {
        Ok(all_toolkits.first())
    }
}

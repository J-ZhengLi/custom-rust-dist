use std::env;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use log::{debug, info, warn};
use semver::Version;
use url::Url;

use super::directories::RimDir;
use super::parser::release_info::ReleaseInfo;
use super::parser::TomlParser;
use crate::utils;

/// Caching the latest manager release info, reduce the number of time accessing the server.
static LATEST_RELEASE: OnceLock<ReleaseInfo> = OnceLock::new();

#[derive(Default)]
pub struct UpdateOpt;

impl RimDir for UpdateOpt {
    fn install_dir(&self) -> &Path {
        crate::get_installed_dir()
    }
}

impl UpdateOpt {
    pub fn new() -> Self {
        Self
    }

    /// Calls a function to update toolkit.
    ///
    /// This is just a callback wrapper (for now), you still have to provide a function to do the
    /// internal work.
    // TODO: find a way to generalize this, so we can write a shared logic here instead of
    // creating update functions for both CLI and GUI.
    pub fn update_toolkit<F>(&self, callback: F) -> Result<()>
    where
        F: FnOnce(&Path) -> Result<()>,
    {
        let dir = self.install_dir();
        callback(dir).context("unable to update toolkit")
    }

    /// Update self when applicable.
    ///
    /// If the program is succesfully updated, this will return `Ok(true)`,
    /// which indicates the program should be restarted.
    pub fn self_update(&self) -> Result<bool> {
        if !check_self_update().update_needed() {
            info!(
                "{}",
                t!(
                    "latest_manager_installed",
                    version = env!("CARGO_PKG_VERSION")
                )
            );
            return Ok(false);
        }

        #[cfg(not(feature = "gui"))]
        let cli = "-cli";
        #[cfg(feature = "gui")]
        let cli = "";

        let src_name = utils::exe!(format!("{}-manager{cli}", t!("vendor_en")));
        let latest_version = &latest_manager_release()?.version;
        let download_url = parse_download_url(&format!(
            "manager/archive/{latest_version}/{}/{src_name}",
            env!("TARGET"),
        ))?;

        info!(
            "{}",
            t!("downloading_latest_manager", version = latest_version)
        );
        // creates another directory under `temp` folder, it will be used to hold a
        // newer version of the manager binary, which will then replacing the current running one.
        let temp_root = tempfile::Builder::new()
            .prefix("manager-download_")
            .tempdir_in(self.temp_dir())?;
        // dest file don't need the `-cli` suffix to confuse users
        let dest_name = utils::exe!(format!("{}-manager", t!("vendor_en")));
        let newer_manager = temp_root.path().join(dest_name);
        utils::download("latest manager", &download_url, &newer_manager, None)?;

        // replace the current executable
        // TODO: restart GUI when available.
        self_replace::self_replace(newer_manager)?;

        info!("{}", t!("self_update_complete"));
        Ok(true)
    }
}

/// Try to get the manager's latest release infomation.
///
/// This will try to access the internet upon first call in order to
/// read the `release.toml` file from the server, and the result will be "cached" after.
fn latest_manager_release() -> Result<&'static ReleaseInfo> {
    if let Some(release_info) = LATEST_RELEASE.get() {
        return Ok(release_info);
    }

    let download_url = parse_download_url(&format!("manager/{}", ReleaseInfo::FILENAME))?;
    let raw = utils::DownloadOpt::<()>::new("manager release info")?.read(&download_url)?;
    let release_info = ReleaseInfo::from_str(&raw)?;

    Ok(LATEST_RELEASE.get_or_init(|| release_info))
}

pub enum SelfUpdateKind<'a> {
    Newer(&'a Version),
    Uncertain,
    UnNeeded,
}

impl SelfUpdateKind<'_> {
    pub fn update_needed(&self) -> bool {
        matches!(self, Self::Newer(_))
    }
}

impl<'a> SelfUpdateKind<'a> {
    pub fn newer_version(&self) -> Option<&Version> {
        match self {
            Self::Newer(v) => Some(*v),
            _ => None,
        }
    }
}

/// Returns `true` if current manager version is lower than its latest version.
///
/// If the version info could not be fetched, this will return `false` otherwise.
pub fn check_self_update() -> SelfUpdateKind<'static> {
    info!("{}", t!("checking_manager_updates"));

    let latest_version = match latest_manager_release() {
        Ok(release) => &release.version,
        Err(e) => {
            warn!("{}: {e}", t!("fetch_latest_manager_version_failed"));
            return SelfUpdateKind::Uncertain;
        }
    };

    // safe to unwrap, otherwise cargo would fails the build
    let cur_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    if &cur_version < latest_version {
        SelfUpdateKind::Newer(latest_version)
    } else {
        SelfUpdateKind::UnNeeded
    }
}

fn parse_download_url(source_path: &str) -> Result<Url> {
    let base_obs_server: Url = env::var("RIM_DIST_SERVER")
        .as_deref()
        .unwrap_or(super::RIM_DIST_SERVER)
        .parse()?;

    debug!("parsing download url for '{source_path}' from server '{base_obs_server}'");
    utils::url_join(&base_obs_server, source_path)
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_comparison() {
        macro_rules! compare {
            ($lhs:literal $op:tt $rhs:literal) => {
                assert!(
                    semver::Version::parse($lhs).unwrap() $op semver::Version::parse($rhs).unwrap()
                );
            };
        }

        compare!("0.1.0" < "0.2.0");
        compare!("0.1.0" < "0.2.0-alpha");
        compare!("0.1.0" > "0.1.0-alpha");
        compare!("0.1.0-alpha" < "0.1.0-beta");
        compare!("0.1.0-alpha" < "0.1.0-alpha.1");
        compare!("0.1.0-alpha.1" < "0.1.0-alpha.2");
        compare!("1.0.0" == "1.0.0");
    }
}

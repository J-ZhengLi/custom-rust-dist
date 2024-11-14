use std::env::{self, current_exe};
use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use log::{debug, info, warn};
use semver::Version;
use url::Url;

use super::parser::release_info::ReleaseInfo;
use super::parser::TomlParser;
use crate::utils;

/// Caching the latest manager release info, reduce the number of time accessing the server.
static LATEST_RELEASE: OnceLock<ReleaseInfo> = OnceLock::new();

/// Calls a function to update toolkit.
///
/// This is just a callback wrapper (for now), you still have to provide a function to do the
/// internal work.
// TODO: find a way to generalize this, so we can write a shared logic here instead of
// creating update functions for both CLI and GUI.
pub fn update_toolkit<F>(callback: F) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    let dir = crate::get_installed_dir();
    callback(dir).context("unable to update toolkit")
}

/// Returns `true` if current manager version is lower than its latest version.
///
/// Otherwise, or when the versions could not be determined, this will return `false`.
pub fn check_self_update() -> bool {
    info!("{}", t!("checking_manager_updates"));

    let latest_version = match latest_release() {
        Ok(release) => &release.version,
        Err(e) => {
            warn!("{}: {e}", t!("fetch_latest_manager_version_failed"));
            return false;
        }
    };

    // safe to unwrap, otherwise cargo would fails the build
    let cur_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    &cur_version < latest_version
}

pub fn do_self_update() -> Result<()> {
    let filename = format!("{}-manager{}", t!("vendor_en"), env::consts::EXE_SUFFIX);
    let latest_version = &latest_release()?.version;
    let download_url = parse_download_url(&format!("manager/archive/{latest_version}/{filename}"))?;

    let dest = current_exe()?;
    utils::download(filename, &download_url, &dest, None)
}

fn parse_download_url(source_path: &str) -> Result<Url> {
    let mut base_obs_server =
        env::var("RIM_DIST_SERVER").unwrap_or_else(|_| super::RIM_DIST_SERVER.to_string());

    // without the trailing slash, `.join` will replace the last component instead of append it.
    if !base_obs_server.ends_with('/') {
        base_obs_server.push('/');
    }
    debug!("parsing download url for '{source_path}' from server '{base_obs_server}'");

    Ok(Url::parse(&base_obs_server)?.join(source_path)?)
}

// Try to get the latest manager version
fn latest_release() -> Result<&'static ReleaseInfo> {
    if let Some(release_info) = LATEST_RELEASE.get() {
        return Ok(release_info);
    }

    let download_url = parse_download_url(&format!("manager/{}", ReleaseInfo::FILENAME))?;
    let raw = utils::DownloadOpt::<()>::new("manager release info")?.read(&download_url)?;
    let release_info = ReleaseInfo::from_str(&raw)?;

    Ok(LATEST_RELEASE.get_or_init(|| release_info))
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

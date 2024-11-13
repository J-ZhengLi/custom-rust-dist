use std::env::{self, current_exe};
use std::path::Path;

use anyhow::{Context, Result};
use semver::Version;
use url::Url;

use crate::utils;

/// Calls a function to update toolkit.
///
/// This is just a callback wrapper, you still have to provide a function to do the
/// internal work.
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
    macro_rules! warn_on_err {
        (let $ident:ident = $operation:expr, $($msg:tt)*) => {
            let Ok($ident) = $operation else {
                log::warn!($($msg)*);
                return false;
            };
        };
    }

    let cur_ver = env!("CARGO_PKG_VERSION");
    warn_on_err!(
        let latest_ver = latest_version(),
        "{}", t!("fetch_latest_manager_version_failed")
    );
    warn_on_err!(
        let cur_version = Version::parse(cur_ver),
        "{}", t!("parse_version_failed", kind = t!("current"), version = cur_ver)
    );
    warn_on_err!(
        let latest_version = Version::parse(&latest_ver),
        "{}", t!("parse_version_failed", kind = t!("target"), version = latest_ver)
    );

    cur_version < latest_version
}

pub fn do_self_update() -> Result<()> {
    let filename = format!("{}-manager{}", t!("vendor_en"), env::consts::EXE_SUFFIX);
    let latest_version = latest_version()?;
    let download_url = parse_download_url(&format!("/manager/dist/{latest_version}/{filename}"))?;

    let dest = current_exe()?;
    utils::download(filename, &download_url, &dest, None)
}

fn parse_download_url(source_path: &str) -> Result<Url> {
    let base_obs_server =
        env::var("RIM_DIST_SERVER").unwrap_or_else(|_| super::RIM_DIST_SERVER.to_string());

    Ok(Url::parse(&base_obs_server)?.join(source_path)?)
}

// Try to get the latest manager version
fn latest_version() -> Result<String> {
    let download_url = parse_download_url("/manager/version")?;
    utils::DownloadOpt::<()>::new("manager version file")?.read(&download_url)
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

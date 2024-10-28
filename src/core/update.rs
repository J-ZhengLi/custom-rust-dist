use std::env;

use anyhow::{Context, Result};
use log::info;
use reqwest::blocking::Client;
use url::Url;

use crate::{get_installed_dir, utils};

pub struct UpdateConfiguration;

pub(crate) const MANAGER_SOURCE_PATH: &str = "/manager/version";

impl UpdateConfiguration {
    pub fn update(&self, self_udpate: bool) -> Result<()> {
        if self_udpate {
            self.upgrade_manager()?;
        } else {
            self.update_toolsets()?;
        }
        Ok(())
    }

    pub fn check_upgrade(&self) -> Result<bool> {
        let local_version = local_version();
        let latest_version = latest_version(MANAGER_SOURCE_PATH)?;
        Ok(local_version != latest_version)
    }

    fn upgrade_manager(&self) -> Result<()> {
        let update = self.check_upgrade()?;
        // By default, if the version is different from the local version, an update is performed.
        if update {
            let latest_version = latest_version(MANAGER_SOURCE_PATH)?;
            let download_url = parse_download_url(&format!(
                "/manager/dist/{}/{}",
                latest_version,
                full_manager_name()
            ))?;

            let dest = get_installed_dir().join(full_manager_name());
            utils::download(full_manager_name().as_str(), &download_url, &dest, None)?;
        } else {
            info!("Already latest version.");
        }

        Ok(())
    }

    fn update_toolsets(&self) -> Result<()> {
        // When updating the tool, it will detect whether
        // there is a new version of the management tool
        let config: UpdateConfiguration = UpdateConfiguration;
        let upgradeable = config.check_upgrade().unwrap_or(false);
        if upgradeable {
            info!(
                "A new manager version has been detected. You can update it via using `--self-update`"
            )
        };
        // TODO: update toolchain via toolsets manifest.
        Ok(())
    }
}

fn full_manager_name() -> String {
    let full_manager_name = format!("{}-manager{}", t!("vendor_en"), env::consts::EXE_SUFFIX);
    full_manager_name
}

fn parse_download_url(source_path: &str) -> Result<Url> {
    let base_obs_server =
        env::var("OBS_DIST_SERVER").unwrap_or_else(|_| super::RIM_DIST_SERVER.to_string());

    Ok(Url::parse(&base_obs_server)?.join(source_path)?)
}

// Try to get manager version via execute command with `--version`.
fn local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Try to get the latest manager version
fn latest_version(source_path: &str) -> Result<String> {
    let download_url = parse_download_url(source_path)?;
    // Download the latest manager version file
    let client = Client::new();
    let resp = client.get(download_url).send().with_context(|| {
        format!(
            "failed to get latest manager version file: \n '{}'",
            source_path
        )
    })?;

    if resp.status().is_success() {
        let content = resp
            .text()
            .with_context(|| "Failed to read response text")?;
        Ok(content)
    } else {
        Err(anyhow::anyhow!(
            "Failed to download object: {}",
            resp.status()
        ))
    }
}

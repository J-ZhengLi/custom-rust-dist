use std::{
    env,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use url::Url;

use crate::{get_installed_dir, utils};

pub struct UpdateConfiguration;

pub(crate) const BASE_OBS_URL: &str = "https://rust-mirror.obs.cn-north-4.myhuaweicloud.com";
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
        let local_version = local_version(full_manager_name().as_str())?;
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
            println!("Already latest version.");
        }

        Ok(())
    }

    fn update_toolsets(&self) -> Result<()> {
        // When updating the tool, it will detect whether
        // there is a new version of the management tool
        utils::check_manager_upgrade();
        // TODO: update toolchain via toolsets manifest.
        Ok(())
    }
}

fn full_manager_name() -> String {
    let full_manager_name = format!("xuanwu-manager{}", env::consts::EXE_SUFFIX);
    full_manager_name
}

fn parse_download_url(source_path: &str) -> Result<Url> {
    let base_obs_server = env::var("OBS_DIST_SERVER").unwrap_or_else(|_| BASE_OBS_URL.to_string());

    Ok(Url::parse(&base_obs_server)?.join(source_path)?)
}

// Try to get manager version via execute command with `--version`.
fn local_version(command: &str) -> Result<String> {
    // execute command
    let output = Command::new(command) // 使用 manager 的路径
        .arg("--version")
        .stdout(Stdio::piped()) // 将标准输出重定向到管道
        .stderr(Stdio::piped()) // 将标准错误重定向到管道
        .output()
        .with_context(|| format!("Failed to execute {}", command))?; // 添加上下文信息

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .with_context(|| "Failed to convert output to UTF-8")?; // 添加上下文信息
        let content = stdout.trim().to_string(); // 返回输出并去除多余空白
        let version = content
            .split(' ')
            .nth(1)
            .context("Failed to parse version")?;
        Ok(version.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Command failed with error: {}", stderr)) // 返回错误信息
    }
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

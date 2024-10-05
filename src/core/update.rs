use std::process::{Command, Stdio};
use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use url::Url;

use crate::get_installed_dir;
use crate::utils::{self, OBSOpt};

pub(crate) const MANAGER_VERSION_URL: &str = "https://rustup.obs.cn-south-1.myhuaweicloud.com";

pub struct UpdateConfiguration {
    pub(crate) install_dir: PathBuf,
}

impl UpdateConfiguration {
    pub(crate) fn init() -> Result<Self> {
        let install_dir = get_installed_dir();
        Ok(Self {
            install_dir: install_dir.to_path_buf(),
        })
    }

    pub fn update(&self, no_self_update: bool) -> Result<()> {
        // replace the manager
        println!("{:?}", self.install_dir);

        if !no_self_update {
            self.update_manager()?;
        }

        self.update_toolchain()?;

        Ok(())
    }

    fn update_manager(&self) -> Result<()> {
        let manager_version = local_manager_version()?;
        let latest_manager_version = latest_manager_version()?;
        println!("{:?}", manager_version);
        if manager_version != latest_manager_version {
            println!("updating manager...");
            let obs_dist_server =
                env::var("OBS_DIST_SERVER").unwrap_or_else(|_| MANAGER_VERSION_URL.to_string());
            let manager = format!("xuanwu-manager{}", env::consts::EXE_SUFFIX);
            let version = latest_manager_version
                .split(' ')
                .nth(1)
                .context("Failed to parse manager version.")?;
            let latest_manager_url =
                Url::parse(&obs_dist_server)?.join(&format!("/dist/{}/{}", version, manager))?;
            let dest = self.install_dir.join(&manager);
            utils::download(manager, &latest_manager_url, &dest, None)?;
        } else {
            println!("Already latest version.");
        }

        Ok(())
    }

    fn update_toolchain(&self) -> Result<()> {
        println!("updating toolchain...");

        Ok(())
    }
}

// Try TO get manager version.
fn local_manager_version() -> Result<String> {
    // Get version information through the manager which is using
    let manager = format!("xuanwu-manager{}", env::consts::EXE_SUFFIX);

    // 执行命令
    let output = Command::new(&manager) // 使用 manager 的路径
        .arg("--version")
        .stdout(Stdio::piped()) // 将标准输出重定向到管道
        .stderr(Stdio::piped()) // 将标准错误重定向到管道
        .output()
        .with_context(|| format!("Failed to execute {}", manager))?; // 添加上下文信息

    // 检查命令是否成功执行
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .with_context(|| "Failed to convert output to UTF-8")?; // 添加上下文信息
        Ok(stdout.trim().to_string()) // 返回输出并去除多余空白
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Command failed with error: {}", stderr)) // 返回错误信息
    }
}

// Try to get the latest managed version
fn latest_manager_version() -> Result<String> {
    let obs_dist_server =
        env::var("OBS_DIST_SERVER").unwrap_or_else(|_| MANAGER_VERSION_URL.to_string());

    let download_url = Url::parse(&obs_dist_server).with_context(|| "Failed to parse URL")?;

    let resource_path = String::from("/manager/version");

    let obs_operator = OBSOpt::new(download_url, resource_path)?;
    obs_operator.read()
}

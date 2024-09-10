use super::{
    parser::{
        cargo_config::CargoConfig,
        fingerprint::FingerPrint,
        manifest::{ToolInfo, ToolsetManifest},
        TomlParser,
    },
    rustup::Rustup,
    tools::Tool,
    CARGO_HOME, RUSTUP_DIST_SERVER, RUSTUP_HOME, RUSTUP_UPDATE_ROOT,
};
use crate::{
    core::os::add_to_path,
    manifest::{Proxy, ToolMap},
    utils::{self, Extractable, MultiThreadProgress},
};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use url::Url;

macro_rules! declare_unfallible_url {
    ($($name:ident($global:ident) -> $val:literal);+) => {
        $(
            static $global: std::sync::OnceLock<url::Url> = std::sync::OnceLock::new();
            pub(crate) fn $name() -> &'static url::Url {
                $global.get_or_init(|| {
                    url::Url::parse($val).expect(
                        &format!("Internal Error: static variable '{}' cannot be parse to URL", $val)
                    )
                })
            }
        )*
    };
}

macro_rules! declare_install_paths {
    ($($path_ident:ident),+) => {
        $(
            static $path_ident: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
        )*
    };
}

/// Get the once-locked path under install_dir, and create that directory if it does not exists.
macro_rules! get_path_and_create {
    ($path_ident:ident, $init:expr) => {{
        let __path__ = $path_ident.get_or_init(|| $init);
        $crate::utils::ensure_dir(__path__)
            .expect("unable to create one of the directory under installation folder");
        __path__
    }};
}

declare_unfallible_url!(
    default_rustup_dist_server(DEFAULT_RUSTUP_DIST_SERVER) -> "https://mirrors.tuna.tsinghua.edu.cn/rustup";
    default_rustup_update_root(DEFAULT_RUSTUP_UPDATE_ROOT) -> "https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup"
);

declare_install_paths!(
    CARGO_HOME_DIR,
    CARGO_BIN_DIR,
    RUSTUP_HOME_DIR,
    TEMP_DIR,
    TOOLS_DIR
);

/// Contains definition of installation steps, including pre-install configs.
///
/// Make sure to always call `init()` as it creates essential folders to
/// hold the installation files.
pub trait EnvConfig {
    /// Configure environment variables.
    ///
    /// This will set persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`, etc.
    fn config_env_vars(&self, manifest: &ToolsetManifest) -> Result<()>;
}

// If you change any public fields in this struct,
// make sure to change `installer/src/utils/types/InstallConfiguration.ts` as well.
#[derive(Debug, Deserialize, Serialize)]
pub struct InstallConfiguration {
    pub cargo_registry: Option<(String, String)>,
    /// Path to install everything.
    ///
    /// Note that this folder will includes `.cargo` and `.rustup` folders as well.
    /// And the default location will be `$HOME` directory (`%USERPROFILE%` on windows).
    /// So, even if the user didn't specify any install path, a pair of env vars will still
    /// be written (CARGO_HOME and RUSTUP_HOME), as they will be located in a sub folder of `$HOME`,
    /// which is [`installer_home`](utils::installer_home).
    pub install_dir: PathBuf,
    pub rustup_dist_server: Url,
    pub rustup_update_root: Url,
    /// Indicates whether `cargo` was already installed, useful when installing third-party tools.
    cargo_is_installed: bool,
}

impl Default for InstallConfiguration {
    fn default() -> Self {
        Self {
            install_dir: default_install_dir(),
            cargo_registry: None,
            rustup_dist_server: default_rustup_dist_server().clone(),
            rustup_update_root: default_rustup_update_root().clone(),
            cargo_is_installed: false,
        }
    }
}

impl InstallConfiguration {
    pub fn init(install_dir: &Path, dry_run: bool) -> Result<Self> {
        if install_dir.parent().is_none() {
            bail!("unable to install in root directory");
        }
        let this = Self {
            install_dir: install_dir.to_path_buf(),
            ..Default::default()
        };

        if !dry_run {
            // Create a new folder to hold installation
            let folder = &this.install_dir;
            utils::ensure_dir(folder)?;

            // TODO: remove this condition check after the uninstallation implementation is finished.
            if env!("PROFILE") == "debug" {
                // Create a copy of this binary to CARGO_HOME/bin
                let self_exe = std::env::current_exe()?;
                // promote this installer to manager
                let manager_name = format!("{}-manager{}", t!("vendor_en"), utils::EXE_EXT);

                let manager_exe = this.cargo_bin().join(manager_name);
                utils::copy_as(self_exe, &manager_exe)?;

                #[cfg(windows)]
                // Create registry entry to add this program into "installed programs".
                super::os::windows::do_add_to_programs(&manager_exe)?;
            }
        }

        Ok(this)
    }

    pub fn cargo_registry(mut self, registry: Option<(String, String)>) -> Self {
        self.cargo_registry = registry;
        self
    }

    pub fn rustup_dist_server(mut self, url: Url) -> Self {
        self.rustup_dist_server = url;
        self
    }

    pub fn rustup_update_root(mut self, url: Url) -> Self {
        self.rustup_update_root = url;
        self
    }

    pub(crate) fn cargo_home(&self) -> &Path {
        get_path_and_create!(CARGO_HOME_DIR, self.install_dir.join(".cargo"))
    }

    pub(crate) fn cargo_bin(&self) -> &Path {
        get_path_and_create!(CARGO_BIN_DIR, self.cargo_home().join("bin"))
    }

    pub(crate) fn rustup_home(&self) -> &Path {
        get_path_and_create!(RUSTUP_HOME_DIR, self.install_dir.join(".rustup"))
    }

    pub(crate) fn temp_root(&self) -> &Path {
        get_path_and_create!(TEMP_DIR, self.install_dir.join("temp"))
    }

    pub(crate) fn tools_dir(&self) -> &Path {
        get_path_and_create!(TOOLS_DIR, self.install_dir.join("tools"))
    }

    pub(crate) fn env_vars(
        &self,
        manifest: &ToolsetManifest,
    ) -> Result<Vec<(&'static str, String)>> {
        let cargo_home = self
            .cargo_home()
            .to_str()
            .map(ToOwned::to_owned)
            .context("`install-dir` cannot contains invalid unicodes")?;
        // This `unwrap` is safe here because we've already make sure the `install_dir`'s path can be
        // converted to string with the `cargo_home` variable.
        let rustup_home = self.rustup_home().to_str().unwrap().to_string();

        let mut env_vars: Vec<(&str, String)> = vec![
            (RUSTUP_DIST_SERVER, self.rustup_dist_server.to_string()),
            (RUSTUP_UPDATE_ROOT, self.rustup_update_root.to_string()),
            (CARGO_HOME, cargo_home),
            (RUSTUP_HOME, rustup_home),
        ];

        // Add proxy settings if has
        if let Some(proxy) = &manifest.proxy {
            if let Some(url) = &proxy.http {
                env_vars.push(("http_proxy", url.to_string()));
            }
            if let Some(url) = &proxy.https {
                env_vars.push(("https_proxy", url.to_string()));
            }
            if let Some(s) = &proxy.no_proxy {
                env_vars.push(("no_proxy", s.to_string()));
            }
        }

        Ok(env_vars)
    }

    pub fn install_tools_with_progress(
        &self,
        manifest: &ToolsetManifest,
        tools: &ToolMap,
        mt_prog: &mut MultiThreadProgress,
    ) -> Result<()> {
        // Ignore tools that need to be installed using `cargo install`
        let to_install = tools
            .into_iter()
            .filter(|(_, t)| !t.is_cargo_tool())
            .collect::<Vec<_>>();
        let sub_progress_delta = if to_install.is_empty() {
            return mt_prog.send_progress();
        } else {
            mt_prog.val / to_install.len()
        };

        for (name, tool) in to_install {
            send_and_print(t!("installing_tool_info", name = name), mt_prog)?;
            install_tool(self, name, tool, manifest.proxy.as_ref())?;

            mt_prog.send_any_progress(sub_progress_delta)?;
        }

        Ok(())
    }

    pub fn install_rust_with_progress(
        &mut self,
        manifest: &ToolsetManifest,
        optional_components: &[String],
        mt_prog: &mut MultiThreadProgress,
    ) -> Result<()> {
        send_and_print(t!("installing_toolchain_info"), mt_prog)?;

        Rustup::init().download_toolchain(self, manifest, optional_components)?;
        add_to_path(self.cargo_bin())?;
        self.cargo_is_installed = true;

        // Add the rust info to the fingerprint.
        add_rust_fingerprint(
            &self.install_dir,
            manifest.rust_version(),
            optional_components,
        )?;

        mt_prog.send_progress()
    }

    pub fn cargo_install_with_progress(
        &self,
        tools: &ToolMap,
        mt_prog: &mut MultiThreadProgress,
    ) -> Result<()> {
        let to_install = tools
            .into_iter()
            .filter(|(_, t)| t.is_cargo_tool())
            .collect::<Vec<_>>();
        let sub_progress_delta = if to_install.is_empty() {
            return mt_prog.send_progress();
        } else {
            mt_prog.val / to_install.len()
        };

        for (name, tool) in to_install {
            send_and_print(t!("installing_via_cargo_info", name = name), mt_prog)?;
            install_tool(self, name, tool, None)?;

            mt_prog.send_any_progress(sub_progress_delta)?;
        }

        Ok(())
    }

    /// Configuration options for `cargo`.
    ///
    /// This will write a `config.toml` file to `CARGO_HOME`.
    pub fn config_cargo(&self) -> Result<()> {
        let mut config = CargoConfig::new();
        if let Some((name, url)) = &self.cargo_registry {
            config.add_source(name, url, true);
        }

        let config_toml = config.to_toml()?;
        if !config_toml.trim().is_empty() {
            let config_path = self.cargo_home().join("config.toml");
            utils::write_file(config_path, &config_toml, false)?;
        }

        Ok(())
    }

    /// Creates a temporary directory under `install_dir/temp`, with a certain prefix.
    pub(crate) fn create_temp_dir(&self, prefix: &str) -> Result<TempDir> {
        let root = self.temp_root();

        tempfile::Builder::new()
            .prefix(&format!("{prefix}_"))
            .tempdir_in(root)
            .with_context(|| format!("unable to create temp directory under '{}'", root.display()))
    }
}

fn send_and_print<S: std::fmt::Display>(msg: S, sender: &mut MultiThreadProgress) -> Result<()> {
    println!("{msg}");
    sender.send_msg(msg.to_string())?;
    Ok(())
}

pub fn default_install_dir() -> PathBuf {
    utils::home_dir().join(format!("{}-rust", t!("vendor_en")))
}

// TODO: Write version info after installing each tool,
// which is later used for updating.
fn install_tool(
    config: &InstallConfiguration,
    name: &str,
    tool: &ToolInfo,
    proxy: Option<&Proxy>,
) -> Result<()> {
    match tool {
        ToolInfo::PlainVersion(version) => {
            if config.cargo_is_installed {
                install_and_add_fingerprint_with_cargo(
                    config,
                    name,
                    vec!["install", name, "--version", version],
                )?;
            }
        }
        ToolInfo::DetailedVersion { ver, .. } => {
            if config.cargo_is_installed {
                install_and_add_fingerprint_with_cargo(
                    config,
                    name,
                    vec!["install", name, "--version", ver],
                )?;
            }
        }
        ToolInfo::Git {
            git,
            branch,
            tag,
            rev,
            ..
        } => {
            if !config.cargo_is_installed {
                return Ok(());
            }

            let mut args = vec!["install", "--git", git.as_str()];
            if let Some(s) = &branch {
                args.extend(["--branch", s]);
            }
            if let Some(s) = &tag {
                args.extend(["--tag", s]);
            }
            if let Some(s) = &rev {
                args.extend(["--rev", s]);
            }

            install_and_add_fingerprint_with_cargo(config, name, args)?;
        }
        ToolInfo::Path { path, .. } => {
            install_and_add_fingerprint_without_cargo(config, name, path)?
        }
        // FIXME: Have a dedicated download folder, do not use temp dir to store downloaded artifacts,
        // so then we can have the `resume download` feature.
        ToolInfo::Url { url, .. } => {
            // TODO: Download first
            let temp_dir = config.create_temp_dir("download")?;

            let downloaded_file_name = url
                .path_segments()
                .ok_or_else(|| anyhow!("unsupported url format '{url}'"))?
                .last()
                // Sadly, a path segment could be empty string, so we need to filter that out
                .filter(|seg| !seg.is_empty())
                .ok_or_else(|| anyhow!("'{url}' doesn't appear to be a downloadable file"))?;

            let dest = temp_dir.path().join(downloaded_file_name);

            utils::download(name, url, &dest, proxy)?;
            // TODO: Then do the `extract or copy to` like `ToolInfo::Path`
            install_and_add_fingerprint_without_cargo(config, name, &dest)?;
        }
    }
    Ok(())
}

fn try_install_from_path(
    config: &InstallConfiguration,
    name: &str,
    path: &Path,
) -> Result<PathBuf> {
    if !path.exists() {
        bail!(
            "unable to install '{name}' because the path to it's installer '{}' does not exist.",
            path.display()
        );
    }

    let temp_dir = config.create_temp_dir(name)?;
    let tool_installer_path = extract_or_copy_to(path, temp_dir.path())?;
    let tool_installer = Tool::from_path(name, &tool_installer_path)
        .with_context(|| format!("no install method for tool '{name}'"))?;
    let dest = tool_installer.install(config)?;
    Ok(dest)
}

/// Perform extraction or copy action base on the given path.
///
/// If `maybe_file` is a path to compressed file, this will try to extract it to `dest`;
/// otherwise this will copy that file into dest.
fn extract_or_copy_to(maybe_file: &Path, dest: &Path) -> Result<PathBuf> {
    if let Ok(mut extractable) = Extractable::load(maybe_file) {
        extractable.extract_to(dest)?;
        Ok(dest.to_path_buf())
    } else {
        utils::copy_into(maybe_file, dest)
    }
}

fn add_rust_fingerprint(
    install_dir: &PathBuf,
    rust_version: &str,
    rust_components: &[String],
) -> Result<()> {
    let mut fingerprint = FingerPrint::load_fingerprint(install_dir);
    fingerprint.record_rust(rust_version.to_string(), rust_components.to_vec());

    let fingerprint_file_path = install_dir.join(".fingerprint");
    let fingerprint_content = fingerprint.to_toml()?;
    utils::write_file(fingerprint_file_path, &fingerprint_content, false)?;

    Ok(())
}

#[allow(unused_mut)]
fn add_tool_fingerprint(
    install_dir: &PathBuf,
    use_cargo: bool,
    tool_name: &str,
    tool_path: Option<PathBuf>,
) -> Result<()> {
    // Add the tool to the fingerprint.
    let mut fingerprint = FingerPrint::load_fingerprint(install_dir);
    fingerprint.record_tool(use_cargo, tool_name.to_string(), tool_path);

    let fingerprint_file_path = install_dir.join(".fingerprint");
    let fingerprint_content = fingerprint.to_toml()?;
    utils::write_file(fingerprint_file_path, &fingerprint_content, false)?;

    Ok(())
}

fn install_and_add_fingerprint_with_cargo(
    config: &InstallConfiguration,
    name: &str,
    args: Vec<&str>,
) -> Result<()> {
    utils::execute("cargo", &args)?;
    // Add the tool to the fingerprint.
    add_tool_fingerprint(&config.install_dir, true, name, None)?;
    Ok(())
}

fn install_and_add_fingerprint_without_cargo(
    config: &InstallConfiguration,
    name: &str,
    path: &PathBuf,
) -> Result<()> {
    let tool_path = try_install_from_path(config, name, path)?;
    // Add the tool to the fingerprint.
    add_tool_fingerprint(&config.install_dir, false, name, Some(tool_path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declare_unfallible_url_macro() {
        let default_dist_server = default_rustup_dist_server();
        let default_update_root = default_rustup_update_root();

        assert_eq!(
            default_dist_server.as_str(),
            "https://mirrors.tuna.tsinghua.edu.cn/rustup"
        );
        assert_eq!(
            default_update_root.as_str(),
            "https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup"
        );
    }
    #[test]
    #[cfg(windows)]
    fn test_create_local_install_info() {
        let install_dir = PathBuf::from("D:\\path\\to");
        let mut install_manifest = FingerPrint::load_fingerprint(&install_dir);
        let rust_version = "stable";
        let rust_components = vec![String::from("rustfmt"), String::from("cargo")];

        install_manifest.record_rust(rust_version.to_string(), rust_components);

        install_manifest.record_tool(false, "aaa".to_string(), Some(install_dir.join("aaa")));

        let install_manifest = install_manifest.to_toml().unwrap();

        let v0: &str = r#"[rust]
version = "stable"
components = ["rustfmt", "cargo"]

[tools.aaa]
use-cargo = false
paths = ['D:\path\to\aaa']
"#;
        assert_eq!(v0, install_manifest);
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_local_install_info() {
        let install_dir = PathBuf::from("/path/to");
        let mut install_manifest = FingerPrint::load_fingerprint(&install_dir);
        let rust_version = "stable";
        let rust_components = vec![String::from("rustfmt"), String::from("cargo")];

        install_manifest.record_rust(rust_version.to_string(), rust_components);

        install_manifest.record_tool(false, "aaa".to_string(), Some(install_dir.join("aaa")));

        let install_manifest = install_manifest.to_toml().unwrap();

        let v0: &str = r#"[rust]
version = "stable"
components = ["rustfmt", "cargo"]

[tools.aaa]
use-cargo = false
paths = ["/path/to/aaa"]
"#;
        assert_eq!(v0, install_manifest);
    }
}

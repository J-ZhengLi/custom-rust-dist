use super::{
    parser::{
        cargo_config::CargoConfig,
        manifest::{ToolInfo, ToolsetManifest},
        TomlParser,
    },
    rustup::Rustup,
    CARGO_HOME, RUSTUP_DIST_SERVER, RUSTUP_HOME, RUSTUP_UPDATE_ROOT,
};
use crate::{
    core::custom_instructions,
    utils::{self, Extractable},
};
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
};
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

declare_unfallible_url!(
    default_rustup_dist_server(DEFAULT_RUSTUP_DIST_SERVER) -> "https://mirrors.tuna.tsinghua.edu.cn/rustup";
    default_rustup_update_root(DEFAULT_RUSTUP_UPDATE_ROOT) -> "https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup"
);

// If you change any public fields in this struct,
// make sure to change `installer/src/utils/types/InstallConfiguration.ts` as well.
#[derive(Debug, Deserialize, Serialize)]
pub struct InstallConfiguration {
    pub cargo_registry: Option<(String, Url)>,
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
    pub fn init(install_dir: PathBuf, dry_run: bool) -> Result<Self> {
        let this = Self {
            install_dir,
            ..Default::default()
        };

        if !dry_run {
            // Create a new folder to hold installation
            let folder = &this.install_dir;
            utils::mkdirs(folder)?;

            // Create a copy of this binary to install dir
            let self_exe = std::env::current_exe()?;
            let cargo_bin_dir = this.cargo_home().join("bin");
            utils::mkdirs(&cargo_bin_dir)?;
            utils::copy_file_to(self_exe, &cargo_bin_dir)?;

            // Create tools directory to store third party tools
            utils::mkdirs(this.tools_dir())?;

            #[cfg(windows)]
            // Create registry entry to add this program into "installed programs".
            super::os::windows::do_add_to_programs()?;
        }

        Ok(this)
    }

    pub fn cargo_registry(mut self, registry: Option<(String, Url)>) -> Self {
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

    pub(crate) fn cargo_home(&self) -> PathBuf {
        self.install_dir.join(".cargo")
    }

    pub(crate) fn cargo_bin(&self) -> PathBuf {
        self.cargo_home().join("bin")
    }

    pub(crate) fn rustup_home(&self) -> PathBuf {
        self.install_dir.join(".rustup")
    }

    pub(crate) fn temp_root(&self) -> PathBuf {
        self.install_dir.join("temp")
    }

    pub(crate) fn tools_dir(&self) -> PathBuf {
        self.install_dir.join("tools")
    }

    pub(crate) fn env_vars(&self) -> Result<Vec<(&'static str, String)>> {
        let cargo_home = self
            .cargo_home()
            .to_str()
            .map(ToOwned::to_owned)
            .context("`install-dir` cannot contains invalid unicodes")?;
        // This `unwrap` is safe here because we've already make sure the `install_dir`'s path can be
        // converted to string with the `cargo_home` variable.
        let rustup_home = self.rustup_home().to_str().unwrap().to_string();

        let env_vars: Vec<(&str, String)> = vec![
            (RUSTUP_DIST_SERVER, self.rustup_dist_server.to_string()),
            (RUSTUP_UPDATE_ROOT, self.rustup_update_root.to_string()),
            (CARGO_HOME, cargo_home),
            (RUSTUP_HOME, rustup_home),
        ];
        Ok(env_vars)
    }

    /// Install rust's toolchain manager `rustup` with a default toolchain
    pub(crate) fn install_rust(&mut self, manifest: &ToolsetManifest) -> Result<()> {
        self.install_rust_with_optional_components(manifest, None)
    }

    pub fn install_rust_with_optional_components(
        &mut self,
        manifest: &ToolsetManifest,
        components: Option<Vec<&String>>,
    ) -> Result<()> {
        println!("installing rustup and rust toolchain");
        Rustup::init().download_toolchain(self, manifest, components)?;
        self.cargo_is_installed = true;
        Ok(())
    }

    /// Steps to install third-party softwares (excluding the ones that requires `cargo install`).
    pub(crate) fn install_tools(&self, manifest: &ToolsetManifest) -> Result<()> {
        let tools_to_install = manifest.current_target_tools();
        self.install_set_of_tools(tools_to_install)
    }

    pub fn install_set_of_tools(&self, tools: BTreeMap<&String, &ToolInfo>) -> Result<()> {
        for (name, tool) in tools {
            // Ignore tools that need to be installed using `cargo install`
            if need_cargo_install(tool) {
                continue;
            }
            println!("installing '{name}'");
            install_tool(self, name, tool)?;
        }

        Ok(())
    }

    /// Steps to install `cargo` compatible softwares, should only be called after toolchain installation.
    pub(crate) fn cargo_install(&self, manifest: &ToolsetManifest) -> Result<()> {
        let tools_to_install = manifest.current_target_tools();
        self.cargo_install_set_of_tools(tools_to_install)
    }

    pub fn cargo_install_set_of_tools(&self, tools: BTreeMap<&String, &ToolInfo>) -> Result<()> {
        for (name, tool) in tools {
            if need_cargo_install(tool) {
                println!("installing '{name}'");
                install_tool(self, name, tool)?;
            }
        }

        Ok(())
    }

    /// Configuration options for `cargo`.
    ///
    /// This will write a `config.toml` file to `CARGO_HOME`.
    pub fn config_cargo(&self) -> Result<()> {
        let mut config = CargoConfig::new();
        if let Some((name, url)) = &self.cargo_registry {
            config.add_source(name, url.to_owned(), true);
        }

        let config_toml = config.to_toml()?;
        if !config_toml.trim().is_empty() {
            // make sure cargo_home dir exists
            let cargo_home = self.cargo_home();
            utils::mkdirs(&cargo_home)?;

            let config_path = cargo_home.join("config.toml");
            utils::write_file(config_path, &config_toml, false)?;
        }

        Ok(())
    }

    /// Creates a temporary directory under `install_dir/temp`, with a certain prefix.
    pub(crate) fn create_temp_dir(&self, prefix: &str) -> Result<TempDir> {
        let root = self.temp_root();
        // Ensure temp directory
        utils::mkdirs(&root)?;

        tempfile::Builder::new()
            .prefix(&format!("{prefix}_"))
            .tempdir_in(&root)
            .with_context(|| format!("unable to create temp directory under '{}'", root.display()))
    }
}

fn need_cargo_install(tool: &ToolInfo) -> bool {
    matches!(tool, ToolInfo::PlainVersion(_) | ToolInfo::Git { .. })
}

pub fn default_install_dir() -> PathBuf {
    utils::home_dir().join(env!("CARGO_PKG_NAME"))
}

// TODO: Write version info after installing each tool,
// which is later used for updating.
fn install_tool(config: &InstallConfiguration, name: &str, tool: &ToolInfo) -> Result<()> {
    match tool {
        ToolInfo::PlainVersion(version) if config.cargo_is_installed => {
            utils::execute_with_env(
                "cargo",
                &["install", name, "--version", version],
                [
                    (CARGO_HOME, utils::path_to_str(&config.cargo_home())?),
                    (RUSTUP_HOME, utils::path_to_str(&config.rustup_home())?),
                ],
            )?;
        }
        ToolInfo::Git {
            git,
            branch,
            tag,
            rev,
            ..
        } if config.cargo_is_installed => {
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

            utils::execute_with_env(
                "cargo",
                &args,
                [
                    (CARGO_HOME, utils::path_to_str(&config.cargo_home())?),
                    (RUSTUP_HOME, utils::path_to_str(&config.rustup_home())?),
                ],
            )?;
        }
        ToolInfo::Path { path, .. } => try_install_from_path(config, name, path)?,
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

            utils::download_from_start(name, url, &dest)?;
            // TODO: Then do the `extract or copy to` like `ToolInfo::Path`
            try_install_from_path(config, name, &dest)?;
        }
        // Don't try to install tools that requires `cargo install` if `cargo` isn't even installed.
        _ => (),
    }
    Ok(())
}

fn try_install_from_path(config: &InstallConfiguration, name: &str, path: &Path) -> Result<()> {
    if !path.exists() {
        bail!(
            "unable to install '{name}' because the path to it's installer '{}' does not exist.",
            path.display()
        );
    }

    let temp_dir = config.create_temp_dir(name)?;
    let tool_installer_path = extract_or_copy_to(path, temp_dir.path())?;
    let tool_installer = ToolInstaller::from_path(name, &tool_installer_path)
        .with_context(|| format!("no install method for tool '{name}'"))?;
    tool_installer.install(config)?;
    Ok(())
}

/// Perform extraction or copy action base on the given path.
///
/// If `maybe_file` is a path to compressed file, this will try to extract it to `dest`;
/// otherwise this will copy that file into dest.
fn extract_or_copy_to(maybe_file: &Path, dest: &Path) -> Result<PathBuf> {
    if let Ok(extractable) = Extractable::try_from(maybe_file) {
        extractable.extract_to(dest)?;
        Ok(dest.to_path_buf())
    } else {
        utils::copy_to(maybe_file, dest)
    }
}

/// Representing the structure of an (extracted) tool's directory.
enum ToolInstaller<'a> {
    /// Pre-built executable files.
    /// i.e.:
    /// ```text
    /// ├─── some_binary.exe
    /// ├─── cargo-some_binary.exe
    /// ```
    Executables(Vec<PathBuf>),
    /// Plugin file, such as `.vsix` files for Visual Studio.
    Plugin { kind: PluginType, path: &'a Path },
    /// Directory containing `bin` subfolder:
    /// ```text
    /// tool/
    /// ├─── bin/
    /// ├─── ...
    /// ```
    DirWithBin { name: &'a str, bin_dir: PathBuf },
    /// We have a custom "script" for how to deal with such directory.
    Custom { name: &'a str, path: &'a Path },
}

impl<'a> ToolInstaller<'a> {
    fn from_path(name: &'a str, path: &'a Path) -> Result<Self> {
        if !path.exists() {
            bail!(
                "the installer path for '{name}' specified as '{}' does not exist.",
                path.display()
            );
        }
        // Step 1: Looking for custom install instruction
        if custom_instructions::SUPPORTED_TOOLS.contains(&name) {
            return Ok(Self::Custom { name, path });
        }

        // Step 2: Identify from file extension (if it's a file ofc).
        if utils::is_executable(path) {
            return Ok(Self::Executables(vec![path.to_path_buf()]));
        } else if path.is_file() {
            let maybe_extension = path.extension();
            if let Some(ext) = maybe_extension.and_then(|ext| ext.to_str()) {
                match ext {
                    "vsix" => {
                        // TODO: When installing, invoke `vscode` plugin install command,
                        // this must be handled after `VS-Code` has been installed,
                        // we might need a `requirements` field in the manifest.
                        return Ok(Self::Plugin {
                            kind: ext.parse()?,
                            path,
                        });
                    }
                    _ => bail!("failed to install '{name}': unknown file format '{ext}'"),
                }
            }
        }
        // TODO: Well, we got a directory, things are getting complicated, there could be one of this scenarios:
        // 1. Directory contains some executable files and nothing else
        //      Throw these executable files into cargo bin folder
        // 2. Directory contains sub-directory, which look like `bin/ lib/ etc/ ...`
        //      Throw and merge this directories into cargo home. (might be bad, therefore we need a `Manifest.in`!!!)
        // 3. Directory doesn't fit all previous characteristics.
        //      We don't know how to install this tool, throw an error instead.
        else {
            // Step 3: read directory to find characteristics.
            let entries = utils::walk_dir(path)?;
            // Check if there is any folder that looks like `bin`
            // Then assuming this is `UsrDirs` type installer.
            if let Some(bin_dir) = entries.iter().find(|path| path.ends_with("bin")) {
                return Ok(Self::DirWithBin {
                    name,
                    bin_dir: bin_dir.to_owned(),
                });
            }
            // If no sub folder exists, and there are binaries lays directly in the folder
            if !entries.iter().any(|path| path.is_dir()) {
                let assumed_binaries = entries
                    .iter()
                    .filter_map(|path| utils::is_executable(path).then_some(path.to_path_buf()));
                return Ok(Self::Executables(assumed_binaries.collect()));
            }
        }

        bail!("installing for '{name}' is not supported")
    }

    fn install(&self, config: &InstallConfiguration) -> Result<()> {
        match self {
            Self::Executables(exes) => {
                for exe in exes {
                    utils::copy_file_to(exe, config.cargo_bin())?;
                }
            }
            Self::Custom { name, path } => {
                custom_instructions::install(name, path, config)?;
            }
            Self::DirWithBin { name, bin_dir } => {
                install_dir_with_bin_(config, name, bin_dir)?;
            }
            Self::Plugin { kind, path } => {
                // First, we need to "cache" to installer, so that we could uninstall with it.
                utils::copy_file_to(path, config.tools_dir())?;
                // Then, run the installation command.
                kind.install_plugin(path)?;
            }
        }
        Ok(())
    }
}

/// Installing [`ToolInstaller::DirWithBin`], with a couple steps:
/// - Move the `tool_dir` to [`tools_dir`](InstallConfiguration::tools_dir).
/// - Add the `bin_dir` to PATH
fn install_dir_with_bin_(config: &InstallConfiguration, name: &str, bin_dir: &Path) -> Result<()> {
    let dir = config.tools_dir().join(name);
    // Safe to unwrap, because we already checked the `bin` dir is inside `tool_dir`
    let tool_dir = bin_dir.parent().unwrap();

    utils::move_to(tool_dir, &dir)?;

    let bin_dir_after_move = dir.join("bin");
    super::os::add_to_path(&bin_dir_after_move)
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
enum PluginType {
    Vsix,
}

pub(crate) static VSCODE_FAMILY: &[&str] = &[
    "code",
    "code-oss",
    "code-exploration",
    "vscode-huawei",
    "wecode",
];

impl FromStr for PluginType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "vsix" => Ok(Self::Vsix),
            _ => bail!("unsupprted plugin file type '{s}'"),
        }
    }
}

impl PluginType {
    fn install_plugin(&self, plugin_path: &Path) -> Result<()> {
        match self {
            PluginType::Vsix => {
                let executables_to_check = VSCODE_FAMILY
                    .iter()
                    .flat_map(|s| {
                        if cfg!(windows) {
                            vec![format!("{s}.cmd"), format!("{s}.exe")]
                        } else {
                            vec![s.to_string()]
                        }
                    })
                    .collect::<Vec<_>>();
                for program in executables_to_check {
                    if utils::cmd_exist(&program) {
                        utils::stdout_output(
                            &program,
                            &["--install-extension", utils::path_to_str(plugin_path)?],
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
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
}

use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{bail, Result};
use log::info;

use super::{
    directories::RimDir, parser::fingerprint::ToolRecord, uninstall::UninstallConfiguration,
    PathExt, CARGO_HOME,
};
use crate::{core::custom_instructions, setter, utils, InstallConfiguration};

#[derive(Debug)]
pub(crate) struct Tool<'a> {
    name: String,
    path: PathExt<'a>,
    pub(crate) kind: ToolKind,
    /// Additional args to run installer, currently only used for `cargo install`.
    install_args: Option<Vec<&'a str>>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
/// Representing the structure of an (extracted) tool's directory.
// NB: Mind the order of the variants, they are crucial to installation/uninstallation.
pub(crate) enum ToolKind {
    /// Directory containing `bin` subfolder:
    /// ```text
    /// tool/
    /// ├─── bin/
    /// ├─── ...
    /// ```
    DirWithBin,
    /// Pre-built executable files.
    /// i.e.:
    /// ```text
    /// ├─── some_binary.exe
    /// ├─── cargo-some_binary.exe
    /// ```
    Executables,
    /// We have a custom "script" for how to deal with such directory.
    Custom,
    /// Plugin file, such as `.vsix` files for Visual Studio.
    Plugin(PluginType),
    // `Cargo` just don't make any sense
    #[allow(clippy::enum_variant_names)]
    CargoTool,
}

impl<'a> Tool<'a> {
    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn new(name: String, kind: ToolKind) -> Self {
        Self {
            name,
            kind,
            path: PathExt::default(),
            install_args: None,
        }
    }

    setter!(path(self, path: impl Into<PathExt<'a>>) { path.into() });
    setter!(install_args(self, Option<Vec<&'a str>>));

    pub(crate) fn from_path(name: &str, path: &'a Path) -> Result<Self> {
        if !path.exists() {
            bail!(
                "the path for '{name}' specified as '{}' does not exist.",
                path.display()
            );
        }
        let name = name.to_string();

        // Step 1: Looking for custom instruction
        if custom_instructions::is_supported(&name) {
            return Ok(Self::new(name, ToolKind::Custom).path(path));
        }

        // Step 2: Identify from file extension (if it's a file ofc).
        if utils::is_executable(path) {
            return Ok(Self::new(name, ToolKind::Executables).path(path));
        } else if path.is_file() {
            let maybe_extension = path.extension();
            if let Some(ext) = maybe_extension.and_then(|ext| ext.to_str()) {
                match ext {
                    "vsix" => {
                        // TODO: When installing, invoke `vscode` plugin install command,
                        // this must be handled after `VS-Code` has been installed,
                        // we might need a `requirements` field in the manifest.
                        return Ok(Self::new(name, ToolKind::Plugin(ext.parse()?)).path(path));
                    }
                    _ => bail!("unable to process tool '{name}': unknown file format '{ext}'"),
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
            let entries = utils::walk_dir(path, false)?;
            // Check if there is any folder that looks like `bin`
            // Then assuming this is `UsrDirs` type installer.
            if entries.iter().any(|path| path.ends_with("bin")) {
                return Ok(Self::new(name, ToolKind::DirWithBin).path(path));
            }
            // If no sub folder exists, and there are binaries lays directly in the folder
            if !entries.iter().any(|path| path.is_dir()) {
                let assumed_binaries = entries
                    .iter()
                    .filter_map(|path| utils::is_executable(path).then_some(path.to_path_buf()))
                    .collect::<Vec<_>>();
                return Ok(Self::new(name, ToolKind::Executables).path(assumed_binaries));
            }
        }

        bail!("unable to process tool '{name}' as it is not supported")
    }

    /// Specify as a tool that managed by `cargo`.
    ///
    /// Note: `extra_args` should not contains "install" and `name`.
    pub(crate) fn cargo_tool(name: &str, extra_args: Option<Vec<&'a str>>) -> Self {
        Self::new(name.to_string(), ToolKind::CargoTool).install_args(extra_args)
    }

    pub(crate) fn install(&self, config: &InstallConfiguration) -> Result<ToolRecord> {
        match self.kind {
            ToolKind::CargoTool => {
                if !config.cargo_is_installed {
                    bail!(
                        "trying to install '{}' using cargo, but cargo is not installed",
                        self.name()
                    );
                }

                cargo_install_or_uninstall(
                    "install",
                    self.install_args.as_deref().unwrap_or(&[self.name()]),
                    config.cargo_home(),
                )?;
                Ok(ToolRecord::cargo_tool())
            }

            ToolKind::Executables => {
                let mut res = vec![];
                for exe in self.path.iter() {
                    res.push(utils::copy_file_to(exe, config.cargo_bin())?);
                }
                Ok(ToolRecord::with_paths(res))
            }
            ToolKind::Custom => {
                let paths =
                    custom_instructions::install(self.name(), self.path.expect_single(), config)?;
                Ok(ToolRecord::with_paths(paths))
            }
            ToolKind::DirWithBin => {
                let tool_dir =
                    install_dir_with_bin_(config, self.name(), self.path.expect_single())?;
                Ok(ToolRecord::with_paths(vec![tool_dir]))
            }
            ToolKind::Plugin(kind) => {
                let path = self.path.expect_single();
                // run the installation command.
                kind.install_plugin(path)?;
                // we need to "cache" to installer, so that we could uninstall with it.
                let plugin_backup = utils::copy_file_to(path, config.tools_dir())?;
                Ok(ToolRecord::with_paths(vec![plugin_backup]))
            }
        }
    }

    pub(crate) fn uninstall(&self, config: &UninstallConfiguration) -> Result<()> {
        match self.kind {
            ToolKind::CargoTool => {
                cargo_install_or_uninstall(
                    "uninstall",
                    self.install_args.as_deref().unwrap_or(&[self.name()]),
                    config.cargo_home(),
                )?;
            }
            ToolKind::Executables => {
                for binary in self.path.iter() {
                    fs::remove_file(binary)?;
                }
            }
            ToolKind::Custom => custom_instructions::uninstall(self.name(), config)?,
            ToolKind::DirWithBin => uninstall_dir_with_bin_(self.path.expect_single())?,
            ToolKind::Plugin(kind) => kind.uninstall_plugin(self.path.expect_single())?,
        }
        Ok(())
    }
}

fn cargo_install_or_uninstall(op: &str, args: &[&str], cargo_home: &Path) -> Result<()> {
    let mut cargo_bin = cargo_home.to_path_buf();
    cargo_bin.push("bin");
    cargo_bin.push(utils::exe!("cargo"));

    utils::Command::new(cargo_bin)
        .arg(op)
        .args(args)
        .env(CARGO_HOME, cargo_home)
        .run()
}

/// Installing [`ToolInstaller::DirWithBin`], with a couple steps:
/// - Move the `tool_dir` to [`tools_dir`](InstallConfiguration::tools_dir).
/// - Add the `bin_dir` to PATH
fn install_dir_with_bin_(
    config: &InstallConfiguration,
    name: &str,
    path: &Path,
) -> Result<PathBuf> {
    let dir = config.tools_dir().join(name);

    utils::move_to(path, &dir, true)?;

    let bin_dir_after_move = dir.join("bin");
    super::os::add_to_path(&bin_dir_after_move)?;
    Ok(dir)
}

/// Uninstalling a tool with bin folder is as simple as removing the directory,
/// and removing the `bin` dir from `PATH`.
fn uninstall_dir_with_bin_(tool_path: &Path) -> Result<()> {
    // Remove from `PATH` at first.
    let bin_dir = tool_path.join("bin");
    super::os::remove_from_path(&bin_dir)?;

    fs::remove_dir_all(tool_path)?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[non_exhaustive]
pub(crate) enum PluginType {
    Vsix,
}

// This list has a fallback order, DO NOT change the order.
#[cfg(not(windows))]
pub(crate) static VSCODE_FAMILY: &[&str] =
    &["hwcode", "wecode", "code-exploration", "code-oss", "code"];
#[cfg(windows)]
pub(crate) static VSCODE_FAMILY: &[&str] = &[
    "hwcode.cmd",
    "wecode.cmd",
    "code-exploration.cmd",
    "code-oss.cmd",
    "code.cmd",
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
    fn install_or_uninstall_(&self, plugin_path: &Path, uninstall: bool) -> Result<()> {
        match self {
            PluginType::Vsix => {
                for program in VSCODE_FAMILY {
                    if utils::cmd_exist(program) {
                        let op = if uninstall { "uninstall" } else { "install" };
                        let arg_opt = format!("--{op}-extension");
                        info!(
                            "{}",
                            t!(
                                "handling_extension_info",
                                op = t!(op),
                                ext = plugin_path.display(),
                                program = program
                            )
                        );
                        match utils::Command::new(program)
                            .arg(arg_opt)
                            .arg(plugin_path)
                            .run()
                        {
                            Ok(()) => continue,
                            // Ignore error when uninstalling.
                            Err(_) if uninstall => {
                                info!(
                                    "{}",
                                    t!(
                                        "skip_extension_uninstall_warn",
                                        ext = plugin_path.display(),
                                        program = program
                                    )
                                );
                                continue;
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }

                // Remove the plugin file if uninstalling
                if uninstall {
                    utils::remove(plugin_path)?;
                }
            }
        }
        Ok(())
    }

    fn install_plugin(&self, plugin_path: &Path) -> Result<()> {
        self.install_or_uninstall_(plugin_path, false)
    }

    fn uninstall_plugin(&self, plugin_path: &Path) -> Result<()> {
        self.install_or_uninstall_(plugin_path, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_order() {
        let mut tools = vec![];

        tools.push(ToolKind::Executables);
        tools.push(ToolKind::CargoTool);
        tools.push(ToolKind::Custom);
        tools.push(ToolKind::Plugin(PluginType::Vsix));
        tools.push(ToolKind::DirWithBin);
        tools.push(ToolKind::Executables);

        tools.sort();

        let mut tools_iter = tools.iter();
        assert!(matches!(tools_iter.next(), Some(ToolKind::DirWithBin)));
        assert!(matches!(tools_iter.next(), Some(ToolKind::Executables)));
        assert!(matches!(tools_iter.next(), Some(ToolKind::Executables)));
        assert!(matches!(tools_iter.next(), Some(ToolKind::Custom)));
        assert!(matches!(
            tools_iter.next(),
            Some(ToolKind::Plugin(PluginType::Vsix))
        ));
        assert!(matches!(tools_iter.next(), Some(ToolKind::CargoTool)));
        assert!(matches!(tools_iter.next(), None));
    }

    #[test]
    fn tools_order_reversed() {
        let mut tools = vec![];

        tools.push(ToolKind::Executables);
        tools.push(ToolKind::CargoTool);
        tools.push(ToolKind::Custom);
        tools.push(ToolKind::Plugin(PluginType::Vsix));
        tools.push(ToolKind::DirWithBin);
        tools.push(ToolKind::Executables);

        tools.sort_by(|a, b| b.cmp(a));

        let mut tools_iter = tools.iter();
        assert!(matches!(tools_iter.next(), Some(ToolKind::CargoTool)));
        assert!(matches!(
            tools_iter.next(),
            Some(ToolKind::Plugin(PluginType::Vsix))
        ));
        assert!(matches!(tools_iter.next(), Some(ToolKind::Custom)));
        assert!(matches!(tools_iter.next(), Some(ToolKind::Executables)));
        assert!(matches!(tools_iter.next(), Some(ToolKind::Executables)));
        assert!(matches!(tools_iter.next(), Some(ToolKind::DirWithBin)));
        assert!(matches!(tools_iter.next(), None));
    }
}

use std::path::PathBuf;

use anyhow::Result;
use indexmap::IndexMap;

use super::{
    os::install_dir_from_exe_path,
    parser::{
        fingerprint::{installed_tools_fresh, InstallationRecord, ToolRecord},
        TomlParser,
    },
    rustup::ToolchainInstaller,
};
use crate::core::tools::Tool;

/// Contains definition of uninstallation steps.
pub(crate) trait Uninstallation {
    /// Remove persistent environment variables for `rustup`.
    ///
    /// This will remove persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`.
    fn remove_rustup_env_vars(&self, install_dir: &PathBuf) -> Result<()>;
    /// The last step of uninstallation, this will remove the binary itself, along with
    /// the folder it's in.
    fn remove_self(&self, install_dir: &PathBuf) -> Result<()>;
}

/// Configurations to use when installing.
pub(crate) struct UninstallConfiguration {
    /// The installation directory that holds every tools, configuration files,
    /// including the manager binary.
    pub(crate) install_dir: PathBuf,
    pub(crate) install_record: InstallationRecord,
}

impl UninstallConfiguration {
    pub(crate) fn init() -> Result<Self> {
        let install_dir = install_dir_from_exe_path()?;
        let install_record = InstallationRecord::load(&install_dir)?;
        Ok(Self {
            install_dir,
            install_record,
        })
    }

    pub(crate) fn tools_dir(&self) -> PathBuf {
        self.install_dir.join("tools")
    }

    pub(crate) fn cargo_home(&self) -> PathBuf {
        self.install_dir.join(".cargo")
    }

    pub(crate) fn uninstall(mut self, remove_self: bool) -> Result<()> {
        let install_dir = self.install_dir.clone();
        // remove all tools.
        self.remove_tools(installed_tools_fresh(&install_dir)?)?;

        // Remove rust toolchain via rustup.
        ToolchainInstaller::init().remove_self(&install_dir)?;
        self.install_record.remove_rust_record();

        // remove all the environments.
        self.remove_rustup_env_vars(&install_dir)?;

        if remove_self {
            self.remove_self(&install_dir)?;
            // TODO: fix core::os::unix::remove_from_path()
            // Rmove the `<InstallDir>` which is added for manager.
            crate::core::os::remove_from_path(&install_dir)?;
        } else {
            self.install_record.write()?;
        }

        Ok(())
    }

    /// Uninstall all tools
    fn remove_tools(&mut self, tools: IndexMap<String, ToolRecord>) -> Result<()> {
        for (name, tool_detail) in &tools {
            let tool = if tool_detail.use_cargo {
                Tool::cargo_tool(name, None)
            } else if let [path] = tool_detail.paths.as_slice() {
                Tool::from_path(name, path)?
            } else if !tool_detail.paths.is_empty() {
                Tool::Executables(name.into(), tool_detail.paths.clone())
            } else {
                println!("{}", t!("uninstall_unknown_tool_warn", tool = name));
                return Ok(());
            };

            if tool.uninstall(self).is_err() {
                println!("{}", t!("uninstall_tool_failed_warn"));
            } else {
                self.install_record.remove_tool_record(name);
            }
        }

        Ok(())
    }
}

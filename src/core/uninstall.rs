use std::path::PathBuf;

use anyhow::Result;
use indexmap::IndexMap;

use super::{
    directories::RimDir,
    parser::fingerprint::{installed_tools_fresh, InstallationRecord, ToolRecord},
    rustup::ToolchainInstaller,
};
use crate::{core::tools::Tool, utils::MultiThreadProgress};

/// Contains definition of uninstallation steps.
pub(crate) trait Uninstallation {
    /// Remove persistent environment variables for `rustup`.
    ///
    /// This will remove persistent environment variables including
    /// `RUSTUP_DIST_SERVER`, `RUSTUP_UPDATE_ROOT`, `CARGO_HOME`, `RUSTUP_HOME`.
    fn remove_rustup_env_vars(&self) -> Result<()>;
    /// The last step of uninstallation, this will remove the binary itself, along with
    /// the folder it's in.
    fn remove_self(&self) -> Result<()>;
}

/// Configurations to use when installing.
pub struct UninstallConfiguration {
    /// The installation directory that holds every tools, configuration files,
    /// including the manager binary.
    pub(crate) install_dir: PathBuf,
    pub(crate) install_record: InstallationRecord,
}

impl RimDir for UninstallConfiguration {
    fn install_dir(&self) -> &std::path::Path {
        self.install_dir.as_path()
    }
}

impl UninstallConfiguration {
    pub fn init() -> Result<Self> {
        let install_record = InstallationRecord::load_from_install_dir()?;
        Ok(Self {
            install_dir: install_record.root.clone(),
            install_record,
        })
    }

    pub fn uninstall(self, remove_self: bool) -> Result<()> {
        let mut dummy_progress = MultiThreadProgress::default();
        self.uninstall_with_progress(remove_self, &mut dummy_progress)
    }

    pub fn uninstall_with_progress(
        mut self,
        remove_self: bool,
        mt_prog: &mut MultiThreadProgress,
    ) -> Result<()> {
        // remove all tools.
        mt_prog.val = 55;
        mt_prog.send_msg_and_print(t!("uninstalling_third_party_tools"))?;
        self.remove_tools(installed_tools_fresh(&self.install_dir)?, mt_prog)?;

        // Remove rust toolchain via rustup.
        mt_prog.val = 30;
        mt_prog.send_msg_and_print(t!("uninstalling_rust_toolchain"))?;
        ToolchainInstaller::init().remove_self(&self)?;
        self.install_record.remove_rust_record();
        mt_prog.send_progress()?;

        // remove all the environments.
        mt_prog.val = 10;
        mt_prog.send_msg_and_print(t!("uninstall_env_config"))?;
        self.remove_rustup_env_vars()?;
        mt_prog.send_progress()?;

        // remove the manager binary itself or update install record
        mt_prog.val = 5;
        if remove_self {
            mt_prog.send_msg_and_print(t!("uninstall_self"))?;
            self.remove_self()?;
        } else {
            self.install_record.write()?;
        }
        mt_prog.send_progress()?;

        Ok(())
    }

    /// Uninstall all tools
    fn remove_tools(
        &mut self,
        tools: IndexMap<String, ToolRecord>,
        mt_prog: &mut MultiThreadProgress,
    ) -> Result<()> {
        let mut tools_to_uninstall = vec![];
        for (name, tool_detail) in &tools {
            let tool = if tool_detail.use_cargo {
                Tool::cargo_tool(name, None)
            } else if let [path] = tool_detail.paths.as_slice() {
                Tool::from_path(name, path)?
            } else if !tool_detail.paths.is_empty() {
                Tool::Executables(name.into(), tool_detail.paths.clone())
            } else {
                mt_prog.send_msg_and_print(t!("uninstall_unknown_tool_warn", tool = name))?;
                continue;
            };
            tools_to_uninstall.push(tool);
        }

        let progress_dt = if tools_to_uninstall.is_empty() {
            return mt_prog.send_progress();
        } else {
            mt_prog.val / tools_to_uninstall.len()
        };

        tools_to_uninstall.sort_by(|a, b| b.cmp(a));

        for tool in tools_to_uninstall {
            mt_prog.send_msg_and_print(t!("uninstalling_for", name = tool.name()))?;
            if tool.uninstall(self, mt_prog).is_err() {
                mt_prog.send_msg_and_print(t!("uninstall_tool_failed_warn", tool = tool.name()))?;
            } else {
                self.install_record.remove_tool_record(tool.name());
            }
            mt_prog.send_any_progress(progress_dt)?;
        }

        Ok(())
    }
}

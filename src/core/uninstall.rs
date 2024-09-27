use std::{fmt::Display, path::PathBuf};

use anyhow::Result;
use indexmap::IndexMap;

use super::{
    directories::RimDir,
    parser::fingerprint::{installed_tools_fresh, InstallationRecord, ToolRecord},
    rustup::ToolchainInstaller,
};
use crate::{core::tools::Tool, utils::Progress};

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
pub struct UninstallConfiguration<'a> {
    /// The installation directory that holds every tools, configuration files,
    /// including the manager binary.
    pub(crate) install_dir: PathBuf,
    pub(crate) install_record: InstallationRecord,
    progress_indicator: Option<Progress<'a>>,
}

impl RimDir for UninstallConfiguration<'_> {
    fn install_dir(&self) -> &std::path::Path {
        self.install_dir.as_path()
    }
}

impl<'a> UninstallConfiguration<'a> {
    pub fn init(progress: Option<Progress<'a>>) -> Result<Self> {
        let install_record = InstallationRecord::load_from_install_dir()?;
        Ok(Self {
            install_dir: install_record.root.clone(),
            install_record,
            progress_indicator: progress,
        })
    }

    /// Print message via progress indicator.
    pub(crate) fn show_progress<S: Display + ToString>(&self, msg: S) -> Result<()> {
        println!("{msg}");
        if let Some(prog) = &self.progress_indicator {
            prog.show_msg(msg)?;
        }
        Ok(())
    }

    pub(crate) fn inc_progress(&self, val: f32) -> Result<()> {
        if let Some(prog) = &self.progress_indicator {
            prog.inc(Some(val))?;
        }
        Ok(())
    }

    pub fn uninstall(mut self, remove_self: bool) -> Result<()> {
        // remove all tools.
        self.show_progress(t!("uninstalling_third_party_tools"))?;
        self.remove_tools(installed_tools_fresh(&self.install_dir)?, 40.0)?;

        // Remove rust toolchain via rustup.
        if self.install_record.rust.is_some() {
            self.show_progress(t!("uninstalling_rust_toolchain"))?;
            ToolchainInstaller::init().remove_self(&self)?;
            self.install_record.remove_rust_record();
            self.install_record.write()?;
        }
        self.inc_progress(40.0)?;

        // remove all env configuration.
        self.show_progress(t!("uninstall_env_config"))?;
        self.remove_rustup_env_vars()?;
        self.inc_progress(10.0)?;

        // remove the manager binary itself or update install record
        if remove_self {
            self.show_progress(t!("uninstall_self"))?;
            self.remove_self()?;
        } else {
            self.install_record.remove_toolkit_meta();
            self.install_record.write()?;
        }
        self.inc_progress(10.0)?;

        Ok(())
    }

    /// Uninstall all tools
    fn remove_tools(&mut self, tools: IndexMap<String, ToolRecord>, weight: f32) -> Result<()> {
        let mut tools_to_uninstall = vec![];
        for (name, tool_detail) in &tools {
            let tool = if tool_detail.use_cargo {
                Tool::cargo_tool(name, None)
            } else if let [path] = tool_detail.paths.as_slice() {
                Tool::from_path(name, path)?
            } else if !tool_detail.paths.is_empty() {
                Tool::Executables(name.into(), tool_detail.paths.clone())
            } else {
                self.show_progress(t!("uninstall_unknown_tool_warn", tool = name))?;
                continue;
            };
            tools_to_uninstall.push(tool);
        }

        if tools_to_uninstall.is_empty() {
            return self.inc_progress(weight);
        }
        let progress_dt = weight / tools_to_uninstall.len() as f32;

        tools_to_uninstall.sort_by(|a, b| b.cmp(a));

        for tool in tools_to_uninstall {
            self.show_progress(t!("uninstalling_for", name = tool.name()))?;
            if tool.uninstall(self).is_err() {
                self.show_progress(t!("uninstall_tool_failed_warn", tool = tool.name()))?;
            }
            self.install_record.remove_tool_record(tool.name());
            self.install_record.write()?;
            self.inc_progress(progress_dt)?;
        }

        Ok(())
    }
}

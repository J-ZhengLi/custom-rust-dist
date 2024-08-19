use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::{core::tools::Tool, utils};

use super::os::install_dir_from_exe_path;

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
// NB: Currently, there's no uninstall configurations, this struct is only
// used for abstract purpose.
pub(crate) struct UninstallConfiguration;

impl UninstallConfiguration {
    pub(crate) fn install_dir(&self) -> Result<PathBuf> {
        install_dir_from_exe_path()
    }

    pub(crate) fn tools_dir(&self) -> Result<PathBuf> {
        self.install_dir()
            .map(|install_dir| install_dir.join("tools"))
    }

    /// Uninstall any tools that may or may not installed with custom instructions.
    pub(crate) fn remove_tools(&self) -> Result<()> {
        // TODO: Read a list of tools to remove, this require a manifest file to be written after installation.
        // But right now we only remove those in `tools` directory
        let tools_to_remove = utils::walk_dir(&self.tools_dir()?, false)?;

        for tool in tools_to_remove {
            // TODO: This name should be read from manifest anyway, but right now we get the name
            // by the `folder`'s name, which technically does the same thing, but for those tools
            // that were installed without folder, things could get a little bit ugly.
            let name = if tool.is_dir() {
                tool.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                    anyhow!(
                        "cannot get the name of tool that installed in '{}'",
                        tool.display()
                    )
                })?
            } else {
                // TODO: For now, this could only mean that this tool is some extension,
                // but it may changed in the future.
                "extension"
            };

            let tool_uninstaller = Tool::from_path(name, &tool)?;
            println!("uninstalling '{name}'");
            tool_uninstaller.uninstall()?;
        }

        Ok(())
    }
}

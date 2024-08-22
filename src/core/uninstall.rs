use std::{cmp::Ordering, path::PathBuf};

use anyhow::Result;

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
        // If there's nothing to remove, do nothing
        let tools_dir = self.tools_dir()?;
        if !tools_dir.exists() {
            return Ok(());
        }
        let entries = utils::walk_dir(&tools_dir, false)?;
        let mut tools_to_remove = entries
            .iter()
            // Ignoreing the paths that cannot be recognized as a tool.
            .filter_map(tool_from_path)
            .collect::<Vec<_>>();

        // Make sure the installation order are: plugin > ...
        tools_to_remove.sort_by(|a, b| match (a, b) {
            (Tool::Plugin { .. }, Tool::Plugin { .. }) => Ordering::Equal,
            (Tool::Plugin { .. }, _) => Ordering::Greater,
            (_, Tool::Plugin { .. }) => Ordering::Less,
            _ => Ordering::Equal,
        });

        for tool in tools_to_remove {
            println!("uninstalling '{}'", tool.name());
            tool.uninstall()?;
        }

        Ok(())
    }
}

fn tool_from_path(path: &PathBuf) -> Option<Tool> {
    // TODO: This name should be read from manifest anyway, but right now we get the name
    // by the `folder`'s name, which technically does the same thing, but for those tools
    // that were installed without folder, things could get a little bit ugly.
    let name = path
        .with_extension("")
        .file_name()
        .and_then(|n| n.to_str())?
        .to_string();
    Tool::from_path(&name, path).ok()
}

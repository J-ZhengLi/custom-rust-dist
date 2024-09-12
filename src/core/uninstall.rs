use std::{cmp::Ordering, path::PathBuf};

use anyhow::Result;

use crate::{core::tools::Tool, utils};

use super::{os::install_dir_from_exe_path, parser::fingerprint::FingerPrint, rustup::Rustup};

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

    #[allow(unused)]
    pub(crate) fn tools_dir(&self) -> Result<PathBuf> {
        self.install_dir()
            .map(|install_dir| install_dir.join("tools"))
    }

    /// Uninstall any tools that may or may not installed with custom instructions.
    pub(crate) fn remove_tools(&self, fingerprint: FingerPrint) -> Result<()> {
        // remove tools by cargo or uninstall api.
        let tools = fingerprint.tools();
        // we need to remove plugins first, so add an extra sort array.
        let mut tools_without_cargo = Vec::new();
        for tool in tools.iter() {
            let tool_name = tool.0;
            let tool_detail = tool.1;
            match tool_detail.use_cargo() {
                true => {
                    // remove the tool by cargo
                    let args = &["uninstall", tool_name];
                    utils::execute("cargo", args)?;
                }
                false => {
                    // remove the tool by using tool's uninstall api.
                    let tools_to_remove = tool_detail
                        .paths()
                        .iter()
                        .filter_map(tool_from_path)
                        .collect::<Vec<_>>();
                    tools_without_cargo.extend(tools_to_remove);
                }
            }
        }

        tools_without_cargo.sort_by(|a, b| match (a, b) {
            (Tool::Plugin { .. }, Tool::Plugin { .. }) => Ordering::Equal,
            (Tool::Plugin { .. }, _) => Ordering::Less,
            (_, Tool::Plugin { .. }) => Ordering::Greater,
            _ => Ordering::Equal,
        });

        for tool in tools_without_cargo {
            println!("{}", t!("uninstalling_tool_info", name = tool.name()));
            tool.uninstall()?;
        }

        // TODO: Remove manager to other directories.
        // Remove rust toolchain via rustup.
        // Rustup::init().remove_self(self)?;

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

//! Separated module to handle uninstallation in command line.

use crate::core::uninstall::{UninstallConfiguration, Uninstallation};

use super::{common, GlobalOpt, ManagerSubcommands};

use anyhow::Result;

/// Execute `uninstall` command.
pub(super) fn execute(subcommand: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Uninstall { remove_self } = subcommand else {
        return Ok(false);
    };

    // Ask confirmation
    // TODO: format an installed list instead
    let installed = "Rust-toolchain";
    let prompt = if *remove_self {
        t!(
            "uninstall_all_confirmation",
            vendor = t!("vendor"),
            list = installed
        )
    } else {
        t!("uninstall_confirmation", list = installed)
    };
    if !common::confirm(prompt, false)? {
        return Ok(true);
    }

    let config = UninstallConfiguration;
    config.remove_rustup_env_vars()?;
    config.remove_tools()?;
    // TODO: remove rust toolchain
    if *remove_self {
        config.remove_self()?;
    }

    Ok(true)
}

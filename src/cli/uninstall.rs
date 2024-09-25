//! Separated module to handle uninstallation in command line.

use crate::core::uninstall::UninstallConfiguration;

use super::{common, GlobalOpt, ManagerSubcommands};

use anyhow::Result;

/// Execute `uninstall` command.
pub(super) fn execute(subcommand: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Uninstall { remove_self } = subcommand else {
        return Ok(false);
    };

    let config = UninstallConfiguration::init(None)?;
    let installed = config.install_record.print_installation();

    // Ask confirmation
    let prompt = if *remove_self {
        t!(
            "uninstall_all_confirmation",
            vendor = t!("vendor_en"),
            list = installed
        )
    } else {
        t!("uninstall_confirmation", list = installed)
    };
    if !common::confirm(prompt, false)? {
        return Ok(true);
    }

    config.uninstall(*remove_self)?;

    Ok(true)
}

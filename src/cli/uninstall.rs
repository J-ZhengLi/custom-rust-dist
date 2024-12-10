//! Separated module to handle uninstallation in command line.

use crate::core::uninstall::UninstallConfiguration;

use super::{common, ManagerSubcommands};

use anyhow::Result;

/// Execute `uninstall` command.
pub(super) async fn execute(subcommand: &ManagerSubcommands) -> Result<bool> {
    let ManagerSubcommands::Uninstall { keep_self } = subcommand else {
        return Ok(false);
    };

    let config = UninstallConfiguration::init(None)?;
    let installed = config.install_record.print_installation();

    // Ask confirmation
    let prompt = if !keep_self {
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

    config.uninstall(!keep_self).await?;

    Ok(true)
}

//! Separated module to handle uninstallation in command line.

use crate::cli::UninstallCommand;
use crate::core::uninstall::{UninstallConfiguration, Uninstallation};

use super::{GlobalOpt, ManagerSubcommands};

use anyhow::Result;

/// Execute `uninstall` command.
pub(super) fn execute(subcommand: &ManagerSubcommands, _opt: GlobalOpt) -> Result<()> {
    let ManagerSubcommands::Uninstall {
        commands: Some(uninst_cmd),
    } = subcommand
    else {
        return Ok(());
    };

    match uninst_cmd {
        UninstallCommand::All => {
            let config = UninstallConfiguration;
            config.remove_rustup_env_vars()?;
            config.remove_tools()?;
            config.remove_self()?;
        }
        UninstallCommand::Tool { names } => {
            // TODO: remove a certain tool, or component
            unimplemented!("attempt to remove '{names:?}', but this is not yet implemented.")
        }
    }

    Ok(())
}

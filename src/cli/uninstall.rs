//! Separated module to handle uninstallation in command line.

use crate::{
    cli::UninstallCommand,
    core::{UninstallConfiguration, Uninstallation},
};

use super::{GlobalOpt, Subcommands};

use anyhow::Result;

/// Execute `uninstall` command.
pub(super) fn execute(subcommand: &Subcommands, _opt: GlobalOpt) -> Result<()> {
    let Subcommands::Uninstall {
        commands: Some(uninst_cmd),
    } = subcommand
    else {
        return Ok(());
    };

    match uninst_cmd {
        UninstallCommand::All => {
            let config = UninstallConfiguration;
            // TODO: remove rust installation using `rustup self uninstall`
            // TODO: remove configurations
            config.remove_rustup_env_vars()?;
            // TODO: remove self
            config.remove_self()?;
            unimplemented!("`uninstall all` is not fully implemented yet.")
        }
        UninstallCommand::Tool { names } => {
            // TODO: remove a certain tool, or component
            unimplemented!("attempt to remove '{names:?}', but this is not yet implemented.")
        }
    }
}

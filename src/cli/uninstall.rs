//! Separated module to handle uninstallation in command line.

use crate::cli::UninstallCommand;

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
            // TODO: remove rust installation using `rustup self uninstall`
            // TODO: remove configurations
            // TODO: remove self
            unimplemented!("`uninstall all` is not yet implemented.")
        }
        UninstallCommand::Tool { names } => {
            // TODO: remove a certain tool, or component
            unimplemented!("attempt to remove '{names:?}', but this is not yet implemented.")
        }
    }
}

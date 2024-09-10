use anyhow::Result;
use clap::Subcommand;

use super::{GlobalOpt, ManagerSubcommands};

#[derive(Subcommand, Debug)]
pub(super) enum ListCommand {
    /// Prints a list of all components instead
    Component,
}

impl ListCommand {
    fn execute(&self) -> Result<()> {
        todo!("print a list of components")
    }
}

pub(super) fn execute(cmd: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::List {
        installed: _,
        command,
    } = cmd
    else {
        return Ok(false);
    };

    if let Some(sub_cmd) = command {
        sub_cmd.execute()?;
    } else {
        todo!("print a list of dist version");
    }

    Ok(true)
}

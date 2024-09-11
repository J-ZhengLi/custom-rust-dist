use super::{GlobalOpt, ManagerSubcommands};
use crate::core::try_it;
use anyhow::Result;

/// Execute `install` command.
pub(super) fn execute(subcommand: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::TryIt { path } = subcommand else {
        return Ok(false);
    };

    try_it::try_it(path.as_deref())?;
    Ok(true)
}

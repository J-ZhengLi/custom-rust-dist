use super::{GlobalOpt, ManagerSubcommands};
use crate::core::try_it;
use anyhow::Result;

/// Execute `install` command.
pub(super) fn execute(subcommand: &ManagerSubcommands, _opt: GlobalOpt) -> Result<()> {
    let ManagerSubcommands::TryIt { path } = subcommand else {
        return Ok(());
    };

    try_it::try_it(path.as_deref())
}

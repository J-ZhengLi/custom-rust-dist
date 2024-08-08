use super::{GlobalOpt, Subcommands};
use crate::core::try_it;
use anyhow::Result;

/// Execute `install` command.
pub(super) fn execute(subcommand: &Subcommands, _opt: GlobalOpt) -> Result<()> {
    let Subcommands::TryIt { path } = subcommand else {
        return Ok(());
    };

    try_it::try_it(path.as_deref())
}

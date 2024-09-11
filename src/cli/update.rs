use anyhow::Result;

use super::{GlobalOpt, ManagerSubcommands};

pub(super) fn execute(cmd: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Update { no_self_update: _ } = cmd else {
        return Ok(false);
    };

    todo!("implement update sub-command")
}

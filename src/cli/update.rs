use anyhow::Result;

use crate::core::update::UpdateConfiguration;

use super::{GlobalOpt, ManagerSubcommands};

pub(super) fn execute(cmd: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Update { self_update } = cmd else {
        return Ok(false);
    };

    let config: UpdateConfiguration = UpdateConfiguration;

    config.update(*self_update)?;

    Ok(true)
}

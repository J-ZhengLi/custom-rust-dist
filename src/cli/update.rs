use anyhow::Result;

use crate::core::update::UpdateConfiguration;

use super::ManagerSubcommands;

pub(super) fn execute(cmd: &ManagerSubcommands) -> Result<bool> {
    let ManagerSubcommands::Update { self_update } = cmd else {
        return Ok(false);
    };

    let config: UpdateConfiguration = UpdateConfiguration;

    config.update(*self_update)?;

    Ok(true)
}

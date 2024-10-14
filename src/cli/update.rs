use anyhow::Result;

use crate::core::update::UpdateConfiguration;

use super::{GlobalOpt, ManagerSubcommands};

pub(super) fn execute(cmd: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Update {
        self_update,
        only_manager,
    } = cmd
    else {
        return Ok(false);
    };

    println!("self_update {}", self_update);
    println!("only_manager {}", only_manager);

    let config: UpdateConfiguration = UpdateConfiguration;

    let upgradeable = config.check_upgrade()?;
    if upgradeable {
        println!(
            "A new manager version has been detected. You can update it via using `--self-update`"
        )
    }

    config.update(*self_update, *only_manager)?;

    Ok(true)
}

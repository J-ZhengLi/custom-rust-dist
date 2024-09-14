use anyhow::Result;
use clap::Subcommand;

use super::{GlobalOpt, ManagerSubcommands};

#[derive(Subcommand, Debug)]
pub(super) enum ComponentCommand {
    /// Install a set of components, check `list component` for available options
    #[command(alias = "add")]
    Install {
        /// The list of components to install
        #[arg(value_name = "COMPONENTS")]
        components: Vec<String>,
    },
    /// Uninstall a set of components, check `list component --installed` for available options
    #[command(alias = "remove")]
    Uninstall {
        /// The list of components to uninstall
        #[arg(value_name = "COMPONENTS")]
        components: Vec<String>,
    },
}

impl ComponentCommand {
    fn execute(&self, _opt: GlobalOpt) -> Result<()> {
        match self {
            Self::Install { components } => todo!("install components: {components:?}"),
            Self::Uninstall { components } => todo!("uninstall components: {components:?}"),
        }
    }
}

pub(super) fn execute(cmd: &ManagerSubcommands, opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Component { command } = cmd else {
        return Ok(false);
    };

    command.execute(opt)?;

    Ok(true)
}

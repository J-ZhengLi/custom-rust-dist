use anyhow::Result;
use clap::Subcommand;

use super::ManagerSubcommands;

#[derive(Subcommand, Debug)]
pub(super) enum ComponentCommand {
    /// Install a set of components, check `list component` for available options
    #[command(alias = "add")]
    Install {
        /// Allow insecure connections when download packages from server.
        #[arg(short = 'k', long)]
        insecure: bool,
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
    fn execute(&self) -> Result<()> {
        match self {
            Self::Install { components, .. } => todo!("install components: {components:?}"),
            Self::Uninstall { components } => todo!("uninstall components: {components:?}"),
        }
    }
}

pub(super) fn execute(cmd: &ManagerSubcommands) -> Result<bool> {
    let ManagerSubcommands::Component { command } = cmd else {
        return Ok(false);
    };

    command.execute()?;

    Ok(true)
}

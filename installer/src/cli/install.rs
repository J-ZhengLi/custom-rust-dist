use super::{GlobalOpt, InstallCommand, Subcommands};
use crate::steps;

use anyhow::Result;

/// Processing `config` subcommand if it exist, otherwise this won't do anything.
pub(super) fn process(subcommand: &Subcommands, opt: GlobalOpt) -> Result<()> {
    let Subcommands::Install {
        commands: Some(install_commands),
    } = subcommand else { return Ok(()) };

    // gather information about settings and installations
    let mut config = steps::load_config().unwrap_or_default();

    match install_commands {
        InstallCommand::Rustup { version } => {
            // install rustup
        }
        InstallCommand::Toolchain {
            toolchain,
            target,
            profile,
            component,
            path,
        } => {
            // install toolchain
            // check if path is some and is dir/file
        }
        InstallCommand::Component {
            name,
            toolchain,
            target,
        } => {
            // install toolchain component
            // no need to check if toolchain installed, just redirect rustup output
        }
        InstallCommand::Tool {
            name,
            path,
            git,
            version,
            force,
            features,
        } => {
            // install crates/tools
            // check if path is some and is dir/file
        }
        _ => (),
    }

    Ok(())
}

//! Separated module to handle installation related behaviors in command line.

use crate::{triple::TargetTriple, utils::Process};

use super::{GlobalOpt, Subcommands};

use anyhow::Result;

/// Execute `install` command.
pub(super) fn execute(subcommand: &Subcommands, _opt: GlobalOpt) -> Result<()> {
    let Subcommands::Install = subcommand else {
        return Ok(());
    };

    // TODO: handle configurations
    // TODO: download rustup then install
    // TODO: install rust toolchian via rustup
    // TODO: install third-party tools via cargo that got installed by rustup
    
    let process = Process::os();
    let triple = TargetTriple::from_host(&process).unwrap();
    println!("{:?}", triple);

    unimplemented!("`install` is not yet implemented.")
}

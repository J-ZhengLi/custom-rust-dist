//! Separated module to handle installation related behaviors in command line.

use crate::{rustup::Rustup, utils::Process};

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

    #[cfg(windows)]
    let rustup_init: &str = "rustup-init.exe";
    #[cfg(not(windows))]
    let rustup_init: &str = "rustup-init";
    let process = Process::os();
    let local_path = format!("/tmp/{}", rustup_init);
    let dest = std::path::Path::new(&local_path);
    let _ = Rustup::new(&process).download(dest);
    
    unimplemented!("`install` is not yet implemented.")
}

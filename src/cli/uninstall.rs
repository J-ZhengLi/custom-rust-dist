//! Separated module to handle uninstallation in command line.

use crate::core::{
    parser::fingerprint::FingerPrint,
    uninstall::{UninstallConfiguration, Uninstallation},
};

use super::{common, GlobalOpt, ManagerSubcommands};

use anyhow::Result;

/// Execute `uninstall` command.
pub(super) fn execute(subcommand: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Uninstall { remove_self } = subcommand else {
        return Ok(false);
    };

    // Ask confirmation
    // TODO: format an installed list instead

    let config = UninstallConfiguration;
    let fingerprint = FingerPrint::load_fingerprint(&config.install_dir()?);
    let installed = fingerprint.print_installation();

    let prompt = if *remove_self {
        t!(
            "uninstall_all_confirmation",
            vendor = t!("vendor"),
            list = installed
        )
    } else {
        t!("uninstall_confirmation", list = installed)
    };
    if !common::confirm(prompt, false)? {
        return Ok(true);
    }

    // We need to check if the installation directory exists and legal while fisrt use it.
    let install_dir = config.install_dir()?;
    // remove all tools.
    config.remove_tools(fingerprint, &install_dir)?;
    // remove all the environments.
    config.remove_rustup_env_vars(&install_dir)?;
    // TODO: remove manager.
    if *remove_self {
        config.remove_self(&install_dir)?;
        // TODO: fix core::os::unix::remove_from_path()
        // Rmove the `<InstallDir>` which is added for manager.
        crate::core::os::remove_from_path(&install_dir)?;
    }

    Ok(true)
}

use super::{GlobalOpt, InstallCommand, Subcommands};
use crate::{defaults, mini_rustup, steps, utils};
use crate::parser::Configuration;

use anyhow::Result;

/// Processing `config` subcommand if it exist, otherwise this won't do anything.
pub(super) fn process(subcommand: &Subcommands, opt: GlobalOpt) -> Result<()> {
    let Subcommands::Install {
        commands: Some(install_commands),
    } = subcommand else { return Ok(()) };

    // gather information about settings and installations
    let mut config = steps::load_config().unwrap_or_default();
    let triple = mini_rustup::target_triple();

    match install_commands {
        InstallCommand::Rustup { version } => {
            download_and_install_rustup(version.as_deref(), &config, &triple)?;
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

fn download_and_install_rustup(version: Option<&str>, config: &Configuration, triple: &str) -> Result<()> {
    #[cfg(windows)]
    let rustup_init_bin = format!("rustup-init.exe");
    #[cfg(not(windows))]
    let rustup_init_bin = "rustup-init".to_string();

    let server_root = config
        .settings
        .rustup_update_root
        .as_ref()
        .map(|u| u.as_str())
        .unwrap_or(defaults::RUSTUP_UPDATE_ROOT);
    let rustup_url_string = if let Some(ver) = version {
        format!("{server_root}/archive/{ver}/{triple}/{rustup_init_bin}")
    } else {
        format!("{server_root}/dist/{triple}/{rustup_init_bin}")
    };

    let rustup_url = utils::parse_url(&rustup_url_string)?;
    let temp_path = tempfile::Builder::new()
        .prefix(crate::APPNAME)
        .tempdir()?;
    let installer_dest = temp_path.path().join(&rustup_init_bin);

    // Download rustup-init
    mini_rustup::utils::download_file(&rustup_url, &installer_dest, None)?;
    mini_rustup::utils::make_executable(&installer_dest)?;

    // TODO: run rustup-init

    Ok(())
}

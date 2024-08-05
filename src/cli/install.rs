//! Separated module to handle installation related behaviors in command line.

use std::path::PathBuf;

use crate::{
    core::{manifest::ToolsetManifest, InstallConfiguration, Installation, TomlParser},
    rustup::Rustup,
    utils,
};

use super::{GlobalOpt, Subcommands};

use anyhow::Result;

/// Execute `install` command.
pub(super) fn execute(subcommand: &Subcommands, _opt: GlobalOpt) -> Result<()> {
    let Subcommands::Install {
        prefix,
        registry_url,
        registry_name,
        rustup_dist_server,
        rustup_update_root,
    } = subcommand
    else {
        return Ok(());
    };

    let cargo_registry = registry_url
        .as_ref()
        .map(|u| (registry_name.clone(), u.clone()));
    let install_dir = prefix
        .clone()
        .unwrap_or_else(utils::home_dir)
        .join(env!("CARGO_PKG_NAME"));

    let config = InstallConfiguration::new(install_dir)
        .cargo_registry(cargo_registry)
        .rustup_dist_server(rustup_dist_server.to_owned())
        .rustup_update_root(rustup_update_root.to_owned());
    config.init(false)?;
    config.config_rustup_env_vars()?;
    config.config_cargo()?;

    // TODO: Download manifest form remote server.
    let manifest = ToolsetManifest::load(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("toolset_manifest.toml"),
    )?;

    // This step taking cares of requirements, such as `MSVC`.
    // Also third-party app such as `VS Code`.
    config.install_tools(&manifest)?;

    Rustup::init().download_toolchain(&config, &manifest)?;

    // TODO: install third-party tools via cargo that got installed by rustup

    unimplemented!("`install` is not fully yet implemented.")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::InstallConfiguration;
    use crate::{
        core::{manifest::ToolsetManifest, Installation, TomlParser},
        utils,
    };

    #[test]
    fn dry_run() {
        let config = InstallConfiguration::default();
        let _manifest = ToolsetManifest::from_str(
            &utils::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/toolset_manifest.toml"),
            )
            .unwrap(),
        )
        .unwrap();

        config.init(true).unwrap();
    }
}

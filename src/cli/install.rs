//! Separated module to handle installation related behaviors in command line.

use crate::core::install::{
    default_rustup_dist_server, default_rustup_update_root, EnvConfig, InstallConfiguration,
};
use crate::core::try_it;
use crate::manifest::baked_in_manifest;
use crate::utils;

use super::Installer;

use anyhow::Result;

/// Perform installer actions.
///
/// This will setup the environment and install everything user selected components.
pub(super) fn execute_installer(installer: &Installer) -> Result<()> {
    let Installer {
        prefix,
        registry_url,
        registry_name,
        rustup_dist_server,
        rustup_update_root,
        ..
    } = installer;

    let cargo_registry = registry_url
        .as_ref()
        .map(|u| (registry_name.clone(), u.clone()));
    let install_dir = prefix
        .clone()
        .unwrap_or_else(utils::home_dir)
        .join(env!("CARGO_PKG_NAME"));

    // TODO: Download manifest form remote server for online build
    let mut manifest = baked_in_manifest()?;
    manifest.adjust_paths()?;

    let mut config = InstallConfiguration::init(&install_dir, false)?
        .cargo_registry(cargo_registry)
        .rustup_dist_server(
            rustup_dist_server
                .clone()
                .unwrap_or_else(|| default_rustup_dist_server().clone()),
        )
        .rustup_update_root(
            rustup_update_root
                .clone()
                .unwrap_or_else(|| default_rustup_update_root().clone()),
        );
    config.config_env_vars(&manifest)?;
    config.config_cargo()?;

    // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
    config.install_tools(&manifest)?;
    config.install_rust(&manifest)?;
    // install third-party tools via cargo that got installed by rustup
    config.cargo_install(&manifest)?;

    println!("{}", t!("install_finish_info"));
    try_it::try_it(Some(&install_dir))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::InstallConfiguration;
    use crate::{core::parser::TomlParser, manifest::ToolsetManifest, utils};

    #[test]
    fn dry_run() {
        let mut cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cache_dir.push("tests");
        cache_dir.push("cache");

        std::fs::create_dir_all(&cache_dir).unwrap();

        let install_root = tempfile::Builder::new().tempdir_in(&cache_dir).unwrap();
        let _config = InstallConfiguration::init(install_root.path(), true).unwrap();
        let _manifest = ToolsetManifest::from_str(
            &utils::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/toolset_manifest.toml"),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

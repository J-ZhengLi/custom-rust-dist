//! Separated module to handle installation related behaviors in command line.

use crate::core::install::{
    default_rustup_dist_server, default_rustup_update_root, InstallConfiguration,
};
use crate::core::parser::manifest::ToolsetManifest;
use crate::core::parser::TomlParser;
use crate::core::{try_it, EnvConfig};
use crate::utils;

use super::{GlobalOpt, Subcommands};

use anyhow::Result;
use tempfile::TempDir;

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

    let mut config = InstallConfiguration::init(install_dir, false)?
        .cargo_registry(cargo_registry)
        .rustup_dist_server(
            rustup_dist_server
                .as_ref()
                .unwrap_or_else(|| default_rustup_dist_server()),
        )
        .rustup_update_root(
            rustup_update_root
                .as_ref()
                .unwrap_or_else(|| default_rustup_update_root()),
        );
    config.config_rustup_env_vars()?;
    config.config_cargo()?;

    // TODO: Download manifest form remote server for online build

    let (mut manifest, pkgs_root) = manifest_with_offline_packages(&config)?;
    manifest.adjust_paths()?;

    // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
    config.install_tools(&manifest)?;
    config.install_rust(&manifest)?;
    // install third-party tools via cargo that got installed by rustup
    config.cargo_install(&manifest)?;

    // Explicitly drop cache to remove the bundled packages.
    drop(pkgs_root);

    println!(
        "Rust is installed, \
        this setup will soon create an example project at current directory for you to try Rust!"
    );
    try_it::try_it(None)?;

    Ok(())
}

/// Try to include offline packages that were bundled in the source,
/// and return the adjusted manifest along with a `TempDir`.
///
/// The returned `TempDir` is meant to keep the package path alive.
///
/// Note that the bundled package sources will replace the one in the original manifest toml.
#[cfg(feature = "offline")]
pub(crate) fn manifest_with_offline_packages(
    config: &InstallConfiguration,
) -> Result<(ToolsetManifest, Option<TempDir>)> {
    let mut orig_manifest =
        ToolsetManifest::from_str(include_str!("../../resources/toolset_manifest.toml"))?;
    let Some(tools_map) = orig_manifest.current_target_tools_mut() else {
        return Ok((orig_manifest, None));
    };

    let temp_dir = config.create_temp_dir("offline_packages")?;
    let offline_pkgs = crate::core::offline_packages::OfflinePackages::load();

    for (key, val) in offline_pkgs.0 {
        // make sure no redundent packages in the source
        let tool_info = tools_map.get_mut(key).unwrap_or_else(|| {
            panic!(
                "Internal Error: Redundent package '{}' in the resource directory.",
                val.filename
            )
        });
        // Then we need to write the pkg content to local file.
        let dest = temp_dir.path().join(val.filename);
        utils::write_bytes(&dest, val.value, false)?;
        // Lastly, we overwrite the `ToolInfo` in the manifest
        tool_info.convert_to_path(dest);
    }

    Ok((orig_manifest, Some(temp_dir)))
}

#[cfg(not(feature = "offline"))]
pub(crate) fn manifest_with_offline_packages(
    _config: &InstallConfiguration,
) -> Result<(ToolsetManifest, Option<TempDir>)> {
    ToolsetManifest::from_str(include_str!("../../resources/toolset_manifest.toml"))
        .map(|m| (m, None))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{InstallConfiguration, TomlParser, ToolsetManifest};
    use crate::utils;

    #[test]
    fn dry_run() {
        let mut cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cache_dir.push("tests");
        cache_dir.push("cache");

        std::fs::create_dir_all(&cache_dir).unwrap();

        let install_root = tempfile::Builder::new().tempdir_in(&cache_dir).unwrap();
        let _config = InstallConfiguration::init(install_root.path().to_path_buf(), true).unwrap();
        let _manifest = ToolsetManifest::from_str(
            &utils::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/toolset_manifest.toml"),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

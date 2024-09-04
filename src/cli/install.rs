//! Separated module to handle installation related behaviors in command line.

use std::path::PathBuf;

use crate::cli::common::question_str;
use crate::core::install::{
    default_rustup_dist_server, default_rustup_update_root, EnvConfig, InstallConfiguration,
};
use crate::core::try_it;
use crate::manifest::{baked_in_manifest, ToolMap};
use crate::{default_install_dir, get_component_list_from_manifest, utils, Component};

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

    // TODO: Download manifest form remote server for online build
    let mut manifest = baked_in_manifest()?;
    manifest.adjust_paths()?;

    let component_list = get_component_list_from_manifest(&manifest)?;
    let user_opt = CustomInstallOpt::collect_from_user(component_list)?;

    todo!("show confirmation and allow the user to go back and modify their choices");

    let cargo_registry = registry_url
        .as_ref()
        .map(|u| (registry_name.clone(), u.clone()));
    let install_dir = user_opt
        .prefix
        .or(prefix.clone())
        .unwrap_or_else(utils::home_dir)
        .join(env!("CARGO_PKG_NAME"));

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

    let mut dummy_prog = utils::MultiThreadProgress::default();
    // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
    config.install_tools_with_progress(&manifest, &user_opt.toolset, &mut dummy_prog)?;
    config.install_rust_with_progress(
        &manifest,
        &user_opt.toolchain_components,
        &mut dummy_prog,
    )?;
    // install third-party tools via cargo that got installed by rustup
    config.cargo_install_with_progress(&user_opt.toolset, &mut dummy_prog)?;

    println!("{}", t!("install_finish_info"));
    try_it::try_it(Some(&install_dir))?;

    Ok(())
}

/// Contains customized install options that will be collected from user input.
///
/// Check [`collect_from_user`](CustomInstallOpt::collect_from_user) for more detail.
#[derive(Debug, Default)]
struct CustomInstallOpt {
    prefix: Option<PathBuf>,
    toolchain_components: Vec<String>,
    toolset: ToolMap,
}

impl CustomInstallOpt {
    /// Asking various questions and collect user input from CLI,
    /// then return user specified installation options.
    fn collect_from_user(component_choices: Vec<Component>) -> Result<Self> {
        // This clear the console screen while also move the cursor to top left
        const CLEAR_SCREEN_SPELL: &str = "\x1B[2J\x1B[1:1H";

        let mut custom_opts = Self::default();

        println!(
            "{CLEAR_SCREEN_SPELL}\n\n{}",
            t!("welcome", vendor = t!("vendor"))
        );
        println!("{}\n", t!("custom_install_help"));

        let default_install_dir = default_install_dir();
        let install_dir: PathBuf = question_str(
            t!("question_install_dir"),
            None,
            utils::path_to_str(&default_install_dir)?,
        )?
        .into();

        let component_list = component_choices
            .iter()
            .map(|c| format!("{}. {}", c.id, &c.name))
            .collect::<Vec<_>>()
            .join("\n");
        let default_choices = component_choices
            .iter()
            .filter_map(|c| c.required.then_some(c.id.to_string()))
            .collect::<Vec<_>>()
            .join(" ");
        let components = question_str(
            t!("question_component_choice"),
            Some(component_list),
            default_choices,
        )?;

        Ok(custom_opts)
    }
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

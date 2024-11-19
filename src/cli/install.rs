//! Separated module to handle installation related behaviors in command line.

use std::collections::HashSet;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::common::{self, Confirm};
use crate::components::Component;
use crate::core::install::{
    default_rustup_dist_server, default_rustup_update_root, InstallConfiguration,
    DEFAULT_CARGO_REGISTRY,
};
use crate::core::try_it;
use crate::toolset_manifest::get_toolset_manifest;
use crate::{components, default_install_dir, utils};

use super::common::{
    question_single_choice, ComponentChoices, ComponentDecoration, ComponentListBuilder,
};
use super::{Installer, ManagerSubcommands};

use anyhow::{bail, Result};
use log::warn;

/// Perform installer actions.
///
/// This will setup the environment and install user selected components.
pub(super) fn execute_installer(installer: &Installer) -> Result<()> {
    let Installer {
        prefix,
        registry_url,
        registry_name,
        rustup_dist_server,
        rustup_update_root,
        manifest: manifest_src,
        ..
    } = installer;

    if matches!(&prefix, Some(p) if utils::is_root_dir(p)) {
        bail!(t!("notify_root_dir"));
    }

    let manifest_url = manifest_src.as_ref().map(|s| s.to_url()).transpose()?;
    let mut manifest = get_toolset_manifest(manifest_url.as_ref())?;
    manifest.adjust_paths()?;

    let component_list = components::get_component_list_from_manifest(&manifest, None)?;
    let user_opt = CustomInstallOpt::collect_from_user(
        prefix.as_deref().unwrap_or(&default_install_dir()),
        component_list,
    )?;

    let (registry_name, registry_value) = registry_url
        .as_deref()
        .map(|u| (registry_name.as_str(), u))
        .unwrap_or(DEFAULT_CARGO_REGISTRY);
    let install_dir = user_opt.prefix;

    InstallConfiguration::init(&install_dir, None, &manifest, false)?
        .cargo_registry(registry_name, registry_value)
        .rustup_dist_server(
            rustup_dist_server
                .clone()
                .unwrap_or_else(|| default_rustup_dist_server().clone()),
        )
        .rustup_update_root(
            rustup_update_root
                .clone()
                .unwrap_or_else(|| default_rustup_update_root().clone()),
        )
        .install(user_opt.components)?;

    println!("\n{}\n", t!("install_finish_info"));

    if common::confirm(t!("question_try_demo"), true)? {
        try_it::try_it(Some(&install_dir))?;
    }

    #[cfg(unix)]
    if let Some(cmd) = crate::core::os::unix::source_command() {
        println!("\n{}", t!("linux_source_hint", cmd = cmd));
    }
    #[cfg(windows)]
    common::pause()?;

    Ok(())
}

/// Contains customized install options that will be collected from user input.
///
/// Check [`collect_from_user`](CustomInstallOpt::collect_from_user) for more detail.
#[derive(Debug, Default)]
struct CustomInstallOpt {
    prefix: PathBuf,
    components: Vec<Component>,
}

impl CustomInstallOpt {
    /// Asking various questions and collect user input from CLI,
    /// then return user specified installation options.
    fn collect_from_user(prefix: &Path, components: Vec<Component>) -> Result<Self> {
        // This clear the console screen while also move the cursor to top left
        #[cfg(not(windows))]
        const CLEAR_SCREEN_SPELL: &str = "\x1B[2J\x1B[1:1H";
        #[cfg(windows)]
        const CLEAR_SCREEN_SPELL: &str = "";

        let mut stdout = io::stdout();
        writeln!(
            &mut stdout,
            "{CLEAR_SCREEN_SPELL}\n\n{}",
            t!("welcome", product = t!("product"))
        )?;
        writeln!(&mut stdout, "\n\n{}", t!("what_this_is"))?;
        writeln!(&mut stdout, "{}\n", t!("custom_install_help"))?;

        // initialize these with default value, but they could be altered by the user
        let mut install_dir = utils::path_to_str(prefix)?.to_string();

        loop {
            if let Some(dir_input) = read_install_dir_input(&install_dir)? {
                install_dir = dir_input;
            } else {
                continue;
            }

            let choices = read_component_selections(&components)?;

            show_confirmation(&install_dir, &choices)?;

            match common::confirm_install()? {
                Confirm::Yes => {
                    return Ok(Self {
                        prefix: install_dir.into(),
                        components: choices.keys().map(|c| (*c).to_owned()).collect(),
                    });
                }
                Confirm::No => (),
                Confirm::Abort => std::process::exit(0),
            }
        }
    }
}

fn read_install_dir_input(default: &str) -> Result<Option<String>> {
    let dir_input = common::question_str(t!("question_install_dir"), None, default)?;
    // verify path input before proceeding
    if utils::is_root_dir(&dir_input) {
        warn!("{}", t!("notify_root_dir"));
        Ok(None)
    } else {
        Ok(Some(dir_input))
    }
}

/// Read user response of what set of components they want to install.
///
/// Currently, there's only three options:
/// 1. default
/// 2. everything
/// 3. custom
fn read_component_selections<'a>(components: &'a [Component]) -> Result<ComponentChoices<'a>> {
    let profile_choices = &[
        t!("install_default"),
        t!("install_everything"),
        t!("install_custom"),
    ];
    let default_set = || -> ComponentChoices<'a> {
        components
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| {
                if !c.installed && !c.optional {
                    Some((c, idx))
                } else {
                    None
                }
            })
            .collect()
    };
    let choice = question_single_choice(t!("question_components_profile"), profile_choices, "1")?;
    let selection = match choice {
        // Default set
        1 => default_set(),
        // Full set, but exclude installed components
        2 => components
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| if !c.installed { Some((c, idx)) } else { None })
            .collect(),
        // Customized set
        3 => {
            let list_of_comps = ComponentListBuilder::new(components)
                .show_desc(true)
                .decorate(ComponentDecoration::InstalledOrRequired)
                .build();
            let default_ids = default_set()
                .values()
                .map(|idx| (idx + 1).to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let choices = common::question_multi_choices(
                t!("select_components_to_install"),
                &list_of_comps,
                &default_ids,
            )?;
            // convert input vec to set for faster lookup
            // Note: user input index are started from 1.
            let index_set: HashSet<usize> = choices.into_iter().collect();

            // convert the input indexes to `ComponentChoices`,
            // and we also need to add the `required` tools even if the user didn't choose it.
            components
                .iter()
                .enumerate()
                .filter_map(|(idx, c)| {
                    if c.required || index_set.contains(&(idx + 1)) {
                        Some((c, idx))
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => unreachable!("out-of-range input should already be caught"),
    };

    Ok(selection)
}

fn show_confirmation(install_dir: &str, choices: &ComponentChoices<'_>) -> Result<()> {
    let mut stdout = std::io::stdout();

    writeln!(&mut stdout, "\n{}\n", t!("current_install_option"))?;
    writeln!(&mut stdout, "{}:\n\t{install_dir}", t!("install_dir"))?;
    writeln!(&mut stdout, "\n{}:", t!("selected_components"))?;
    let list_of_comp = ComponentListBuilder::new(choices.keys().copied())
        .decorate(ComponentDecoration::Confirmation)
        .build()
        .join("\n");
    for line in list_of_comp.lines() {
        writeln!(&mut stdout, "\t{line}")?;
    }

    Ok(())
}

pub(super) fn execute_manager(manager: &ManagerSubcommands) -> Result<bool> {
    let ManagerSubcommands::Install { version } = manager else {
        return Ok(false);
    };

    todo!("install dist with version '{version}'");
}

//! Separated module to handle installation related behaviors in command line.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::common::{self, Confirm};
use crate::components::Component;
use crate::core::install::{
    default_rustup_dist_server, default_rustup_update_root, InstallConfiguration,
    DEFAULT_CARGO_REGISTRY,
};
use crate::core::try_it;
use crate::toolset_manifest::{get_toolset_manifest, ToolMap};
use crate::{components, default_install_dir, utils};

use super::{Installer, ManagerSubcommands};

use anyhow::{bail, Result};
use indexmap::IndexSet;
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

    let component_list = components::get_component_list_from_manifest(&manifest, false)?;
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
        .install(user_opt.toolchain_components, user_opt.toolset)?;

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
    toolchain_components: Vec<String>,
    toolset: ToolMap,
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

        let default_install_dir = utils::path_to_str(prefix)?.to_string();
        let component_list = displayed_component_list(components.iter(), false);
        let mut default_choices = vec![];
        let mut enforced_choices = vec![];
        for (idx, c) in components.iter().enumerate() {
            if !c.installed {
                if c.required {
                    enforced_choices.push(idx);
                }
                if !c.optional {
                    default_choices.push(idx);
                }
            }
        }
        let default_choices_str = default_choices
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let mut install_dir = String::new();
        let mut raw_choices: Option<String> = None;

        loop {
            // NB: Do NOT change `Some(res?)` to `result.ok()` in this case here,
            // we want to throw error if the input cannot be read.
            install_dir = common::question_str(
                t!("question_install_dir"),
                None,
                if install_dir.is_empty() {
                    &default_install_dir
                } else {
                    &install_dir
                },
            )?;
            // verify path input before proceeding
            if utils::is_root_dir(&install_dir) {
                warn!("{}", t!("notify_root_dir"));
                continue;
            }

            raw_choices = Some(common::question_str_with_retry(
                t!("question_component_choice"),
                Some(&component_list),
                raw_choices.as_deref().unwrap_or(&default_choices_str),
                |input| -> bool {
                    if input
                        .split_whitespace()
                        .all(|s| matches!(s.parse::<usize>(), Ok(idx) if idx < components.len()))
                    {
                        true
                    } else {
                        warn!(
                            "{}",
                            t!(
                                "invalid_input",
                                actual = input,
                                expect = t!("list_of_ids", bound = components.len())
                            )
                        );
                        false
                    }
                },
            )?);
            let choices = choice_string_to_choices(
                raw_choices.as_deref().unwrap(),
                &components,
                &enforced_choices,
            );

            show_confirmation(&install_dir, &choices)?;

            match common::confirm_install()? {
                Confirm::Yes => {
                    let mut toolchain_components = vec![];
                    let mut toolset = ToolMap::default();

                    for component in choices
                        .iter()
                        // Skip `Rust minimal toolchain`
                        .skip(1)
                    {
                        if component.is_toolchain_component {
                            toolchain_components.push(component.name.clone());
                        } else if let Some(installer) = &component.tool_installer {
                            toolset.insert(component.name.clone(), installer.to_owned());
                        }
                    }

                    return Ok(Self {
                        prefix: install_dir.into(),
                        toolchain_components,
                        toolset,
                    });
                }
                Confirm::No => (),
                Confirm::Abort => std::process::exit(0),
            }
        }
    }
}

// Convert the choice input such as `1 2 3` to actual selected set of components
fn choice_string_to_choices<'a>(
    raw_choices: &str,
    components: &'a [Component],
    enforced: &[usize],
) -> Vec<&'a Component> {
    let user_seleted = raw_choices
        .split_whitespace()
        // The choices should already be valid at this point, but use filter_map just in case.
        .filter_map(|s| s.parse::<usize>().ok());
    // Use `IndexSet` for easy dedup.
    let idx_set = enforced
        .iter()
        .copied()
        .chain(user_seleted)
        .collect::<IndexSet<_>>();
    idx_set
        .iter()
        .filter_map(|idx| components.get(*idx))
        .collect()
}

fn show_confirmation(install_dir: &str, choices: &[&Component]) -> Result<()> {
    let mut stdout = std::io::stdout();

    writeln!(&mut stdout, "\n{}\n", t!("current_install_option"))?;
    writeln!(&mut stdout, "{}:\n\t{install_dir}", t!("install_dir"))?;
    writeln!(&mut stdout, "\n{}:", t!("selected_components"))?;
    for line in displayed_component_list(choices.iter().copied(), true).lines() {
        writeln!(&mut stdout, "\t{line}")?;
    }

    Ok(())
}

fn displayed_component_list<'a, I: Iterator<Item = &'a Component>>(
    components: I,
    is_confirm: bool,
) -> String {
    components
        .enumerate()
        .map(|(idx, c)| {
            format!(
                "{}{}{}{}",
                if is_confirm {
                    "".to_string()
                } else {
                    format!("{idx}) ")
                },
                &c.name,
                if c.installed {
                    if is_confirm {
                        format!(" ({})", t!("reinstall"))
                    } else {
                        format!(" ({})", t!("installed"))
                    }
                } else if c.required {
                    format!(" ({})", t!("required"))
                } else {
                    "".to_string()
                },
                if is_confirm || c.desc.is_empty() {
                    "".to_string()
                } else {
                    format!("\n\t{}: {}", t!("description"), &c.desc)
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn execute_manager(manager: &ManagerSubcommands) -> Result<bool> {
    let ManagerSubcommands::Install { version } = manager else {
        return Ok(false);
    };

    todo!("install dist with version '{version}'");
}

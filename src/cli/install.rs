//! Separated module to handle installation related behaviors in command line.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cli::common::{self, Confirm};
use crate::core::install::{
    default_cargo_registry, default_rustup_dist_server, default_rustup_update_root, EnvConfig,
    InstallConfiguration,
};
use crate::core::try_it;
use crate::manifest::{baked_in_manifest, ToolMap};
use crate::{default_install_dir, get_component_list_from_manifest, utils, Component};

use super::{GlobalOpt, Installer, ManagerSubcommands};

use anyhow::Result;
use indexmap::IndexSet;

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
        ..
    } = installer;

    // TODO: Download manifest form remote server for online build
    let mut manifest = baked_in_manifest()?;
    manifest.adjust_paths()?;

    let component_list = get_component_list_from_manifest(&manifest)?;
    let user_opt = CustomInstallOpt::collect_from_user(
        prefix.as_deref().unwrap_or(&default_install_dir()),
        component_list,
    )?;

    let cargo_registry = registry_url
        .as_ref()
        .map(|u| (registry_name.clone(), u.clone()))
        .or(default_cargo_registry());
    let install_dir = user_opt.prefix;

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

    println!("\n{}\n", t!("install_finish_info"));

    if common::confirm(t!("question_try_demo"), true)? {
        try_it::try_it(Some(&install_dir))?;
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

        println!(
            "{CLEAR_SCREEN_SPELL}\n\n{}",
            t!("welcome", vendor = t!("vendor"))
        );
        println!("{}\n", t!("custom_install_help"));

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
                        println!(
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

pub(super) fn execute_manager(manager: &ManagerSubcommands, _opt: GlobalOpt) -> Result<bool> {
    let ManagerSubcommands::Install { version } = manager else {
        return Ok(false);
    };

    todo!("install dist with version '{version}'");
}

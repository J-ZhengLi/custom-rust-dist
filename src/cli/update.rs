use anyhow::{anyhow, Result};
use log::info;
use std::collections::HashSet;
use std::path::Path;
use url::Url;

use crate::components::{get_component_list_from_manifest, Component};
use crate::core::toolkit::Toolkit;
use crate::core::update::UpdateOpt;
use crate::toolkit::latest_installable_toolkit;
use crate::toolset_manifest::get_toolset_manifest;
use crate::InstallConfiguration;

use super::common::{ComponentChoices, ComponentDecoration, ComponentListBuilder, VersionDiffMap};
use super::{common, ManagerSubcommands};

pub(super) fn execute(cmd: &ManagerSubcommands) -> Result<bool> {
    let ManagerSubcommands::Update {
        toolkit_only,
        manager_only,
    } = cmd
    else {
        return Ok(false);
    };

    let update_opt = UpdateOpt;
    if !manager_only {
        update_opt.update_toolkit(update_toolkit_)?;
    }
    if !toolkit_only {
        update_opt.self_update()?;
    }

    Ok(true)
}

fn update_toolkit_(install_dir: &Path) -> Result<()> {
    let Some(installed) = Toolkit::installed()? else {
        info!("{}", t!("no_toolkit_installed"));
        return Ok(());
    };

    // get possible update
    let Some(latest_toolkit) = latest_installable_toolkit()? else {
        return Ok(());
    };

    // load the latest manifest
    let manifest_url = latest_toolkit
        .manifest_url
        .as_deref()
        .and_then(|s| Url::parse(s).ok())
        .ok_or_else(|| {
            anyhow!(
                "invalid dist manifest downloaded from server: \
            must contains a valid `manifest_url`"
            )
        })?;
    let manifest = get_toolset_manifest(Some(&manifest_url))?;
    let new_components = get_component_list_from_manifest(&manifest, None)?;

    // notify user that we will install the latest update to replace their current installation
    info!(
        "{}",
        t!(
            "pre_update_note",
            target_version = latest_toolkit.version,
            current_version = installed.version
        )
    );

    let updater = ComponentsUpdater::new(&installed.components, &new_components);
    // let user choose if they want to update installed component only, or want to select more components to install
    if let UpdateOption::Yes(components) = updater.get_user_choices()? {
        // install update for selected components
        let config = InstallConfiguration::init(install_dir, None, &manifest, true)?;
        config.update(components.into_keys().cloned().collect())
    } else {
        Ok(())
    }
}

enum UpdateOption<'c> {
    Yes(ComponentChoices<'c>),
    NoUpdate,
}

struct ComponentsUpdater<'c> {
    target: &'c [Component],
    version_diff: VersionDiffMap<'c>,
}

impl<'c> ComponentsUpdater<'c> {
    fn new(installed: &'c [Component], target: &'c [Component]) -> Self {
        let version_diff = target
            .iter()
            .map(|c| {
                let installed_version = installed
                    .iter()
                    .find_map(|ic| (ic.name == c.name).then_some(ic.version.as_deref()))
                    .flatten();
                (c.name.as_str(), (installed_version, c.version.as_deref()))
            })
            .collect();
        Self {
            target,
            version_diff,
        }
    }

    // We are only pre-selecting the components for update if the component exists in both lists
    // and having different version.
    // Note that we don't check if the new version is actually "newer" than the installed version,
    // it is intended to prevent a scenario where a component needs to be rollback in a new toolkit.
    fn component_names_with_diff_version(&self) -> HashSet<&'c str> {
        self.version_diff
            .iter()
            .filter_map(|(name, (from, to))| (from != to).then_some(*name))
            .collect()
    }

    fn get_user_choices(&self) -> Result<UpdateOption<'c>> {
        let default = self.default_component_choices();
        self.handle_update_interaction_(default)
    }

    // Default component set contains components that are:
    // 1. installed
    // 2. have different versions
    fn default_component_choices(&self) -> ComponentChoices<'c> {
        let diff_ver_comps = self.component_names_with_diff_version();

        self.target
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| diff_ver_comps.contains(c.name.as_str()).then_some((c, idx)))
            .collect()
    }

    fn custom_component_choices(&self, orig: ComponentChoices<'c>) -> Result<ComponentChoices<'c>> {
        let choices = ComponentListBuilder::new(self.target)
            .decorate(ComponentDecoration::VersionDiff(&self.version_diff))
            .show_desc(true)
            .build();
        let defult_choices = orig
            .values()
            .map(|idx| (idx + 1).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let input = common::question_multi_choices(
            t!("select_components_to_update"),
            &choices,
            defult_choices,
        )?;

        // convert input vec to set for faster lookup
        // Note: user input index are started from 1.
        let index_set: HashSet<usize> = input.into_iter().collect();

        // convert the input indexes to `ComponentChoices`
        Ok(self
            .target
            .iter()
            .enumerate()
            .filter_map(|(idx, c)| index_set.contains(&(idx + 1)).then_some((c, idx)))
            .collect())
    }

    // recursively ask for user input
    fn handle_update_interaction_(&self, list: ComponentChoices<'c>) -> Result<UpdateOption<'c>> {
        let choices = vec![t!("continue"), t!("customize"), t!("cancel")];
        let comp_list = ComponentListBuilder::new(list.keys().copied())
            .decorate(ComponentDecoration::VersionDiff(&self.version_diff))
            .build()
            .join("\n");
        let choice = common::question_single_choice(
            t!("pre_update_confirmation", list = comp_list),
            &choices,
            1,
        )?;
        match choice {
            1 => Ok(UpdateOption::Yes(list)),
            2 => {
                let custom_choices = self.custom_component_choices(list)?;
                self.handle_update_interaction_(custom_choices)
            }
            3 => Ok(UpdateOption::NoUpdate),
            _ => {
                unreachable!("input function should already catches out of range input '{choice}'")
            }
        }
    }
}

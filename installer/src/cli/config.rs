use std::collections::HashMap;
use std::path::Path;

use super::{Subcommands, common};
use crate::applog::*;
use crate::parser::{load_config, CargoRegistry, Settings};
use crate::steps;

use anyhow::{bail, Result};

/// Processing `config` subcommand if it exist, otherwise this won't do anything.
pub(super) fn process(subcommand: &Subcommands, verbose: bool) -> Result<()> {
    let Subcommands::Config {
        list,
        cargo_home,
        rustup_home,
        rustup_dist_server,
        rustup_update_root,
        proxy,
        no_proxy,
        git_fetch_with_cli,
        check_revoke,
        registry,
        input
    } = subcommand else { return Ok(()) };

    let mut existing_config = steps::load_config()
        .ok()
        .filter(|c| !c.settings.is_default());
    let create_new = existing_config.is_none();
    let existing_settings = &mut existing_config.unwrap_or_default().settings;

    if *list {
        list_config(existing_settings, verbose);
        return Ok(());
    }
    if let Some(cfg_path_str) = input {
        import_config(cfg_path_str, existing_settings, create_new)?;
        return Ok(());
    }

    Ok(())
}

/// Format program settings ([`Settings`]), and prints them.
fn list_config(settings: &Settings, verbose: bool) {
    macro_rules! print_opt {
        ($s:tt, $def:tt, $($opt:tt)*) => {{
            println!(
                "{}: {}",
                $s,
                settings.$($opt)*.as_ref().map(|t| t.to_string())
                    .unwrap_or_else(|| $def.to_string())
            );
        }};
    }

    println!(
        "\n\
        list of configurations\n\
        ----------------------"
    );
    print_opt!("cargo-home", "[default]", cargo_home);
    print_opt!("rustup-home", "[default]", rustup_home);
    print_opt!("rustup-dist-server", "[default]", rustup_dist_server);
    print_opt!("rustup-update-root", "[default]", rustup_update_root);
    print_opt!("proxy", "N/A", proxy);
    print_opt!("no_proxy", "N/A", no_proxy);
    print_opt!(
        "git-fetch-with-cli",
        "[default]",
        cargo.as_ref().map(|c| c.git_fetch_with_cli)
    );
    print_opt!(
        "check-revoke",
        "[default]",
        cargo.as_ref().map(|c| c.check_revoke)
    );
    print_opt!(
        "default-registry",
        "[default]",
        cargo.as_ref().and_then(|c| c.default_registry.as_ref())
    );
    // special treatment for cargo registries, which is a hashmap thus do not have `to_string` method
    let registries = settings.cargo.as_ref().map(|c| &c.registries)
        .map(|hm| {
            hm.iter()
                .map(|(k, v)| format!("'{k} - ({})'", &v.index))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_else(|| "default".to_string());
    println!("registries: [{registries}]");
    println!("----------------------\n");
}

fn import_config(path_str: &str, existing: &mut Settings, create_new: bool) -> Result<()> {
    let cfg_path = Path::new(path_str);
    if !cfg_path.is_file() {
        bail!(
            "unable to import configuration '{}': path is not a file",
            cfg_path.display()
        );
    }
    let importing_cfg = load_config(cfg_path)?;
    if !create_new {
        warn!("existing configuration detected");
        if !common::confirm("override?", false)? {
            return Ok(());
        }
    }
    if importing_cfg.installation.is_some() {
        info!("force skipping `[installation]` sections");
    }

    Ok(())
}

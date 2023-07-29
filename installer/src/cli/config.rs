use std::collections::HashMap;
use std::path::Path;

use super::{Subcommands, common};
use crate::applog::*;
use crate::parser::{load_config, CargoRegistry, Settings, Configuration};
use crate::steps::{self, update_config};

use anyhow::{bail, Result};

/// Processing `config` subcommand if it exist, otherwise this won't do anything.
pub(super) fn process(subcommand: &Subcommands, verbose: bool, yes: bool) -> Result<()> {
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

    let maybe_config = steps::load_config().ok();
    let create_new = maybe_config.is_none();
    let mut existing_config = maybe_config.unwrap_or_default();

    if *list {
        if create_new {
            warn!("no configuration file detected, showing default configuration instead");
        }
        list_config(&existing_config.settings, verbose);
        return Ok(());
    }
    if let Some(cfg_path_str) = input {
        import_config(cfg_path_str, &mut existing_config, create_new, yes)?;
        return Ok(());
    }

    Ok(())
}

/// Format program settings ([`Settings`]), and prints them.
fn list_config(settings: &Settings, verbose: bool) {
    // FIXME: hygine
    macro_rules! print_opt {
        ($s:literal, $f:expr) => {{
            println!("{}: {}", $s, $f);
        }};
        ($s:literal, $def:literal, $($opt:tt)*) => {{
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
    print_opt!("registries", registries_string(settings));
    println!("----------------------\n");
}

fn registries_string(settings: &Settings) -> String {
    settings.cargo.as_ref().map(|c| &c.registries)
        .map(|hm| {
            hm.iter()
                .map(|(k, v)| format!("'{k} - ({})'", &v.index))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_else(|| "default".to_string())
}

fn import_config(path_str: &str, existing: &mut Configuration, create_new: bool, yes: bool) -> Result<()> {
    let cfg_path = Path::new(path_str);
    if !cfg_path.is_file() {
        bail!(
            "unable to import configuration '{}': path is not a file",
            cfg_path.display()
        );
    }
    let importing_cfg = load_config(cfg_path)?;

    info!("configuration will be updated to:");
    println!();
    show_settings_diff(&existing.settings, &importing_cfg.settings);

    if !create_new {
        warn!("existing configuration detected");
        if !yes && !common::confirm("override? (y/n)", false)? {
            return Ok(());
        }
    }
    if importing_cfg.installation.is_some() {
        info!("force skipping `[installation]` sections");
    }

    existing.settings = importing_cfg.settings;
    update_config(existing)?;
    info!("configuration updated successfully.");

    Ok(())
}

fn show_settings_diff(old: &Settings, new: &Settings) {
    // FIXME: hygine
    macro_rules! diff {
        ($name:literal, $lhs:expr => $rhs:expr) => {{
            let rhs_str = $rhs;
            let rhs_colored = logger::color::ColoredStr::new()
                .content(&rhs_str)
                .color(logger::color::Color::Green)
                .bright()
                .build();
            println!("{}: {} -> {}", $name, $lhs, rhs_colored);
        }};
        ($name:literal, $def:literal, $($opt:tt)*) => {{
            let new_string = new.$($opt)*.as_ref().map(|t| t.to_string())
                .unwrap_or_else(|| $def.to_string());
            let new_colored_string = logger::color::ColoredStr::new()
                .content(&new_string)
                .color(logger::color::Color::Green)
                .bright()
                .build();
            println!(
                "{}: {} -> {}",
                $name,
                old.$($opt)*.as_ref().map(|t| t.to_string())
                    .unwrap_or_else(|| $def.to_string()),
                new_colored_string
            );
        }};
    }

    diff!("cargo-home", "[default]", cargo_home);
    diff!("rustup-home", "[default]", rustup_home);
    diff!("rustup-dist-server", "[default]", rustup_dist_server);
    diff!("rustup-update-root", "[default]", rustup_update_root);
    diff!("proxy", "[N/A]", proxy);
    diff!("no-proxy", "[N/A]", no_proxy);
    diff!("git-fetch-with-cli", "[default]", cargo.as_ref().map(|c| c.git_fetch_with_cli));
    diff!("check-revoke", "[default]", cargo.as_ref().map(|c| c.check_revoke));
    diff!("default-registries", "[default]", cargo.as_ref().and_then(|c| c.default_registry.as_ref()));
    diff!("registries", registries_string(old) => registries_string(new));
    println!();
}

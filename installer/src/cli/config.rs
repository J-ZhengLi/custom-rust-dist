use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, bail, Result};

use super::{common, Subcommands};
use crate::applog::*;
use crate::cli::{ConfigSubcommand, RegistryOpt};
use crate::parser::{load_config, CargoRegistry, Configuration, Settings};
use crate::steps::{self, update_config};

macro_rules! print_opt {
    ($s:literal, $f:expr) => {{
        println!("{}: {}", $s, $f);
    }};
    ($name:literal, $def:literal, $($opt:tt)*) => {{
        println!(
            "{}: {}",
            $name,
            $($opt)*.as_ref().map(|t| t.to_string())
                .unwrap_or_else(|| $def.to_string())
        );
    }};
}

macro_rules! diff {
    ($name:literal, $lhs:expr => $rhs:expr) => {{
        let lhs_str = $lhs;
        let rhs_str = $rhs;
        let rhs_colored = if lhs_str != rhs_str {
            logger::color::ColoredStr::new()
                .content(&rhs_str)
                .color(logger::color::Color::Green)
                .bright()
                .build()
        } else {
            rhs_str
        };
        println!("{}: {} -> {}", $name, lhs_str, rhs_colored);
    }};
    ($name:literal, $def:literal, $old:ident => $new:ident, $($opt:tt)*) => {{
        let old_string = $old.$($opt)*.as_ref().map(|t| t.to_string())
            .unwrap_or_else(|| $def.to_string());
        let new_string = $new.$($opt)*.as_ref().map(|t| t.to_string())
            .unwrap_or_else(|| $def.to_string());
        let new_colored_string = if old_string != new_string {
            logger::color::ColoredStr::new()
                .content(&new_string)
                .color(logger::color::Color::Green)
                .bright()
                .build()
        } else {
            new_string
        };
        println!(
            "{}: {} -> {}",
            $name,
            old_string,
            new_colored_string
        );
    }};
}

macro_rules! apply {
    ($from:ident, $to:expr) => {
        if $from.is_some() {
            $to = $from.clone();
        }
    };
    ($from:ident, $to:expr, $inner:ident) => {
        if $from.is_some() {
            let mut $inner = $to.unwrap_or_default();
            $inner.$from = $from.clone();
            $to = Some($inner);
        }
    };
}

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

    let mut temp_setts = Settings::default();
    // because `registry` is a seperated command, it will be checked seperatedly
    if let Some(ConfigSubcommand::Registry { opt: Some(reg_opt) }) = registry.as_ref().map(|cs| cs)
    {
        match reg_opt {
            RegistryOpt::Default {
                default: Some(default),
            } => {}
            RegistryOpt::Add {
                url: Some(url),
                name,
            } => {
                let name_fullback =
                    name.as_deref().or_else(|| url.host_str()).ok_or_else(|| {
                        anyhow!(
                            "fail to automatically resolve the registry name, \
                            try using `--name` to specify one"
                        )
                    })?;
            }
            RegistryOpt::Rm { name: Some(name) } => {}
            _ => (),
        }
    } else {
        // apply provided configs one by one
        apply!(cargo_home, temp_setts.cargo_home);
        apply!(rustup_home, temp_setts.rustup_home);
        apply!(rustup_dist_server, temp_setts.rustup_dist_server);
        apply!(rustup_update_root, temp_setts.rustup_update_root);
        apply!(proxy, temp_setts.proxy);
        apply!(no_proxy, temp_setts.no_proxy);
        apply!(git_fetch_with_cli, temp_setts.cargo, cargo_settings);
        apply!(check_revoke, temp_setts.cargo, cargo_settings);
    }

    Ok(())
}

/// Format program settings ([`Settings`]), and prints them.
fn list_config(settings: &Settings, verbose: bool) {
    println!(
        "\n\
        list of configurations\n\
        ----------------------"
    );
    print_opt!("cargo-home", "[default]", settings.cargo_home);
    print_opt!("rustup-home", "[default]", settings.rustup_home);
    print_opt!(
        "rustup-dist-server",
        "[default]",
        settings.rustup_dist_server
    );
    print_opt!(
        "rustup-update-root",
        "[default]",
        settings.rustup_update_root
    );
    print_opt!("proxy", "N/A", settings.proxy);
    print_opt!("no_proxy", "N/A", settings.no_proxy);
    print_opt!(
        "git-fetch-with-cli",
        "[default]",
        settings.cargo.as_ref().and_then(|c| c.git_fetch_with_cli)
    );
    print_opt!(
        "check-revoke",
        "[default]",
        settings.cargo.as_ref().and_then(|c| c.check_revoke)
    );
    print_opt!(
        "default-registry",
        "[default]",
        settings
            .cargo
            .as_ref()
            .and_then(|c| c.default_registry.as_ref())
    );
    print_opt!("registries", registries_string(settings));
    println!("----------------------\n");
}

fn registries_string(settings: &Settings) -> String {
    let content = settings
        .cargo
        .as_ref()
        .map(|c| &c.registries)
        .map(|hm| {
            hm.iter()
                .map(|(k, v)| format!("'{k} - ({})'", &v.index))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "default".to_string());
    format!("[{content}]")
}

fn import_config(
    path_str: &str,
    existing: &mut Configuration,
    create_new: bool,
    yes: bool,
) -> Result<()> {
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

    if !overriding(create_new, yes)? {
        return Ok(());
    }
    if importing_cfg.installation.is_some() {
        info!("force skipping `[installation]` sections");
    }

    existing.settings = importing_cfg.settings;
    update_config(existing)?;
    info!("configuration updated successfully.");

    Ok(())
}

/// Ask confirmation whether or not to override existing configuration, e.g.
/// returning `Ok(true)` means it will be overrided.
fn overriding(create_new: bool, yes: bool) -> Result<bool> {
    if !create_new {
        warn!("existing configuration detected");
        if !yes && !common::confirm("override? (y/n)", false)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn show_settings_diff(old: &Settings, new: &Settings) {
    diff!("cargo-home", "[default]", old => new, cargo_home);
    diff!("rustup-home", "[default]", old => new, rustup_home);
    diff!("rustup-dist-server", "[default]", old => new, rustup_dist_server);
    diff!("rustup-update-root", "[default]", old => new, rustup_update_root);
    diff!("proxy", "[N/A]", old => new, proxy);
    diff!("no-proxy", "[N/A]", old => new, no_proxy);
    diff!("git-fetch-with-cli", "[default]", old => new, cargo.as_ref().and_then(|c| c.git_fetch_with_cli));
    diff!("check-revoke", "[default]", old => new, cargo.as_ref().and_then(|c| c.check_revoke));
    diff!("default-registries", "[default]", old => new, cargo.as_ref().and_then(|c| c.default_registry.as_ref()));
    diff!("registries", registries_string(old) => registries_string(new));
    println!();
}

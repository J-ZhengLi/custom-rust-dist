use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use super::{GlobalOpt, Subcommands};
use crate::applog::*;
use crate::cli::{ConfigSubcommand, RegistryOpt};
use crate::mini_rustup::cli_common;
use crate::parser::{load_toml, CargoRegistry, CargoSettings, Configuration, Settings};
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

/// Print a nice message when two individual settings are different.
// TODO: reduce repeatition
macro_rules! diff {
    ($name:literal, $lhs:expr => $rhs:expr) => {{
        let lhs_str = $lhs;
        let rhs_str = $rhs;
        if lhs_str != rhs_str {
            let rhs_colored = logger::color::ColoredStr::new()
                .content(&rhs_str)
                .color(logger::color::Color::Green)
                .bright()
                .build();
            println!("{}: {} -> {}", $name, lhs_str, rhs_colored);
        }
    }};
    ($name:literal, $def:literal, $old:ident => $new:ident, $($opt:tt)*) => {{
        let lhs_str = $old.$($opt)*.as_ref().map(|t| t.to_string())
            .unwrap_or_else(|| $def.to_string());
        let rhs_str = $new.$($opt)*.as_ref().map(|t| t.to_string())
            .unwrap_or_else(|| $def.to_string());
        if lhs_str != rhs_str {
            let rhs_colored = logger::color::ColoredStr::new()
                .content(&rhs_str)
                .color(logger::color::Color::Green)
                .bright()
                .build();
            println!("{}: {} -> {}", $name, lhs_str, rhs_colored);
        }
    }};
}

/// Simple macro to replace an option value with another,
/// if only the other option contains some value.
macro_rules! apply {
    ($from:expr, $to:expr) => {
        if $from.is_some() {
            $to = $from.clone();
        }
    };
}

/// Processing `config` subcommand if it exist, otherwise this won't do anything.
pub(super) fn process(subcommand: &Subcommands, opt: GlobalOpt) -> Result<()> {
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
        input,
        default,
    } = subcommand else { return Ok(()) };

    let maybe_config = steps::load_config().ok();
    let create_new = maybe_config.is_none();
    let mut existing_config = &mut maybe_config.unwrap_or_default();

    if *list {
        if !opt.quiet && create_new {
            warn!("no configuration file detected, showing default configuration instead");
        }
        list_config(&existing_config.settings, opt);
        return Ok(());
    }
    if let Some(cfg_path_str) = input {
        import_config(cfg_path_str, existing_config, create_new, opt)?;
        return Ok(());
    }
    if let Some(true) = default {
        if !overriding("override with default", create_new, opt)? {
            return Ok(());
        }
        return apply_settings(existing_config, Settings::default(), opt);
    }

    let mut temp_settings = existing_config.settings.clone();
    let has_no_cargo_setts = temp_settings.cargo.is_none();
    // FIXME: this is a simple but not optimized solution to bypass lifetime restriction
    // of the next line.
    let mut def_cargo_setts = CargoSettings::default();
    let mut cargo_settings = temp_settings.cargo.as_mut().unwrap_or(&mut def_cargo_setts);
    // because `registry` is a seperated command, it will be checked seperatedly
    if let Some(ConfigSubcommand::Registry { opt: Some(reg_opt) }) = registry.as_ref() {
        match reg_opt {
            RegistryOpt::Default {
                default: Some(default),
            } => {
                cargo_settings.default_registry = Some(default.to_owned());
            }
            RegistryOpt::Add {
                url: Some(url),
                name,
            } => {
                let name_fullback =
                    name.as_deref().or_else(|| url.host_str()).ok_or_else(|| {
                        anyhow!(
                            "failed to automatically resolve the registry name, \
                            try using `--name` to specify one"
                        )
                    })?;
                cargo_settings
                    .registries
                    .insert(name_fullback.to_string(), url.as_str().to_string().into());
            }
            RegistryOpt::Remove { name: Some(name) } => {
                // user might passing the name with quotes, which somehow does not
                // took care by `clap`, but we need to remove them, otherwise the hashmap
                // might not able to find it.
                let raw_name = name.trim_matches(|c| c == '\'' || c == '"');
                cargo_settings.registries.remove(raw_name);
            }
            _ => return Ok(()),
        }
    } else {
        // apply provided configs one by one
        apply!(
            cargo_home.as_ref().map(PathBuf::from),
            temp_settings.cargo_home
        );
        apply!(
            rustup_home.as_ref().map(PathBuf::from),
            temp_settings.rustup_home
        );
        apply!(rustup_dist_server, temp_settings.rustup_dist_server);
        apply!(rustup_update_root, temp_settings.rustup_update_root);
        apply!(proxy, temp_settings.proxy);
        apply!(no_proxy, temp_settings.no_proxy);
        apply!(git_fetch_with_cli, cargo_settings.git_fetch_with_cli);
        apply!(check_revoke, cargo_settings.check_revoke);
    }
    if has_no_cargo_setts {
        temp_settings.cargo = Some(cargo_settings.clone());
    }

    apply_settings(existing_config, temp_settings, opt)
}

/// Format program settings ([`Settings`]), and prints them.
fn list_config(settings: &Settings, _opt: GlobalOpt) {
    println!(
        "\n\
        list of configurations\n\
        ----------------------"
    );
    print_opt!(
        "cargo-home",
        "[default]",
        settings.cargo_home.as_deref().map(Path::to_string_lossy)
    );
    print_opt!(
        "rustup-home",
        "[default]",
        settings.rustup_home.as_deref().map((Path::to_string_lossy))
    );
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
    opt: GlobalOpt,
) -> Result<()> {
    let cfg_path = Path::new(path_str);
    if !cfg_path.is_file() {
        bail!(
            "unable to import configuration '{}': path is not a file",
            cfg_path.display()
        );
    }
    let importing_cfg: Configuration = load_toml(cfg_path)?;

    if !overriding("overide", create_new, opt)? {
        return Ok(());
    }
    if !opt.quiet && importing_cfg.installation.is_some() {
        info!("force skipping `[installation]` sections");
    }

    apply_settings(existing, importing_cfg.settings, opt)
}

fn apply_settings(conf: &mut Configuration, mut setts: Settings, opt: GlobalOpt) -> Result<()> {
    // don't do anything if two settings are identical
    if conf.settings == setts {
        if !opt.quiet {
            info!("no change applied to user configuration");
        }
        return Ok(());
    }
    if matches!(setts.cargo.as_ref(), Some(cargo_setts) if cargo_setts.is_default()) {
        // avoids write an empty `[settings.cargo]` section in the result toml
        setts.cargo = None;
    }

    if !opt.quiet {
        info!("these settings will be updated to:");
        println!();
        show_settings_diff(&conf.settings, &setts);
    }
    conf.settings = setts;
    update_config(conf)?;
    if !opt.quiet {
        info!("configuration updated successfully.");
    }
    Ok(())
}

/// Ask confirmation whether or not to override existing configuration, e.g.
/// returning `Ok(true)` means it will be overrided.
fn overriding(msg: &str, create_new: bool, opt: GlobalOpt) -> Result<bool> {
    if !create_new {
        if !opt.quiet {
            warn!("existing configuration detected");
        }
        if !opt.yes && !cli_common::confirm(&format!("{msg}? (y/n)"), false)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn show_settings_diff(old: &Settings, new: &Settings) {
    diff!("cargo-home", "[default]", old => new, cargo_home.as_ref().map(|s| s.to_string_lossy()));
    diff!("rustup-home", "[default]", old => new, rustup_home.as_ref().map(|s| s.to_string_lossy()));
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

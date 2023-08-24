use std::env;
use std::path::PathBuf;

use anyhow::{Result, bail};
use logger::{err, info};
use reqwest::{NoProxy, Proxy};

use crate::mini_rustup::cli_common;
use crate::parser::{Configuration, Settings};
use crate::utils::DownloadOpt;
use crate::{defaults, mini_rustup, steps, utils};

use super::{GlobalOpt, Subcommands};

pub(super) fn process(subcommand: &Subcommands, opt: GlobalOpt) -> Result<()> {
    let Subcommands::Init {
        no_modify_path,
        root,
        rustup_update_root,
        rustup_version,
        proxy,
        no_proxy,
    } = subcommand else { return Ok(()) };

    if steps::load_config().is_ok() {
        bail!(
            "unable to initialize because configuration file already exists.\n\
            run `uninstall` first if you want to re-init, otherwise just run `config` with \
            desired values instead."
        );
    }
    

    if let Some(root_path) = root {
        // make sure this "root" directory exist
        utils::mkdirs(&root_path)?;
    }

    let cargo_home_string = confirm_env_override(
        "cargo-home (derived from `--root`)",
        root.as_ref().map(|r| format!("{r}/.cargo")),
        &["CARGO_HOME"],
        opt,
    )?
    .map(utils::to_nomalized_abspath);
    let rustup_home_string = confirm_env_override(
        "rustup-home (derived from `--root`)",
        root.as_ref().map(|r| format!("{r}/.rustup")),
        &["RUSTUP_HOME"],
        opt,
    )?
    .map(utils::to_nomalized_abspath);
    let rustup_update_root = confirm_env_override(
        "--rustup-update-root",
        rustup_update_root.as_ref().map(|u| u.as_str().into()),
        &["RUSTUP_UPDATE_ROOT"],
        opt,
    )?
    .map(|s| utils::parse_url(&s));
    let proxy = confirm_env_override(
        "--proxy",
        proxy.clone(),
        &["http_proxy", "https_proxy", "HTTP_PROXY", "HTTPS_PROXY"],
        opt,
    )?;
    let no_proxy = confirm_env_override(
        "--no-proxy",
        no_proxy.clone(),
        &["no_proxy", "NO_PROXY"],
        opt,
    )?;
    let config = Configuration {
        settings: Settings {
            cargo_home: utils::flip_option_result(cargo_home_string)?,
            rustup_home: utils::flip_option_result(rustup_home_string)?,
            rustup_update_root: utils::flip_option_result(rustup_update_root)?,
            proxy,
            no_proxy,
            ..Default::default()
        },
        ..Default::default()
    };

    let triple = mini_rustup::target_triple();
    download_and_install_rustup(rustup_version.as_deref(), &config, &triple, *no_modify_path)?;

    Ok(())
}

fn download_and_install_rustup(
    version: Option<&str>,
    config: &Configuration,
    triple: &str,
    no_modify_path: bool,
) -> Result<()> {
    #[cfg(windows)]
    let rustup_init_bin = "rustup-init.exe";
    #[cfg(not(windows))]
    let rustup_init_bin = "rustup-init";

    // The server prefix to download rustup from
    let server_root = config
        .settings
        .rustup_update_root
        .as_ref()
        .map(|u| u.as_str())
        .unwrap_or(defaults::RUSTUP_UPDATE_ROOT);
    let rustup_url_string = if let Some(ver) = version {
        format!("{server_root}/archive/{ver}/{triple}/{rustup_init_bin}")
    } else {
        format!("{server_root}/dist/{triple}/{rustup_init_bin}")
    };

    let rustup_url = utils::parse_url(&rustup_url_string)?;
    let temp_path = tempfile::Builder::new().prefix(crate::APPNAME).tempdir()?;
    let installer_dest = temp_path.path().join(rustup_init_bin);

    // Download rustup-init
    info!("downloading rustup-init from '{server_root}'");
    let proxy = configured_proxy_to_reqwest_proxy(config)?;
    let dl_opt = DownloadOpt::new(
        "rustup-init".to_string(),
        proxy,
        Some(super::extra::progress_bar_indicator()),
    )?;
    dl_opt.download_file(&rustup_url, &installer_dest, false)?;

    mini_rustup::utils::make_executable(&installer_dest)?;

    let env_vars = config.settings.to_key_value_pairs();
    let mut args = vec![
        "--default-toolchain",
        "none",
        "--no-update-default-toolchain",
        "-y",
    ];
    if no_modify_path {
        args.push("--no-modify-path")
    }

    info!("setting up rustup...");
    // TODO: find a way to forward error messages to current console.
    utils::execute_for_output_with_env(&installer_dest, &args, env_vars)?;
    info!("rustup successfully installed");

    Ok(())
}

fn configured_proxy_to_reqwest_proxy(config: &Configuration) -> Result<Option<Proxy>> {
    let Some(proxy_url) = config.settings.proxy.as_ref() else { return Ok(None) };
    let proxy = Proxy::all(proxy_url)?;
    let maybe_no_proxy = config
        .settings
        .no_proxy
        .as_ref()
        .and_then(|s| NoProxy::from_string(s));
    Ok(Some(proxy.no_proxy(maybe_no_proxy)))
}

/// Notify when the provided `val` is conflicting with a certain set of environment variables,
/// and return a value base on user's choice.
fn confirm_env_override(
    name: &str,
    val: Option<String>,
    env_key: &[&str],
    opt: GlobalOpt,
) -> Result<Option<String>> {
    if opt.yes {
        return Ok(val);
    }
    let existing_env_var = env_key
        .iter()
        .find_map(|key| env::var(key).ok().map(|v| (key, v)));

    if let Some(val_inner) = &val {
        if let Some((env_key, env_val)) = existing_env_var {
            // Both specified val and env var exists.
            if val_inner == &env_val {
                // Both specified val and env var exists but are the same, so return either one.
                return Ok(Some(env_val));
            }

            if !opt.quiet {
                info!(
                    "specified value of '{name}' is already exist as environment variable '{env_key}', \
                    continue with specified value? (This will overrides its environment variable)"
                );
            }
            if !cli_common::confirm("Override (Y/n):", true)? {
                return Ok(Some(env_val));
            }
        }
    } else {
        if let Some((env_key, env_val)) = existing_env_var {
            // Only env var exists.
            if !opt.quiet {
                info!(
                    "value of '{name}' was not specified but exists as environment variable '{env_key}', \
                    do you want to keep it unspecified? (This will overrides its environment variable)"
                );
            }
            if !cli_common::confirm("Keep value unspecified (y/N):", true)? {
                return Ok(Some(env_val));
            }
        }
    }
    Ok(val)
}

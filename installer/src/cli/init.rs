use std::env;
use std::path::PathBuf;

use anyhow::Result;
use logger::{err, info};
use reqwest::{NoProxy, Proxy};

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
        err!("unable to initialize because a configuration is already exist.");
        return Ok(());
    }

    let root = root
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(utils::home_dir);
    let cargo_home = root.join(".cargo");
    let rustup_home = root.join(".rustup");
    // read the env var if the user did not pass the arg
    let update_root = rustup_update_root.clone().or(env::var("RUSTUP_UPDATE_ROOT")
        .ok()
        .and_then(|s| s.parse().ok()));
    let proxy = proxy
        .clone()
        .or(env::var("http_proxy").ok())
        .or(env::var("HTTP_PROXY").ok())
        .or(env::var("https_proxy").ok())
        .or(env::var("HTTPS_PROXY").ok());
    let no_proxy = no_proxy
        .clone()
        .or(env::var("no_proxy").ok())
        .or(env::var("NO_PROXY").ok());
    let config = Configuration {
        settings: Settings {
            cargo_home: Some(utils::stringify_path(cargo_home)?),
            rustup_home: Some(utils::stringify_path(rustup_home)?),
            rustup_update_root: update_root,
            proxy,
            no_proxy,
            ..Default::default()
        },
        ..Default::default()
    };

    let triple = mini_rustup::target_triple();

    // make sure this "root" directory exist
    utils::mkdirs(&root)?;
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

use std::path::PathBuf;

use custom_rust_dist::{
    cli::{CliOpt, Subcommands, UninstallCommand},
    utils,
};
use serde_json::Value;
use tauri::api::cli::Matches;

pub fn tauri_cli_to_cli_opt(matches: &Matches) -> Option<CliOpt> {
    let mut cli_opt = CliOpt::default();

    for (arg_name, arg_data) in &matches.args {
        match (arg_name.as_str(), &arg_data.value) {
            ("verbose", Value::Bool(b)) => cli_opt.verbose = *b,
            ("quiet", Value::Bool(b)) => cli_opt.quiet = *b,
            ("yes_to_all", Value::Bool(b)) => cli_opt.yes_to_all = *b,
            _ => (),
        }
    }

    if let Some(subcmd) = &matches.subcommand {
        cli_opt.command = match (subcmd.name.as_str(), &subcmd.matches) {
            ("install", sc_matches) => parse_install_args(sc_matches),
            ("uninstall", sc_matches) => parse_uninstall_args(sc_matches),
            _ => None,
        }
    }

    Some(cli_opt)
}

macro_rules! init_mut_none {
    ($($name:ident),+) => {
        $(
            let mut $name = None;
        )*
    };
}

fn parse_install_args(matches: &Matches) -> Option<Subcommands> {
    if matches.subcommand.is_some() {
        // `install` command doesn't have subcommands yet.
        return None;
    }

    init_mut_none!(
        prefix,
        registry_url,
        registry_name,
        rustup_dist_server,
        rustup_update_root
    );

    for (arg_name, arg_data) in &matches.args {
        match (arg_name.as_str(), &arg_data.value) {
            ("prefix", Value::String(path_str)) => prefix = Some(PathBuf::from(path_str)),
            ("registry_url", Value::String(s)) => registry_url = Some(utils::force_parse_url(s)),
            ("registry_name", Value::String(s)) => registry_name = Some(s.to_string()),
            ("rustup_dist_server", Value::String(s)) => {
                rustup_dist_server = Some(utils::force_parse_url(s))
            }
            ("rustup_update_root", Value::String(s)) => {
                rustup_update_root = Some(utils::force_parse_url(s))
            }
            _ => (),
        }
    }

    Some(Subcommands::Install {
        prefix,
        registry_url,
        registry_name: registry_name.unwrap_or_else(|| String::from("mirror")),
        rustup_dist_server,
        rustup_update_root,
    })
}

fn parse_uninstall_args(matches: &Matches) -> Option<Subcommands> {
    let Some(subcmd) = &matches.subcommand else {
        return None;
    };

    let uninstall_cmd = match (subcmd.name.as_str(), &subcmd.matches) {
        ("all", _) => UninstallCommand::All,
        (
            "tool",
            Matches {
                args,
                subcommand: None,
                ..
            },
        ) => {
            if let Some(names_arg) = args.get("names") {
                UninstallCommand::Tool {
                    names: value_to_string_list(&names_arg.value)?,
                }
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(Subcommands::Uninstall {
        commands: Some(uninstall_cmd),
    })
}

fn value_to_string_list(value: &Value) -> Option<Vec<String>> {
    let arr = value.as_array()?;
    if !arr.iter().all(|arg| arg.is_string()) {
        return None;
    }
    Some(
        arr.iter()
            .map(|v| v.as_str().map(ToString::to_string).unwrap())
            .collect(),
    )
}

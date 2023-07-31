use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand, ValueHint};
use url::Url;

mod common;
mod config;

#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Suppress all messages
    #[arg(short, long)]
    quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(short, long = "yes")]
    yes_to_all: bool,
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub(crate) enum Subcommands {
    /// Adjust program configurations
    Config {
        /// List all configuration
        #[arg(short, long)]
        list: bool,
        /// Set the path to cargo home, where the binaries and crate caches are located
        #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        cargo_home: Option<String>,
        /// Set the path to rustup home, where the Rust toolchains are located
        #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        rustup_home: Option<String>,
        /// Specify which server to download Rust toolchains from
        #[arg(long, value_name = "URL", value_hint = ValueHint::Url)]
        rustup_dist_server: Option<Url>,
        /// Specify which site to download rustup
        #[arg(long, value_name = "URL", value_hint = ValueHint::Url)]
        rustup_update_root: Option<Url>,
        /// Set internet proxy
        #[arg(long, value_name = "URL", value_hint = ValueHint::Url)]
        proxy: Option<String>,
        /// Skip proxy for following hostname globs, seperated by commas
        #[arg(long, value_name = "NAMES")]
        no_proxy: Option<String>,
        /// [Cargo] use the `git` executable for git operations
        #[arg(long)]
        git_fetch_with_cli: Option<bool>,
        /// [Cargo] check for SSL certificate revocation
        #[arg(long)]
        check_revoke: Option<bool>,
        #[command(subcommand)]
        registry: Option<ConfigSubcommand>,
        /// Load configuration from a file
        #[arg(
            short,
            long,
            value_name = "FILE",
            value_hint = ValueHint::FilePath,
            conflicts_with_all = [
                "cargo_home", "rustup_home", "rustup_dist_server", "rustup_update_root",
                "proxy", "no_proxy", "git_fetch_with_cli", "check_revoke",
            ]
        )]
        input: Option<String>,
        /// Use default settings
        #[arg(
            short,
            long,
            action = ArgAction::SetTrue,
            conflicts_with_all = [
                "cargo_home", "rustup_home", "rustup_dist_server", "rustup_update_root",
                "proxy", "no_proxy", "git_fetch_with_cli", "check_revoke", "input",
            ]
        )]
        default: Option<bool>,
    },
    /// Install new components, including tools or toolchains
    Install,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigSubcommand {
    /// [Cargo] Configurations for cargo registries
    Registry {
        #[command(subcommand)]
        opt: Option<RegistryOpt>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum RegistryOpt {
    /// Set the default registry to download crates from
    Default {
        /// Name of an existing registry
        default: Option<String>,
    },
    /// Add a cargo registry
    Add {
        /// Url of the cargo registry
        url: Option<Url>,
        /// Name of the cargo registry, URL hostname will be used if not set
        #[arg(short, long, value_name = "NAME")]
        name: Option<String>,
    },
    /// Remove a certain registry by its name
    Rm {
        /// Name of the carge registry
        name: Option<String>,
    },
}

impl Subcommands {
    pub fn process(&self, verbose: bool, yes: bool) -> Result<()> {
        config::process(self, verbose, yes)?;
        Ok(())
    }
}

pub(crate) fn run() -> Result<()> {
    // let config = steps::load_config()?;
    // println!("config: {:?}", config);
    let cli = Cli::parse();

    let verbose = cli.verbose;
    let quiet = cli.quiet;
    let yes_to_all = cli.yes_to_all;

    match &cli.command {
        Some(subcommand) => subcommand.process(verbose, yes_to_all),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;

    #[test]
    fn cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}

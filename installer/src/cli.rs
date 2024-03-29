use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand, ValueHint};
use url::Url;

mod config;
mod extra;
mod init;
mod install;

#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, conflicts_with_all = ["quiet", "yes_to_all"])]
    verbose: bool,
    /// Suppress non-critical messages
    #[arg(short, long, conflicts_with_all = ["verbose", "yes_to_all"])]
    quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(short, long = "yes", conflicts_with_all = ["quiet", "verbose"])]
    yes_to_all: bool,
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub(crate) enum Subcommands {
    /// Adjust program configurations
    Config {
        // TODO: move this list arg as a separated subcommand
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
    },
    /// Install rustup, rust toolchain, or various tools
    Install {
        #[command(subcommand)]
        commands: Option<InstallCommand>,
    },
    /// Uninstall options
    Uninstall {
        #[command(subcommand)]
        commands: Option<UninstallCommand>,
    },
    /// Initialize application by setting up environment, and install rustup
    Init {
        /// Don't configure the PATH environment variable
        #[arg(long)]
        no_modify_path: bool,
        /// Specify a root path for installation, default to `$HOME/rupe`
        #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        root: Option<String>,
        /// Specify which server to download rustup-init
        #[arg(long, value_name = "URL", value_hint = ValueHint::Url)]
        rustup_update_root: Option<Url>,
        /// Choose a specific version of rustup to install
        #[arg(long)]
        rustup_version: Option<String>,
        /// Set proxy server. If this option was not provided,
        /// the program will attemp to read it from env vars
        #[arg(long, value_name = "URL", value_hint = ValueHint::Url)]
        proxy: Option<String>,
        /// Set no proxy, separated using comma ','. If this option was not provided,
        /// the program will attemp to read it from env vars
        #[arg(long)]
        no_proxy: Option<String>,
    },
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
    #[command(visible_alias = "rm")]
    /// Remove a certain registry by its name
    Remove {
        /// Name of the carge registry
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum InstallCommand {
    /// Install rust toolchain, requires `rustup` being installed
    Toolchain {
        /// Specify a (toolchain) version of rust to install
        toolchain: Option<String>,
        /// Add specific targets on installation
        #[arg(short, long, value_name = "TARGETS")]
        target: Option<String>,
        /// Specify a different toolchain profile
        #[arg(long)]
        profile: Option<String>,
        /// Add specific components on installation
        #[arg(short, long, value_name = "COMPONENTS")]
        component: Option<String>,
        /// Filesystem path to local toolchain directory/package
        #[arg(long, value_hint = ValueHint::AnyPath)]
        path: Option<String>,
    },
    /// Install standalone tools for rust, requires `cargo` being installed
    Tool {
        /// Name of cargo crates
        name: Option<String>,
        /// Filesystem path to local crate/package to install
        #[arg(long, value_hint = ValueHint::AnyPath)]
        path: Option<String>,
        /// Git URL to install the specified crate from
        #[arg(long, value_name = "URL")]
        git: Option<Url>,
        /// Specify a version to install
        #[arg(long, conflicts_with = "path")]
        version: Option<String>,
        /// Force overwriting existing crates or binaries
        #[arg(short, long)]
        force: bool,
        /// Space or comma separated list of features to activate
        #[arg(short = 'F', long)]
        features: Option<Vec<String>>,
    },
    /// Install component for toolchain
    Component {
        /// Add a specific toolchain component
        name: Option<String>,
        #[arg(long)]
        /// Toolchain name, such as 'stable', 'nightly', or '1.8.0'
        toolchain: Option<String>,
        #[arg(long)]
        target: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum UninstallCommand {
    All,
}

impl Subcommands {
    pub fn process(&self, opt: GlobalOpt) -> Result<()> {
        init::process(self, opt)?;
        config::process(self, opt)?;
        install::process(self, opt)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GlobalOpt {
    verbose: bool,
    quiet: bool,
    yes: bool,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let global_opt = GlobalOpt {
        verbose: cli.verbose,
        quiet: cli.quiet,
        yes: cli.yes_to_all,
    };

    match &cli.command {
        Some(subcommand) => subcommand.process(global_opt),
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

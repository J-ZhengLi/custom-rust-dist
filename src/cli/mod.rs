//! Contains all the definition of command line arguments.

mod common;
mod install;
mod tryit;
mod uninstall;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::{Path, PathBuf};
use url::Url;

/// Install rustup, rust toolchain, and various tools.
// NOTE: If you changed anything in this struct, or any other child types that related to
// this struct, make sure the README doc is updated as well,
#[derive(Parser, Default, Debug)]
#[command(version, about)]
pub struct Installer {
    /// Enable verbose output
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,
    /// Suppress non-critical messages
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(short, long = "yes")]
    pub yes_to_all: bool,
    #[cfg(feature = "gui")]
    /// Don't show GUI when running the program.
    #[arg(long)]
    pub no_gui: bool,

    /// Set another path to install Rust.
    #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath)]
    pub prefix: Option<PathBuf>,
    /// Specify another cargo registry url to replace `crates.io`, could be `sparse+URL`.
    #[arg(hide = true, long)]
    pub registry_url: Option<String>,
    /// Specify another cargo registry name to replace `crates.io`.
    #[arg(hide = true, long, default_value = "mirror")]
    pub registry_name: String,
    /// Specify another server to download Rust toolchain.
    #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
    pub rustup_dist_server: Option<Url>,
    /// Specify another server to download rustup.
    #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
    pub rustup_update_root: Option<Url>,
}

/// Manage Rust installation, mostly used for uninstalling.
// NOTE: If you changed anything in this struct, or any other child types that related to
// this struct, make sure the README doc is updated as well,
#[derive(Parser, Default, Debug)]
#[command(version, about)]
#[command(arg_required_else_help = true)]
pub struct Manager {
    /// Enable verbose output
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,
    /// Suppress non-critical messages
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(short, long = "yes")]
    pub yes_to_all: bool,
    #[cfg(feature = "gui")]
    /// Don't show GUI when running the program.
    #[arg(long)]
    pub no_gui: bool,
    #[command(subcommand)]
    pub command: Option<ManagerSubcommands>,
}

impl Installer {
    pub fn install_dir(&self) -> Option<&Path> {
        self.prefix.as_deref()
    }

    pub fn execute(&self) -> Result<()> {
        install::execute_installer(self)
    }
}

impl Manager {
    pub fn execute(&self) -> Result<()> {
        let global_opt = GlobalOpt {
            verbose: self.verbose,
            quiet: self.quiet,
            yes: self.yes_to_all,
        };

        if let Some(subcommand) = &self.command {
            subcommand.execute(global_opt)
        } else {
            Ok(())
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum ManagerSubcommands {
    // TODO: Add install command to install **individual** components.
    /// Uninstall individual components or everything.
    Uninstall {
        #[command(subcommand)]
        commands: Option<UninstallCommand>,
    },
    /// A subcommand to create a new Rust project template and let you start coding with it.
    TryIt {
        /// Specify another directory to create project template, defaulting to current directory.
        #[arg(long, short, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
    },
}

impl ManagerSubcommands {
    pub(crate) fn execute(&self, opt: GlobalOpt) -> Result<()> {
        uninstall::execute(self, opt)?;
        tryit::execute(self, opt)?;
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub enum UninstallCommand {
    /// Uninstall everything.
    All,
    /// Uninstall a list of individual tools, separated by space.
    Tool {
        #[arg(value_name = "TOOLS")]
        names: Vec<String>,
    },
}

/// Contain options that are accessed globally.
///
/// Such as `--verbose`, `--quiet`, `--yes`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GlobalOpt {
    pub verbose: bool,
    pub quiet: bool,
    pub yes: bool,
}

pub fn parse_installer_cli() -> Installer {
    Installer::parse()
}

pub fn parse_manager_cli() -> Manager {
    Manager::parse()
}

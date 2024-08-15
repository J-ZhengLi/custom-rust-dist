//! Contains all the definition of command line arguments.

mod install;
mod tryit;
mod uninstall;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::{Path, PathBuf};
use url::Url;

/// Master struct for command line args.
///
// NOTE: If you changed anything in this struct, or any other child types that related to
// this struct, make sure the README doc is updated as well,
// also make sure to update the cli section of `installer/src-tauri/tauri.conf.json`.
#[derive(Parser, Default, Debug)]
#[command(version, about)]
pub struct CliOpt {
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
    pub command: Option<Subcommands>,
}

impl CliOpt {
    pub fn install_dir(&self) -> Option<&Path> {
        self.command.as_ref().and_then(|cmd| {
            if let Subcommands::Install { prefix, .. } = cmd {
                prefix.as_deref()
            } else {
                None
            }
        })
    }

    pub fn execute(&self) -> Result<()> {
        let global_opt = GlobalOpt {
            verbose: self.verbose,
            quiet: self.quiet,
            yes: self.yes_to_all,
        };

        if let Some(subcommand) = &self.command {
            subcommand.execute(global_opt)
        } else {
            // Do nothing if no subcommand provided.
            // Note that options like `--version`, `--help` are already handled by `clap`.
            Ok(())
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Install rustup, rust toolchain, and various tools.
    Install {
        /// Set another path to install Rust.
        #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath)]
        prefix: Option<PathBuf>,
        /// Specify another cargo registry url to replace `crates.io`.
        #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
        registry_url: Option<Url>,
        /// Specify another cargo registry name to replace `crates.io`.
        #[arg(hide = true, long, default_value = "mirror", value_hint = ValueHint::Url)]
        registry_name: String,
        /// Specify another server to download Rust toolchain.
        #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
        rustup_dist_server: Option<Url>,
        /// Specify another server to download rustup.
        #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
        rustup_update_root: Option<Url>,
    },
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

impl Subcommands {
    pub(crate) fn execute(&self, opt: GlobalOpt) -> Result<()> {
        install::execute(self, opt)?;
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

pub fn parse_cli() -> CliOpt {
    CliOpt::parse()
}

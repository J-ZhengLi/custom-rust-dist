//! Contains all the definition of command line arguments.

mod install;
mod uninstall;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Master struct for command line args.
///
/// NOTE: If you changed anything in this struct, or any other child types that related to
/// this struct, make sure the README doc is updated as well.
#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct CliOpt {
    /// Enable verbose output
    #[arg(short, long, conflicts_with = "quiet")]
    verbose: bool,
    /// Suppress non-critical messages
    #[arg(short, long, conflicts_with = "verbose")]
    quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(short, long = "yes")]
    yes_to_all: bool,
    #[command(subcommand)]
    command: Option<Subcommands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Subcommands {
    /// Install rustup, rust toolchain, and various tools.
    Install,
    /// Uninstall individual components or everything.
    Uninstall {
        #[command(subcommand)]
        commands: Option<UninstallCommand>,
    },
}

impl Subcommands {
    pub fn execute(&self, opt: GlobalOpt) -> Result<()> {
        install::execute(self, opt)?;
        uninstall::execute(self, opt)?;
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
#[command(arg_required_else_help = true)]
pub(crate) enum UninstallCommand {
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
#[derive(Debug, Clone, Copy)]
pub struct GlobalOpt {
    pub verbose: bool,
    pub quiet: bool,
    pub yes: bool,
}

pub fn run() -> Result<()> {
    let cli = CliOpt::parse();

    let global_opt = GlobalOpt {
        verbose: cli.verbose,
        quiet: cli.quiet,
        yes: cli.yes_to_all,
    };

    let Some(subcommand) = &cli.command else {
        // Do nothing if no subcommand provided.
        // Note that options like `--version`, `--help` are already handled by `clap`.
        return Ok(());
    };

    subcommand.execute(global_opt)
}

//! Contains all the definition of command line arguments.

mod common;
mod component;
mod install;
mod list;
mod tryit;
mod uninstall;
mod update;

use anyhow::{anyhow, bail, Result};
use clap::{Parser, Subcommand, ValueHint};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

use crate::utils;

/// Install rustup, rust toolchain, and various tools.
// NOTE: If you changed anything in this struct, or any other child types that related to
// this struct, make sure the README doc is updated as well,
#[derive(Parser, Default, Debug)]
#[command(version, about)]
pub struct Installer {
    /// Enable verbose output
    #[arg(hide = true, short, long, conflicts_with = "quiet")]
    verbose: bool,
    /// Suppress non-critical messages
    #[arg(hide = true, short, long, conflicts_with = "verbose")]
    quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(hide = true, short, long = "yes")]
    yes_to_all: bool,
    #[cfg(feature = "gui")]
    /// Don't show GUI when running the program.
    #[arg(hide = true, long)]
    pub no_gui: bool,

    /// Set another path to install Rust.
    #[arg(long, value_name = "PATH", value_hint = ValueHint::DirPath)]
    prefix: Option<PathBuf>,
    /// Specify another cargo registry url to replace `crates.io`, could be `sparse+URL`.
    #[arg(hide = true, long)]
    registry_url: Option<String>,
    /// Specify another cargo registry name to replace `crates.io`.
    #[arg(hide = true, long, default_value = "mirror")]
    registry_name: String,
    /// Specify another server to download Rust toolchain.
    #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
    rustup_dist_server: Option<Url>,
    /// Specify another server to download rustup.
    #[arg(hide = true, long, value_name = "URL", value_hint = ValueHint::Url)]
    rustup_update_root: Option<Url>,
    /// Specify a path or url of manifest file that contains package source and various configurations.
    #[arg(long, value_name = "PATH or URL")]
    manifest: Option<PathOrUrl>,
}

#[derive(Debug, Clone)]
pub(crate) enum PathOrUrl {
    Path(PathBuf),
    Url(Url),
}

impl FromStr for PathOrUrl {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        if let Ok(abs_path) = utils::to_nomalized_abspath(s, None) {
            if !abs_path.exists() {
                bail!("the specified path '{s}' does not exist");
            }
            Ok(PathOrUrl::Path(abs_path))
        } else {
            Ok(PathOrUrl::Url(Url::parse(s)?))
        }
    }
}

impl PathOrUrl {
    /// Extract [`Url`] value or convert [`PathBuf`] to [`Url`] with file scheme.
    ///
    /// # Error
    /// This will fail when trying to convert a relative path.
    fn to_url(&self) -> Result<Url> {
        match self {
            Self::Url(url) => Ok(url.clone()),
            Self::Path(path) => {
                Url::from_file_path(path).map_err(|_| anyhow!("invalid path '{}'", path.display()))
            }
        }
    }
}

/// Manage Rust installation, mostly used for uninstalling.
// NOTE: If you changed anything in this struct, or any other child types that related to
// this struct, make sure the README doc is updated as well,
#[derive(Parser, Debug)]
#[command(version, about)]
#[cfg_attr(not(feature = "gui"), command(arg_required_else_help(true)))]
pub struct Manager {
    /// Enable verbose output
    #[arg(hide = true, short, long, conflicts_with = "quiet")]
    verbose: bool,
    /// Suppress non-critical messages
    #[arg(hide = true, short, long, conflicts_with = "verbose")]
    quiet: bool,
    /// Disable interaction and answer 'yes' to all prompts
    #[arg(hide = true, short, long = "yes")]
    yes_to_all: bool,
    #[cfg(feature = "gui")]
    /// Don't show GUI when running the program.
    #[arg(hide = true, long)]
    pub no_gui: bool,
    #[command(subcommand)]
    command: Option<ManagerSubcommands>,
}

impl Installer {
    pub fn install_dir(&self) -> Option<&Path> {
        self.prefix.as_deref()
    }

    pub fn manifest_url(&self) -> Result<Option<Url>> {
        self.manifest.as_ref().map(|m| m.to_url()).transpose()
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

        let Some(subcmd) = &self.command else {
            return Ok(());
        };
        subcmd.execute(global_opt)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug)]
enum ManagerSubcommands {
    /// Install a specific dist version
    #[command(hide = true)]
    Install {
        #[arg(value_name = "VERSION")]
        version: String,
    },
    #[command(hide = true)]
    /// Update the current installed dist suite to the newest version
    Update {
        /// We keep the Manager version unchanged by default.
        /// You can update the Manager using the flag
        #[arg(long)]
        self_update: bool,
    },
    #[command(hide = true)]
    /// Show a list of available dist version or components
    List {
        /// Prints the current installed dist version
        #[arg(long)]
        installed: bool,
        #[command(subcommand)]
        command: Option<list::ListCommand>,
    },
    #[command(hide = true)]
    /// Install or uninstall components
    Component {
        #[command(subcommand)]
        command: component::ComponentCommand,
    },
    /// Uninstall individual components or everything.
    Uninstall {
        /// Remove this manager tool as well
        #[arg(long)]
        remove_self: bool,
    },
    /// A subcommand to create a new Rust project template and let you start coding with it.
    TryIt {
        /// Specify another directory to create project template, defaulting to current directory.
        #[arg(long, short, value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
    },
}

macro_rules! return_if_executed {
    ($($fn:expr),+) => {
        $(
            if $fn {
                return Ok(());
            }
        )*
    };
}

impl ManagerSubcommands {
    pub(crate) fn execute(&self, opt: GlobalOpt) -> Result<()> {
        return_if_executed! {
            install::execute_manager(self, opt)?,
            update::execute(self, opt)?,
            list::execute(self, opt)?,
            component::execute(self, opt)?,
            uninstall::execute(self, opt)?,
            tryit::execute(self, opt)?
        }
        Ok(())
    }
}

/// Contain options that are accessed globally.
///
/// Such as `--verbose`, `--quiet`, `--yes`.
#[allow(unused)]
#[derive(Debug, Clone, Copy, Default)]
struct GlobalOpt {
    verbose: bool,
    quiet: bool,
    yes: bool,
}

pub fn parse_installer_cli() -> Installer {
    Installer::parse()
}

pub fn parse_manager_cli() -> Manager {
    Manager::parse()
}

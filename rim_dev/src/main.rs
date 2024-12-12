#[macro_use]
extern crate rust_i18n;

mod common;
mod dist;
mod mocked;
mod vendor;

use anyhow::Result;
use dist::{DistMode, DIST_HELP};
use mocked::{installation, manager, server};
use std::env;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use vendor::VENDOR_HELP;

i18n!("../locales", fallback = "en");

const HELP: &str = r#"
Usage: cargo dev [OPTIONS] [COMMAND]

Options:
    -h, -help       Print this help message

Commands:
    dist, d         Generate release binaries
    run-manager     Run with manager mode
    vendor          Download packages that are specified in `resource/packages.txt`
"#;

const MANAGER_MODE_HELP: &str = r#"
Run with manager mode

Usage: cargo dev run-manager [OPTIONS]

Options:
        --cli       Run manager mode with commandline interface
        --gui       Run manager mode with graphical interface (default)
    -h, -help       Print this help message
"#;

#[derive(Debug)]
enum DevCmd {
    Dist { mode: DistMode, binary_only: bool },
    RunManager { no_gui: bool, args: Vec<String> },
    Vendor,
}

impl DevCmd {
    fn execute(&self) -> Result<()> {
        match self {
            Self::Dist { mode, binary_only } => dist::dist(*mode, *binary_only)?,
            Self::RunManager { no_gui, args } => {
                // a mocked server is needed to run most of function in manager
                server::generate()?;

                // generate a fake manager binary with higher version so we
                // can test the self update.
                if args.iter().any(|arg| arg == "update") {
                    manager::generate()?;
                }

                installation::generate_and_run_manager(*no_gui, args)?;
            }
            Self::Vendor => vendor::vendor()?,
        }
        Ok(())
    }
}

fn current_exe() -> PathBuf {
    env::current_exe().expect("failed to get the path of current binary")
}

fn main() -> Result<ExitCode> {
    let mut args = std::env::args().skip(1);
    let mut stdout = stdout();

    let Some(subcmd) = args.next() else {
        writeln!(&mut stdout, "{HELP}")?;
        return Ok(ExitCode::FAILURE);
    };

    let cmd = match subcmd.to_lowercase().as_str() {
        "-h" | "--help" => {
            writeln!(&mut stdout, "{HELP}")?;
            return Ok(ExitCode::SUCCESS);
        }
        "d" | "dist" => {
            let mut binary_only = false;
            let mut mode = DistMode::Both;

            match args.next().as_deref() {
                Some("-h" | "--help") => {
                    writeln!(&mut stdout, "{DIST_HELP}")?;
                    return Ok(ExitCode::SUCCESS);
                }
                Some("--cli") => mode = DistMode::Cli,
                Some("--gui") => mode = DistMode::Gui,
                Some("-b" | "--binary-only") => binary_only = true,
                _ => (),
            };
            DevCmd::Dist { mode, binary_only }
        }
        "vendor" => match args.next().as_deref() {
            Some("-h" | "--help") => {
                writeln!(&mut stdout, "{VENDOR_HELP}")?;
                return Ok(ExitCode::SUCCESS);
            }
            Some(s) => {
                writeln!(&mut stdout, "invalid argument '{s}'")?;
                return Ok(ExitCode::FAILURE);
            }
            None => DevCmd::Vendor,
        },
        "run-manager" => match args.next().as_deref() {
            Some("-h" | "--help") => {
                writeln!(&mut stdout, "{MANAGER_MODE_HELP}")?;
                return Ok(ExitCode::SUCCESS);
            }
            Some("--cli") => DevCmd::RunManager {
                no_gui: true,
                args: args.collect(),
            },
            _ => DevCmd::RunManager {
                no_gui: false,
                args: args.collect(),
            },
        },
        s => {
            writeln!(
                &mut stdout,
                "invalid argument '{s}', check 'cargo dev --help' for available options"
            )?;
            return Ok(ExitCode::FAILURE);
        }
    };
    cmd.execute()?;

    Ok(ExitCode::SUCCESS)
}

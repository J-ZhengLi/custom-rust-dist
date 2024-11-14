#[macro_use]
extern crate rust_i18n;

mod common;
mod dist;
mod mocked_manager;
mod vendor;

use anyhow::{Context, Result};
use dist::{DistMode, DIST_HELP};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::{env, fs};
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
                let mut cargo_args = if *no_gui {
                    ["run", "--"].to_vec()
                } else {
                    ["tauri", "dev", "--"].to_vec()
                };
                cargo_args.extend(args.iter().map(|s| s.as_str()));

                gen_mocked_files()?;
                if args.iter().any(|arg| arg == "update") {
                    mocked_manager::generate()?;
                }

                let mut mock_dir =
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("resources");
                mock_dir.push("mock");
                let mocked_server = url::Url::from_directory_path(&mock_dir).unwrap_or_else(|_| {
                    panic!("path {} cannot be converted to URL", mock_dir.display())
                });
                let status = Command::new("cargo")
                    .args(cargo_args)
                    .env("MODE", "manager")
                    .env("RIM_DIST_SERVER", mocked_server.as_str())
                    .status()?;
                println!(
                    "\nmanager exited with status code: {}",
                    status.code().unwrap_or(-1)
                );
            }
            Self::Vendor => vendor::vendor()?,
        }
        Ok(())
    }
}

fn current_exe() -> Result<PathBuf> {
    env::current_exe().context("failed to get the path of current binary")
}

/// Generate mocked `.fingerprint`, and `toolset-manifest` files when running with `run-manager`
fn gen_mocked_files() -> Result<()> {
    let cur_exe = current_exe()?;
    // safe to unwrap, always have parent dir
    let debug_dir = cur_exe.parent().unwrap();
    // Note: there is a `.fingerprint` folder generated by cargo, don't touch it
    let fingerprint_path = debug_dir.join(".fingerprint.toml");
    fs::write(
        fingerprint_path,
        format!(
            "
name = 'Custom Rust Distribution'
version = 'stable-1.80.1'
root = '{0}'

[rust]
version = '1.80.1'
components = [\"llvm-tools\", \"rustc-dev\"]

[tools.mingw64]
use-cargo = false
paths = ['{0}/tools/mingw64']",
            debug_dir.display()
        ),
    )?;

    let manifest = include_str!("../../resources/toolset_manifest.toml");
    let manifest_path = debug_dir.join("toolset-manifest.toml");
    fs::write(manifest_path, manifest)?;

    Ok(())
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

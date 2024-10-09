use anyhow::{bail, Context, Result};
use cfg_if::cfg_if;
use rust_i18n::{i18n, t};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::{env, fs};

i18n!("../locales", fallback = "en");

const HELP: &str = r#"
Usage: cargo dev [OPTIONS] [COMMAND]

Options:
    -h, -help       Print this help message

Commands:
    dist, d         Generate release binaries
    run-manager     Run with manager mode
    set-vendor      Set binaries' vendor name and the identifier etc
"#;

const DIST_HELP: &str = r#"
Generate release binaries

Usage: cargo dev dist [OPTIONS]

Options:
        --cli       Generate release binary for CLI mode only
        --gui       Generate release binary for GUI mode only
    -h, -help       Print this help message
"#;

const MANAGER_MODE_HELP: &str = r#"
Run with manager mode

Usage: cargo dev run-manager [OPTIONS]

Options:
        --cli       Run manager mode with commandline interface
        --gui       Run manager mode with graphical interface (default)
    -h, -help       Print this help message
"#;

const SET_VENDOR_HELP: &str = r#"
Set binaries' vendor name and the identifier etc

Usage: cargo dev set-vendor [OPTIONS] <ARG>

Arguments:
    NAME:           Name of the vendor to replace

Options:
    -h, -help       Print this help message
"#;

#[derive(Debug)]
enum DistMode {
    Both,
    Gui,
    Cli,
}

#[derive(Debug)]
enum DevCmd {
    Dist { mode: DistMode },
    RunManager { no_gui: bool, args: Vec<String> },
    SetVendor { vendor: String },
}

impl DevCmd {
    fn execute(&self) -> Result<()> {
        match self {
            Self::Dist { mode } => {
                let x = match mode {
                    DistMode::Cli => {
                        vec![(
                            ["build", "--release", "--locked"].as_slice(),
                            format!("rim-cli{}", env::consts::EXE_SUFFIX),
                            format!(
                                "{}-installer-cli{}",
                                t!("vendor_en"),
                                env::consts::EXE_SUFFIX
                            ),
                        )]
                    }
                    DistMode::Gui => {
                        vec![(
                            ["tauri", "build", "-b", "none", "--", "--locked"].as_slice(),
                            format!("rim-gui{}", env::consts::EXE_SUFFIX),
                            format!("{}-installer{}", t!("vendor_en"), env::consts::EXE_SUFFIX),
                        )]
                    }
                    DistMode::Both => {
                        vec![
                            (
                                ["build", "--release", "--locked"].as_slice(),
                                format!("rim-cli{}", env::consts::EXE_SUFFIX),
                                format!(
                                    "{}-installer-cli{}",
                                    t!("vendor_en"),
                                    env::consts::EXE_SUFFIX
                                ),
                            ),
                            (
                                ["tauri", "build", "-b", "none", "--", "--locked"].as_slice(),
                                format!("rim-gui{}", env::consts::EXE_SUFFIX),
                                format!("{}-installer{}", t!("vendor_en"), env::consts::EXE_SUFFIX),
                            ),
                        ]
                    }
                };

                if !matches!(mode, DistMode::Cli) {
                    pnpm_install();
                }

                let dist_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("dist");
                // make sure there is a `dist` folder in root
                std::fs::create_dir_all(&dist_dir)?;

                // Get the target dir
                let dev_bin = current_exe()?;
                let release_dir = dev_bin.parent().unwrap().with_file_name("release");

                // Get additional args
                let num_to_skip = if let DistMode::Both = mode { 2 } else { 3 };
                let adt_args = env::args().skip(num_to_skip).collect::<Vec<_>>();

                for (args, orig_name, new_name) in x {
                    let status = Command::new("cargo").args(args).args(&adt_args).status()?;
                    if status.success() {
                        // copy and rename the binary with vendor name
                        let from = release_dir.join(&orig_name);
                        let to = dist_dir.join(&new_name);
                        std::fs::copy(from, to)?;
                    } else {
                        bail!("build failed");
                    }
                }
            }
            Self::RunManager { no_gui, args } => {
                let mut cargo_args = if *no_gui {
                    ["run", "--"].to_vec()
                } else {
                    ["tauri", "dev", "--"].to_vec()
                };
                cargo_args.extend(args.iter().map(|s| s.as_str()));

                gen_mocked_files()?;

                let status = Command::new("cargo")
                    .args(cargo_args)
                    .env("MODE", "manager")
                    .status()?;
                println!(
                    "\nmanager exited with status code: {}",
                    status.code().unwrap_or(-1)
                );
            }
            Self::SetVendor { vendor } => {
                todo!("change all the `xuanwu` strings to {vendor}");
            }
        }
        Ok(())
    }
}

fn pnpm_install() {
    println!("running `pnpm i`");
    let fail_msg = "unable to run `pnpm i`, \
            please manually cd to `installer/` then run the command manually";

    let gui_crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("installer");
    assert!(gui_crate_dir.exists());

    cfg_if! {
        if #[cfg(windows)] {
            let mut status = Command::new("cmd.exe");
            status.args(["/C", "pnpm", "i"]);
        } else {
            let mut status = Command::new("pnpm");
            status.arg("i");
        }
    }
    status
        .current_dir(gui_crate_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let Ok(st) = status.status() else {
        println!("{fail_msg}");
        return;
    };

    if !st.success() {
        println!("{fail_msg}: {}", st.code().unwrap_or(-1));
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
name = 'XuanWu Rust Distribution (Community)'
version = '1.80.1'
root = '{}'",
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
            let mode = match args.next().as_deref() {
                Some("-h" | "--help") => {
                    writeln!(&mut stdout, "{DIST_HELP}")?;
                    return Ok(ExitCode::SUCCESS);
                }
                Some("--cli") => DistMode::Cli,
                Some("--gui") => DistMode::Gui,
                _ => DistMode::Both,
            };
            DevCmd::Dist { mode }
        }
        "set-vendor" => match args.next().as_deref() {
            Some("-h" | "--help") => {
                writeln!(&mut stdout, "{SET_VENDOR_HELP}")?;
                return Ok(ExitCode::SUCCESS);
            }
            Some(n) => DevCmd::SetVendor {
                vendor: n.to_string(),
            },
            None => {
                writeln!(
                    &mut stdout,
                    "no vendor name provided, usage: 'cargo dev set-vendor [Name]'"
                )?;
                return Ok(ExitCode::FAILURE);
            }
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

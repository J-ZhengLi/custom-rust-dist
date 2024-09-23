#[macro_use]
extern crate rust_i18n;

mod error;
mod installer_mode;
mod manager_mode;
mod toolkit;

use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;

use error::Result;
use rim::cli::{parse_installer_cli, parse_manager_cli, Installer, Manager};
use rim::utils;

i18n!("../../locales", fallback = "en");

static INSTALL_DIR: OnceLock<PathBuf> = OnceLock::new();

enum Mode {
    Manager(Box<Manager>),
    Installer(Box<Installer>),
}

impl Mode {
    /// Determine which mode to run
    fn detect() -> Self {
        let manager_mode = || Mode::Manager(Box::new(parse_manager_cli()));
        let installer_mode = || {
            let cli = parse_installer_cli();
            if let Some(dir) = cli.install_dir() {
                _ = INSTALL_DIR.set(dir.to_path_buf());
            }
            Mode::Installer(Box::new(cli))
        };

        match env::var("MODE").as_deref() {
            Ok("manager") => manager_mode(),
            // fallback to installer mode
            Ok(_) => installer_mode(),
            Err(_) => match utils::lowercase_program_name() {
                Some(s) if s.contains("manager") => manager_mode(),
                // fallback to installer mode
                _ => installer_mode(),
            },
        }
    }

    fn run(&self) -> Result<()> {
        match self {
            Mode::Manager(cli) if cli.no_gui => cli.execute()?,
            Mode::Manager(_) => manager_mode::main()?,
            Mode::Installer(cli) if cli.no_gui => cli.execute()?,
            Mode::Installer(_) => installer_mode::main()?,
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    Mode::detect().run()
}

/// Prevents additional console window on Windows in release
fn hide_console() {
    #[cfg(all(windows, not(debug_assertions)))]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
}

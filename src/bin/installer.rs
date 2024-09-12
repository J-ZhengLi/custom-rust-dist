use anyhow::Result;
use rim::{cli, utils};

enum Mode {
    Manager(Box<cli::Manager>),
    Installer(Box<cli::Installer>),
}

impl Mode {
    /// Determine which mode to run
    fn detect() -> Self {
        let manager_mode = || Mode::Manager(Box::new(cli::parse_manager_cli()));
        let installer_mode = || Mode::Installer(Box::new(cli::parse_installer_cli()));

        match std::env::var("MODE").as_deref() {
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
            Mode::Installer(cli) => cli.execute(),
            Mode::Manager(cli) => cli.execute(),
        }
    }
}

fn main() -> Result<()> {
    Mode::detect().run()
}

// TODO: warn unused before release
#![allow(unused)]

use anyhow::Result;
use cfg_if::cfg_if;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use logger::Logger;

#[cfg(feature = "iced-gui")]
mod app;
#[cfg(feature = "cli")]
mod cli;
mod parser;
mod steps;
mod utils;

mod applog {
    pub use logger::{debug, err, info, trace, warn};
}

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();

fn main() -> Result<()> {
    // initialize logger
    Logger::new().colored().init()?;

    cfg_if! {
        if #[cfg(feature = "iced-gui")] {
            use iced::Application;
            app::App::run(app::default_settings())?;
        } else if #[cfg(feature = "cli")] {
            cli::run()?;
        }
    }

    Ok(())
}

pub(crate) fn config_path() -> &'static Path {
    CONFIG_PATH.get_or_init(|| {
        let installer_home = utils::installer_home();
        // ensure installer home
        std::fs::create_dir_all(&installer_home)
            .expect("aborting because installer home cannot be created");
        installer_home.join("config")
    })
}

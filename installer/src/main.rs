use anyhow::Result;
use cfg_if::cfg_if;

#[cfg(feature = "iced-gui")]
mod app;
#[cfg(feature = "cli")]
mod cli;
mod configuration;
mod utils;
mod steps;

fn main() -> Result<()> {
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

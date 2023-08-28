use anyhow::Result;
use cfg_if::cfg_if;
use logger::Logger;

use rupe::cli;

fn main() -> Result<()> {
    // initialize logger
    Logger::new_with_level(log::LevelFilter::Info)
        .colored()
        .init()?;

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

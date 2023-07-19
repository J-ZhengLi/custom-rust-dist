use anyhow::Result;
use iced::Application;

mod app;

fn main() -> Result<()> {
    app::App::run(app::default_settings())?;
    Ok(())
}

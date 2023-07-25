use anyhow::Result;
use cfg_if::cfg_if;

#[cfg(feature = "iced-gui")]
mod app;

fn main() -> Result<()> {
    cfg_if! {
        if #[cfg(feature = "iced-gui")] {
            use iced::Application;
            app::App::run(app::default_settings())?;
        } else {
            println!("hello");
        }
    }

    Ok(())
}

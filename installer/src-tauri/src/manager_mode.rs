use std::{sync::Arc, thread, time::Duration};

use crate::{error::Result, toolkit::Toolkit};
use anyhow::Context;
use rim::{utils::Progress, UninstallConfiguration};

pub(super) fn main() -> Result<()> {
    super::hide_console();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            super::close_window,
            get_installed_kit,
            get_install_dir,
            uninstall_toolkit,
        ])
        .setup(|app| {
            let version = env!("CARGO_PKG_VERSION");
            tauri::WindowBuilder::new(
                app,
                "manager_window",
                tauri::WindowUrl::App("index.html/#/manager".into()),
            )
            .title(format!("玄武 Rust 管理工具 v{}", version))
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

#[tauri::command]
fn get_installed_kit() -> Result<Option<Toolkit>> {
    let toolkit = Toolkit::from_installed()?;
    println!("installed: {:#?}", &toolkit);

    Ok(toolkit)
}

#[tauri::command]
fn get_install_dir() -> String {
    rim::get_installed_dir().to_string_lossy().to_string()
}

#[tauri::command(rename_all = "snake_case")]
fn uninstall_toolkit(window: tauri::Window, remove_self: bool) -> Result<()> {
    let window = Arc::new(window);
    let window_clone = Arc::clone(&window);

    let uninstall_thread = thread::spawn(move || -> anyhow::Result<()> {
        // Initialize a progress sender.
        let msg_cb = |msg: String| -> anyhow::Result<()> {
            // Note: a small timeout to make sure the message are emitted properly.
            thread::sleep(Duration::from_millis(100));
            Ok(window.emit("update-output", msg)?)
        };
        let pos_cb = |pos: f32| -> anyhow::Result<()> { Ok(window.emit("update-progress", pos)?) };
        let progress = Progress::new(&msg_cb, &pos_cb);

        let config = UninstallConfiguration::init(Some(progress))?;
        config.uninstall(remove_self)?;
        Ok(())
    });

    if uninstall_thread.is_finished() {
        return if let Err(known_error) = uninstall_thread
            .join()
            .expect("unexpected error occurs when processing uninstallation.")
        {
            let error_str = known_error.to_string();
            window_clone
                .emit("uninstall-failed", error_str.clone())
                .expect("failed to emit message");
            Err(known_error.into())
        } else {
            Ok(())
        };
    }

    Ok(())
}

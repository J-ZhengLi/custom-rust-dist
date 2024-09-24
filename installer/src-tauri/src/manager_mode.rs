use std::{
    sync::{mpsc, Arc},
    thread,
};

use crate::{error::Result, toolkit::Toolkit};
use anyhow::Context;
use rim::{utils::MultiThreadProgress, UninstallConfiguration};

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
            let _manager_window = tauri::WindowBuilder::new(
                app,
                "manager_window",
                tauri::WindowUrl::App("index.html/#/manager".into()),
            )
            .title(format!("玄武 Rust 管理工具 v{}", version))
            .build()
            .unwrap();

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
    let (tx_progress, rx_progress) = mpsc::channel();
    let (tx_output, rx_output) = mpsc::channel();

    let uninstall_thread = thread::spawn(move || -> anyhow::Result<()> {
        let mut progress_sender = MultiThreadProgress::new(&tx_output, &tx_progress, 0);
        let config = UninstallConfiguration::init()?;
        config.uninstall_with_progress(remove_self, &mut progress_sender)?;
        Ok(())
    });

    let gui_thread = thread::spawn(move || -> anyhow::Result<()> {
        loop {
            if let Ok(progress) = rx_progress.try_recv() {
                window_clone.emit("update-progress", progress)?;
            }
            if let Ok(detail) = rx_output.try_recv() {
                window_clone.emit("update-output", detail)?;
            }

            if uninstall_thread.is_finished() {
                return if let Err(known_error) = uninstall_thread
                    .join()
                    .expect("unexpected error occurs when processing uninstallation.")
                {
                    let error_str = known_error.to_string();
                    window_clone.emit("uninstall-failed", error_str.clone())?;
                    Err(known_error)
                } else {
                    Ok(())
                };
            }
        }
    });

    if gui_thread.is_finished() {
        gui_thread
            .join()
            .expect("unexpected error occurs when handling toolkit uninstalltion")?;
    }

    Ok(())
}

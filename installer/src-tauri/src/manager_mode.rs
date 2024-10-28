use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use crate::error::Result;
use anyhow::Context;
use rim::toolkit::{get_available_kits_from_server, Toolkit};
use rim::{
    utils::{self, Progress},
    UninstallConfiguration, UpdateConfiguration,
};

pub(super) fn main() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            super::close_window,
            get_installed_kit,
            get_available_kits,
            get_install_dir,
            uninstall_toolkit,
            check_manager_version,
            upgrade_manager,
        ])
        .setup(|app| {
            tauri::WindowBuilder::new(
                app,
                "manager_window",
                tauri::WindowUrl::App("index.html/#/manager".into()),
            )
            .inner_size(800.0, 600.0)
            .min_inner_size(640.0, 480.0)
            .title(format!(
                "{} v{}",
                t!("manager_title", product = t!("product")),
                env!("CARGO_PKG_VERSION")
            ))
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

#[tauri::command]
fn get_installed_kit() -> Result<Option<Toolkit>> {
    Ok(Toolkit::from_installed()?)
}

#[tauri::command]
fn get_available_kits() -> Result<Vec<Toolkit>> {
    let res = get_available_kits_from_server()?;
    println!("available kits: {:#?}", res);
    Ok(res)
}

#[tauri::command]
fn get_install_dir() -> String {
    rim::get_installed_dir().to_string_lossy().to_string()
}

#[tauri::command(rename_all = "snake_case")]
fn uninstall_toolkit(window: tauri::Window, remove_self: bool) -> Result<()> {
    let (msg_sendr, msg_recvr) = mpsc::channel::<String>();
    // config logger to use the `msg_sendr` we just created
    utils::Logger::new().sender(msg_sendr).setup()?;

    let window = Arc::new(window);
    let window_clone = Arc::clone(&window);

    let uninstall_thread = thread::spawn(move || -> anyhow::Result<()> {
        // FIXME: this is needed to make sure the other thread could recieve the first couple messages
        // we sent in this thread. But it feels very wrong, there has to be better way.
        thread::sleep(Duration::from_millis(500));

        let pos_cb = |pos: f32| -> anyhow::Result<()> { Ok(window.emit("update-progress", pos)?) };
        let progress = Progress::new(&pos_cb);

        let config = UninstallConfiguration::init(Some(progress))?;
        config.uninstall(remove_self)?;
        Ok(())
    });

    let gui_thread = spawn_gui_update_thread(window_clone, uninstall_thread, msg_recvr);

    if gui_thread.is_finished() {
        gui_thread.join().expect("failed to join GUI thread")?;
    }
    Ok(())
}

#[tauri::command]
fn check_manager_version() -> bool {
    let check_update_thread = thread::spawn(|| {
        let config: UpdateConfiguration = UpdateConfiguration;
        config.check_upgrade().unwrap_or(false)
    });

    // Join the thread and capture the result, with a default value in case of failure
    check_update_thread.join().unwrap_or_default()
}

#[tauri::command]
fn upgrade_manager() -> Result<()> {
    let upgrade_thread = thread::spawn(move || -> anyhow::Result<()> {
        let config: UpdateConfiguration = UpdateConfiguration;
        config.update(true)
    });

    upgrade_thread.join().expect("failed to upgrade manager.")?;

    Ok(())
}

fn spawn_gui_update_thread(
    win: Arc<tauri::Window>,
    core_thread: thread::JoinHandle<anyhow::Result<()>>,
    msg_recvr: mpsc::Receiver<String>,
) -> thread::JoinHandle<anyhow::Result<()>> {
    thread::spawn(move || loop {
        for pending_message in msg_recvr.try_iter() {
            win.emit("update-output", pending_message)?;
        }

        if core_thread.is_finished() {
            return if let Err(known_error) = core_thread
                .join()
                .expect("failed to join uninstallation thread.")
            {
                let error_str = known_error.to_string();
                win.emit("uninstall-failed", error_str.clone())?;
                Err(known_error)
            } else {
                Ok(())
            };
        }
    })
}

use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use super::Result;
use rim::{
    components::Component,
    toolset_manifest::ToolsetManifest,
    utils::{self, Progress},
    InstallConfiguration,
};

pub(crate) const MESSAGE_UPDATE_EVENT: &str = "update-message";
pub(crate) const PROGRESS_UPDATE_EVENT: &str = "update-progress";
pub(crate) const ON_COMPLETE_EVENT: &str = "on-complete";
pub(crate) const ON_FAILED_EVENT: &str = "on-failed";

/// Configure the logger to use a communication channel ([`mpsc`]),
/// allowing us to send logs accrossing threads.
///
/// This will return a log message's receiver which can be used to emitting
/// messages onto [`tauri::Window`]
pub(crate) fn setup_logger() -> Result<Receiver<String>> {
    let (msg_sendr, msg_recvr) = mpsc::channel::<String>();
    utils::Logger::new().sender(msg_sendr).setup()?;
    Ok(msg_recvr)
}

pub(crate) fn spawn_gui_update_thread(window: tauri::Window, msg_recv: Receiver<String>) {
    thread::spawn(move || loop {
        // Note: `recv()` will block, therefore it's important to check thread execution atfirst
        if let Ok(msg) = msg_recv.recv() {
            if msg.starts_with("error:") {
                emit(&window, ON_FAILED_EVENT, msg);
                break;
            } else {
                emit(&window, MESSAGE_UPDATE_EVENT, msg);
            }
        }
    });
}

fn emit(window: &tauri::Window, event: &str, msg: String) {
    window.emit(event, msg).unwrap_or_else(|e| {
        log::error!(
            "unexpected error occurred \
            while emiting tauri event: {e}"
        )
    });
}

pub(crate) fn install_toolkit_in_new_thread(
    window: tauri::Window,
    components_list: Vec<Component>,
    install_dir: PathBuf,
    manifest: ToolsetManifest,
    is_update: bool,
) {
    thread::spawn(move || -> anyhow::Result<()> {
        // FIXME: this is needed to make sure the other thread could recieve the first couple messages
        // we sent in this thread. But it feels very wrong, there has to be better way.
        thread::sleep(Duration::from_millis(500));

        // Initialize a progress sender.
        let pos_cb =
            |pos: f32| -> anyhow::Result<()> { Ok(window.emit(PROGRESS_UPDATE_EVENT, pos)?) };
        let progress = Progress::new(&pos_cb);

        // TODO: Use continuous progress
        let config =
            InstallConfiguration::new(&install_dir, &manifest)?.progress_indicator(Some(progress));
        if is_update {
            config.update(components_list)?;
        } else {
            config.install(components_list)?;
        }

        // 安装完成后，发送安装完成事件
        window.emit(ON_COMPLETE_EVENT, ())?;

        Ok(())
    });
}

#[derive(serde::Serialize)]
pub struct Language {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub(crate) fn supported_languages() -> Vec<Language> {
    rim::Language::possible_values()
        .iter()
        .map(|lang| {
            let id = lang.as_str();
            match lang {
                rim::Language::EN => Language {
                    id: id.to_string(),
                    name: "English".to_string(),
                },
                rim::Language::CN => Language {
                    id: id.to_string(),
                    name: "简体中文".to_string(),
                },
                _ => Language {
                    id: id.to_string(),
                    name: id.to_string(),
                },
            }
        })
        .collect()
}

#[tauri::command]
pub(crate) fn set_locale(language: String) -> Result<()> {
    let lang: rim::Language = language.parse()?;
    utils::set_locale(lang.locale_str());
    Ok(())
}

/// Add back rounded corners (on Windows) and shadow effects.
///
// TODO: This is not needed if we migrate to tauri@2, also make sure to get rid
// of the `window_shadows` dependency at the time since it adds 6 dependencies in total.
#[allow(unused_variables)]
pub(crate) fn set_window_shadow(window: &tauri::Window) {
    #[cfg(any(windows, target_os = "macos"))]
    if let Err(e) = window_shadows::set_shadow(window, true) {
        log::error!("unable to apply window effects: {e}");
    }
}

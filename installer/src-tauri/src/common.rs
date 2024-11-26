use std::{
    path::PathBuf,
    sync::{mpsc, Arc},
    thread::{self, JoinHandle},
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

pub(crate) fn install_components(
    window: tauri::Window,
    components_list: Vec<Component>,
    install_dir: PathBuf,
    manifest: &ToolsetManifest,
    is_update: bool,
) -> Result<()> {
    let (msg_sendr, msg_recvr) = mpsc::channel::<String>();
    // config logger to use the `msg_sendr` we just created
    utils::Logger::new().sender(msg_sendr).setup()?;

    // 使用 Arc 来共享
    // NB (J-ZhengLi): Threads don't like 'normal' references
    let window = Arc::new(window);
    let window_clone = Arc::clone(&window);
    let manifest = Arc::new(manifest.to_owned());

    // 在一个新线程中执行安装过程
    let install_thread =
        spawn_install_thread(window, components_list, install_dir, manifest, is_update);
    // 在主线程中接收进度并发送事件
    let gui_thread = spawn_gui_update_thread(window_clone, install_thread, msg_recvr);

    if gui_thread.is_finished() {
        gui_thread.join().expect("unable to join GUI thread")?;
    }
    Ok(())
}

fn spawn_install_thread(
    window: Arc<tauri::Window>,
    components_list: Vec<Component>,
    install_dir: PathBuf,
    manifest: Arc<ToolsetManifest>,
    is_update: bool,
) -> JoinHandle<anyhow::Result<()>> {
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
            InstallConfiguration::init(&install_dir, Some(progress), &manifest, is_update)?;
        if is_update {
            config.update(components_list)?;
        } else {
            config.install(components_list)?;
        }

        // 安装完成后，发送安装完成事件
        window.emit(ON_COMPLETE_EVENT, ())?;

        Ok(())
    })
}

pub(crate) fn spawn_gui_update_thread(
    win: Arc<tauri::Window>,
    install_handle: thread::JoinHandle<anyhow::Result<()>>,
    msg_recvr: mpsc::Receiver<String>,
) -> JoinHandle<anyhow::Result<()>> {
    thread::spawn(move || loop {
        for pending_message in msg_recvr.try_iter() {
            win.emit(MESSAGE_UPDATE_EVENT, pending_message)?;
        }

        if install_handle.is_finished() {
            return if let Err(known_error) = install_handle
                .join()
                .expect("unexpected error occurs when attempting to join thread.")
            {
                let error_str = known_error.to_string();
                win.emit(ON_FAILED_EVENT, error_str.clone())?;
                Err(known_error)
            } else {
                Ok(())
            };
        }
    })
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

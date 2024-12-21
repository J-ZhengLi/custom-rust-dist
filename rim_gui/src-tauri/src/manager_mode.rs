use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crate::{
    common::{self, ON_COMPLETE_EVENT, PROGRESS_UPDATE_EVENT},
    error::Result,
};
use anyhow::Context;
use rim::UninstallConfiguration;
use rim::{
    components::Component,
    toolkit::{self, Toolkit},
    toolset_manifest::{get_toolset_manifest, ToolsetManifest},
    update::{self, UpdateOpt},
    utils::{self, Progress},
};
use tauri::{api::dialog, AppHandle, Manager};

static SELECTED_TOOLSET: Mutex<Option<ToolsetManifest>> = Mutex::new(None);

fn selected_toolset<'a>() -> MutexGuard<'a, Option<ToolsetManifest>> {
    SELECTED_TOOLSET
        .lock()
        .expect("unable to lock global mutex")
}

pub(super) fn main() -> Result<()> {
    let msg_recv = common::setup_logger()?;

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            super::close_window,
            get_installed_kit,
            get_available_kits,
            get_install_dir,
            uninstall_toolkit,
            install_toolkit,
            maybe_self_update,
            handle_toolkit_install_click,
            window_title,
            common::supported_languages,
            common::set_locale,
        ])
        .setup(|app| {
            let window = tauri::WindowBuilder::new(
                app,
                "manager_window",
                tauri::WindowUrl::App("index.html/#/manager".into()),
            )
            .inner_size(800.0, 600.0)
            .min_inner_size(640.0, 480.0)
            .decorations(false)
            .transparent(true)
            .build()?;

            common::set_window_shadow(&window);
            common::spawn_gui_update_thread(window, msg_recv);

            Ok(())
        })
        .run(tauri::generate_context!())
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

#[tauri::command]
fn window_title() -> String {
    format!(
        "{} v{}",
        t!("installer_title", product = t!("product")),
        env!("CARGO_PKG_VERSION")
    )
}

#[tauri::command]
fn get_installed_kit(reload: bool) -> Result<Option<Toolkit>> {
    Ok(Toolkit::installed(reload)?.map(|mutex| mutex.lock().unwrap().clone()))
}

#[tauri::command]
fn get_available_kits(reload: bool) -> Result<Vec<Toolkit>> {
    Ok(toolkit::installable_toolkits(reload, false)?
        .into_iter()
        .cloned()
        .collect())
}

#[tauri::command]
fn get_install_dir() -> String {
    rim::get_installed_dir().to_string_lossy().to_string()
}

#[tauri::command(rename_all = "snake_case")]
fn uninstall_toolkit(window: tauri::Window, remove_self: bool) -> Result<()> {
    let window = Arc::new(window);

    thread::spawn(move || -> anyhow::Result<()> {
        // FIXME: this is needed to make sure the other thread could recieve the first couple messages
        // we sent in this thread. But it feels very wrong, there has to be better way.
        thread::sleep(Duration::from_millis(500));

        let pos_cb =
            |pos: f32| -> anyhow::Result<()> { Ok(window.emit(PROGRESS_UPDATE_EVENT, pos)?) };
        let progress = Progress::new(&pos_cb);

        let config = UninstallConfiguration::init(Some(progress))?;
        config.uninstall(remove_self)?;

        window.emit(ON_COMPLETE_EVENT, ())?;
        Ok(())
    });

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
fn install_toolkit(window: tauri::Window, components_list: Vec<Component>) -> Result<()> {
    UpdateOpt::new().update_toolkit(|p| {
        let guard = selected_toolset();
        let manifest = guard
            .as_ref()
            .expect("internal error: a toolkit must be selected to install");
        common::install_toolkit_in_new_thread(
            window,
            components_list,
            p.to_path_buf(),
            manifest.to_owned(),
            true,
        );
        Ok(())
    })?;
    Ok(())
}

#[tauri::command]
fn maybe_self_update(app: AppHandle) -> Result<()> {
    let update_kind = update::check_self_update(false);
    let Some(new_ver) = update_kind.newer_version() else {
        return Ok(());
    };

    dialog::ask(
        app.get_focused_window().as_ref(),
        t!("update_available"),
        t!(
            "ask_self_update",
            latest = new_ver,
            current = env!("CARGO_PKG_VERSION")
        ),
        move |yes| {
            if yes {
                if let Ok(true) = UpdateOpt::new().self_update() {
                    // FIXME: find a way to block main windows interaction
                    app.restart();
                }
            }
        },
    );

    Ok(())
}

/// When the `install` button in a toolkit's card was clicked,
/// the URL of that toolkit was pass to this function. Which will be used to
/// download its manifest from the server, and we can then return a list of
/// components that are loaded from it.
#[tauri::command]
fn handle_toolkit_install_click(url: String) -> Result<Vec<Component>> {
    // the `url` input was converted from `Url`, so it will definitely be convert back without issue,
    // thus the below line should never panic
    let url_ = utils::force_parse_url(&url);

    // load the manifest for components information
    let manifest = get_toolset_manifest(Some(&url_), false)?;
    let components = manifest.current_target_components(false)?;

    // cache the selected toolset manifest
    let mut guard = selected_toolset();
    *guard = Some(manifest);

    Ok(components)
}

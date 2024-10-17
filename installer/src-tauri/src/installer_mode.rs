use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use anyhow::Context;
use indexmap::IndexMap;
use tauri::api::dialog::FileDialogBuilder;

use super::INSTALL_DIR;
use crate::error::Result;
use rim::components::{get_component_list_from_manifest, Component};
use rim::toolset_manifest::{get_toolset_manifest, ToolInfo, ToolsetManifest};
use rim::utils::Progress;
use rim::{try_it, utils, InstallConfiguration};

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();
static TOOLSET_MANIFEST: OnceLock<ToolsetManifest> = OnceLock::new();

pub(super) fn main() -> Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            super::close_window,
            default_install_dir,
            select_folder,
            check_install_path,
            get_component_list,
            install_toolchain,
            run_app,
            welcome_label,
            load_manifest_and_ret_version,
        ])
        .setup(|app| {
            let version = env!("CARGO_PKG_VERSION");
            tauri::WindowBuilder::new(
                app,
                "installer_window",
                tauri::WindowUrl::App("index.html/#/installer".into()),
            )
            .inner_size(800.0, 600.0)
            .min_inner_size(640.0, 480.0)
            .title(format!(
                "{} v{version}",
                t!("installer_title", product = t!("product"))
            ))
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

#[tauri::command]
fn default_install_dir() -> String {
    INSTALL_DIR
        .get()
        .cloned()
        .unwrap_or_else(rim::default_install_dir)
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
fn select_folder(window: tauri::Window) {
    FileDialogBuilder::new().pick_folder(move |path| {
        // 处理用户选择的路径
        let folder = path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        // 通过窗口发送事件给前端
        window.emit("folder-selected", folder).unwrap();
    });
}

/// Check if the given path could be used for installation, and return the reason if not.
#[tauri::command]
fn check_install_path(path: String) -> Option<String> {
    if path.is_empty() {
        Some(t!("notify_empty_path").to_string())
    } else if Path::new(&path).is_relative() {
        // We won't accept relative path because the result might gets a little bit unpredictable
        Some(t!("notify_relative_path").to_string())
    } else if utils::is_root_dir(path) {
        Some(t!("notify_root_dir").to_string())
    } else {
        None
    }
}

/// Get full list of supported components
#[tauri::command]
fn get_component_list() -> Result<Vec<Component>> {
    let manifest = cached_manifest();
    Ok(get_component_list_from_manifest(manifest, false)?)
}

#[tauri::command]
fn welcome_label() -> String {
    t!("welcome", product = t!("product")).into()
}

// Make sure this function is called first after launch.
#[tauri::command]
fn load_manifest_and_ret_version() -> Result<String> {
    // TODO: Give an option for user to specify another manifest.
    // note that passing command args currently does not work due to `windows_subsystem = "windows"` attr
    let mut manifest = get_toolset_manifest(None)?;
    manifest.adjust_paths()?;

    let m = TOOLSET_MANIFEST.get_or_init(|| manifest);
    Ok(m.version.clone().unwrap_or_default())
}

#[tauri::command(rename_all = "snake_case")]
fn install_toolchain(
    window: tauri::Window,
    components_list: Vec<Component>,
    install_dir: String,
) -> Result<()> {
    // 使用 Arc 来共享 window
    let window = Arc::new(window);
    let window_clone = Arc::clone(&window);
    let log_path = log_file_path(&install_dir);
    // FIXME: for some reason, having an existin log file makes other thread fails to read the log,
    // find the cause of it. Until then, let's just remove the existing file for now.
    if log_path.is_file() {
        _ = utils::remove(log_path);
    }

    // 在一个新线程中执行安装过程
    let install_thread = spawn_install_thread(window, components_list, install_dir, log_path)?;
    // 在主线程中接收进度并发送事件
    let gui_thread = spawn_gui_update_thread(window_clone, install_thread, log_path);

    if gui_thread.is_finished() {
        gui_thread.join().expect("unable to join GUI thread")?;
    }
    Ok(())
}

// This spawns a thread that handles installation of user selected components
fn spawn_install_thread(
    win: Arc<tauri::Window>,
    components: Vec<Component>,
    install_dir: String,
    log_path: &'static Path,
) -> Result<thread::JoinHandle<anyhow::Result<()>>> {
    // Split components list to `toolchain_components` and `toolset_components`,
    // as we are running `rustup` to install toolchain components.
    let toolset_components = component_list_to_map(
        components
            .iter()
            .filter(|cm| !cm.is_toolchain_component)
            .collect(),
    );
    let toolchain_components: Vec<String> = components
        .into_iter()
        // Skip the mocked `rust toolchain` component that we added first,
        // it will be installed as requirement anyway.
        .skip(1)
        .filter_map(|comp| {
            if comp.is_toolchain_component {
                Some(comp.name)
            } else {
                None
            }
        })
        .collect();

    Ok(thread::spawn(move || -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(log_path)?;
        // Here we redirect all console output during installation to a buffer
        // Note that `rustup` collect `info:` strings in stderr.
        let drop_with_care = capture_output_to_file(file)?;

        // Initialize a progress sender.
        let msg_cb = |msg: String| -> anyhow::Result<()> {
            // Note: a small timeout to make sure the message are emitted properly.
            thread::sleep(Duration::from_millis(100));
            Ok(win.emit("install-details", msg)?)
        };
        let pos_cb = |pos: f32| -> anyhow::Result<()> { Ok(win.emit("install-progress", pos)?) };
        let progress = Progress::new(&msg_cb, &pos_cb);

        let manifest = cached_manifest();
        // TODO: Use continuous progress
        InstallConfiguration::init(Path::new(&install_dir), false, Some(progress), manifest)?
            .install(toolchain_components, toolset_components)?;

        // Manually drop this, to stop capturing output to file.
        drop(drop_with_care);

        // 安装完成后，发送安装完成事件
        win.emit("install-complete", ())?;

        Ok(())
    }))
}

/// Retrieve cached toolset manifest.
///
/// # Panic
/// Will panic if the manifest is not cached.
fn cached_manifest() -> &'static ToolsetManifest {
    TOOLSET_MANIFEST
        .get()
        .expect("toolset manifest should be loaded by now")
}

fn spawn_gui_update_thread(
    win: Arc<tauri::Window>,
    install_handle: thread::JoinHandle<anyhow::Result<()>>,
    log_path: &'static Path,
) -> thread::JoinHandle<anyhow::Result<()>> {
    let mut existing_log = String::new();
    thread::spawn(move || {
        loop {
            // Install log should be created once the install thread starts running,
            // otherwise we'll keep waiting.
            let mut log_file = if log_path.is_file() {
                fs::OpenOptions::new().read(true).open(log_path)?
            } else {
                continue;
            };

            if let Some(new_content) = get_new_log_content(&mut existing_log, &mut log_file) {
                win.emit("install-details", new_content)?;
            }

            if install_handle.is_finished() {
                return if let Err(known_error) = install_handle
                    .join()
                    .expect("unexpected error occurs when running installation thread.")
                {
                    let error_str = known_error.to_string();
                    win.emit("install-failed", error_str.clone())?;
                    Err(known_error)
                } else {
                    Ok(())
                };
            }

            thread::sleep(Duration::from_millis(50));
        }
    })
}

fn log_file_path(install_dir: &str) -> &'static Path {
    LOG_FILE.get_or_init(|| {
        utils::ensure_dir(install_dir).expect("unable to create install dir");
        PathBuf::from(install_dir).join("install.log")
    })
}

fn get_new_log_content(old_content: &mut String, file: &mut File) -> Option<String> {
    let mut new_content = String::new();
    file.read_to_string(&mut new_content).ok()?;

    if new_content.len() > old_content.len() {
        let new_stuff = new_content[old_content.len()..].to_string();
        *old_content = new_content;
        // TODO: We need some advance rule to filter irrelevant infomation instead.
        let headers = ["info", "warn", "error"];
        let filtered = new_stuff
            .lines()
            .filter(|line| headers.iter().any(|h| line.starts_with(h)))
            .collect::<Vec<_>>()
            .join("\n");
        if !filtered.is_empty() {
            return Some(filtered);
        }
    }

    None
}

fn capture_output_to_file(
    file: File,
) -> anyhow::Result<(gag::Redirect<File>, gag::Redirect<File>)> {
    Ok((
        gag::Redirect::stdout(file.try_clone()?)?,
        gag::Redirect::stderr(file)?,
    ))
}

#[tauri::command(rename_all = "snake_case")]
fn run_app(install_dir: String) -> Result<()> {
    let dir: PathBuf = install_dir.into();
    try_it(Some(&dir))?;
    Ok(())
}

fn component_list_to_map(list: Vec<&Component>) -> IndexMap<String, ToolInfo> {
    let mut map = IndexMap::new();

    for comp in list {
        let (name, tool_info) = (
            comp.name.clone(),
            comp.tool_installer.clone().expect(
                "Internal Error: `component_list_to_map` should only be used on third-party tools",
            ),
        );

        map.insert(name, tool_info);
    }

    map
}

use std::fs::{self, File};
use std::io::{Read, Write};
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
use rim::toolset_manifest::{baked_in_manifest, ToolInfo};
use rim::utils::Progress;
use rim::{try_it, utils, EnvConfig, InstallConfiguration};

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

pub(super) fn main() -> Result<()> {
    super::hide_console();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            super::close_window,
            default_install_dir,
            select_folder,
            get_component_list,
            install_toolchain,
            run_app
        ])
        .setup(|app| {
            let version = env!("CARGO_PKG_VERSION");
            tauri::WindowBuilder::new(
                app,
                "installer_window",
                tauri::WindowUrl::App("index.html/#/installer".into()),
            )
            .title(format!("玄武 Rust 安装工具 v{}", version))
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

#[tauri::command]
fn get_component_list() -> Result<Vec<Component>> {
    // 这里可以放置生成组件列表的逻辑

    // TODO: Download manifest form remote server for online build
    let mut manifest = baked_in_manifest()?;
    manifest.adjust_paths()?;

    Ok(get_component_list_from_manifest(&manifest, false)?)
}

#[tauri::command(rename_all = "snake_case")]
fn install_toolchain(
    window: tauri::Window,
    components_list: Vec<Component>,
    install_dir: String,
) -> Result<()> {
    // Split components list to `toolchain_components` and `toolset_components`,
    // as we are running `rustup` to install toolchain components.
    let toolset_components = component_list_to_map(
        components_list
            .iter()
            .filter(|cm| !cm.is_toolchain_component)
            .collect(),
    );
    let toolchain_components: Vec<String> = components_list
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

    // FIXME: Don't use manifest here, instead, load everything we need to `component`
    let manifest = baked_in_manifest()?;

    // 使用 Arc 来共享 window
    let window = Arc::new(window);
    // 克隆 Arc
    let window_clone = Arc::clone(&window);

    // 在一个新线程中执行安装过程
    let install_thread = thread::spawn(move || -> anyhow::Result<()> {
        let log_file = LOG_FILE.get_or_init(|| PathBuf::from(&install_dir).join("install.log"));
        utils::ensure_parent_dir(log_file)?;
        let file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .read(true)
            .write(true)
            .open(log_file)?;
        // Here we redirect all console output during installation to a buffer
        // Note that `rustup` collect `info:` strings in stderr.
        let drop_with_care = capture_output_to_file(file)?;

        // Initialize a progress sender.
        let msg_cb = |msg: String| -> anyhow::Result<()> {
            // Note: a small timeout to make sure the message are emitted properly.
            thread::sleep(Duration::from_millis(100));
            Ok(window.emit("install-details", msg)?)
        };
        let pos_cb = |pos: f32| -> anyhow::Result<()> { Ok(window.emit("install-progress", pos)?) };
        let progress = Progress::new(&msg_cb, &pos_cb);

        // TODO: Use continuous progress
        let mut config =
            InstallConfiguration::init(Path::new(&install_dir), false, Some(progress))?;
        config.config_env_vars(&manifest)?;
        config.config_cargo()?;
        // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
        config.install_tools(&manifest, &toolset_components)?;
        config.install_rust(&manifest, &toolchain_components)?;
        // install third-party tools via cargo that got installed by rustup
        config.cargo_install(&toolset_components)?;

        // Manually drop this, to tell instruct the thread stop capturing output.
        drop(drop_with_care);

        // 安装完成后，发送安装完成事件
        window.emit("install-complete", ())?;

        Ok(())
    });

    // 在主线程中接收进度并发送事件
    let log_collector_thread = thread::spawn(move || -> anyhow::Result<()> {
        let mut existing_log = String::new();
        loop {
            // Install log should be created once the install thread starts running,
            // otherwise we'll keep waiting.
            let Some(mut log_file) = LOG_FILE.get().and_then(|path| {
                fs::OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(path)
                    .ok()
            }) else {
                continue;
            };

            if let Some(new_content) = get_new_log_content(&mut existing_log, &mut log_file) {
                window_clone.emit("install-details", new_content)?;
            }

            if install_thread.is_finished() {
                return if let Err(known_error) = install_thread
                    .join()
                    .expect("unexpected error occurs when running installation thread.")
                {
                    let error_str = known_error.to_string();

                    // Write this error to log file
                    log_file.write_all(error_str.as_bytes())?;

                    window_clone.emit("install-failed", format!("ERROR: {error_str}"))?;
                    Err(known_error)
                } else {
                    Ok(())
                };
            }

            thread::sleep(Duration::from_millis(50));
        }
    });

    if log_collector_thread.is_finished() {
        log_collector_thread
            .join()
            .expect("unexpected error occurs when handling installation progress")?;
    }

    Ok(())
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

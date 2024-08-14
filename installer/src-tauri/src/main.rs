// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc};
use std::thread;

use custom_rust_dist::manifest::{baked_in_manifest, ToolInfo};
use custom_rust_dist::{EnvConfig, InstallConfiguration};
use tauri::api::dialog::FileDialogBuilder;
use xuanwu_installer::components::{get_component_list_from_manifest, Component};
use xuanwu_installer::Result;

#[tauri::command]
fn finish(window: tauri::Window) {
    window.close().unwrap();
}

#[tauri::command]
fn close_window(window: tauri::Window) {
    // TODO：check and remove cache
    window.close().unwrap();
}

#[tauri::command]
fn default_install_dir() -> String {
    println!("using default install directory");
    custom_rust_dist::default_install_dir()
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
    get_component_list_from_manifest()
}

macro_rules! step {
    ($info_sender:ident, $info:expr, $progress_sender:ident, $progress:expr, $($s:tt)+) => {
        send(&$info_sender, $info);
        $($s)*
        send(&$progress_sender, $progress);
    };
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
        .filter_map(|comp| {
            if comp.is_toolchain_component && comp.name != "Rust" {
                Some(comp.name)
            } else {
                None
            }
        })
        .collect();

    println!("toolsets: {toolset_components:#?}");
    println!("toolchain: {toolchain_components:#?}");

    // TODO: Download manifest form remote server for online build
    let mut manifest = baked_in_manifest()?;
    manifest.adjust_paths()?;

    // 使用 Arc 来共享 window
    let window = Arc::new(window);
    let (tx_progress, rx_progress) = mpsc::channel();
    let (tx_details, rx_details) = mpsc::channel();

    // 在一个新线程中执行安装过程
    {
        let window_clone = Arc::clone(&window); // 克隆 Arc
        thread::spawn(move || -> Result<()> {
            // TODO: Use continuous progress
            step!(
                tx_details,
                format!("Initalizing & Creating directory '{install_dir}'..."),
                tx_progress,
                10,
                let mut config = InstallConfiguration::init(install_dir.into(), false)?;
            );

            step!(
                tx_details,
                "Configuring environment variables...".to_string(),
                tx_progress,
                20,
                config.config_rustup_env_vars()?;
            );

            step!(
                tx_details,
                "Writing cargo configuration...".to_string(),
                tx_progress,
                30,
                config.config_cargo()?;
            );

            // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
            step!(
                tx_details,
                "Installing dependencies & standalone tools...".to_string(),
                tx_progress,
                55,
                config.install_set_of_tools(toolset_components.iter().collect())?;
            );

            step!(
                tx_details,
                "Installing rust toolchain components...".to_string(),
                tx_progress,
                80,
                config.install_rust_with_optional_components(
                    &manifest,
                    Some(toolchain_components.iter().collect()),
                )?;
            );

            // install third-party tools via cargo that got installed by rustup
            step!(
                tx_details,
                "Installing cargo tools...".to_string(),
                tx_progress,
                100,
                config.cargo_install_set_of_tools(toolset_components.iter().collect())?;
            );

            // 安装完成后，发送安装完成事件
            let _ = window_clone.emit("install-complete", ());
            println!(
                "Rust is installed, \
                this setup will soon create an example project at current directory for you to try Rust!"
            );

            Ok(())
        });
    }

    // 在主线程中接收进度并发送事件
    {
        let window_clone = Arc::clone(&window); // 克隆 Arc
        thread::spawn(move || {
            loop {
                // 接收进度
                if let Ok(progress) = rx_progress.recv() {
                    let _ = window_clone.emit("install-progress", progress);
                }
                // 接收详细信息
                if let Ok(detail) = rx_details.recv() {
                    let _ = window_clone.emit("install-details", detail);
                }
            }
        });
    }

    Ok(())
}

fn send<T>(sender: &Sender<T>, msg: T) {
    sender.send(msg).unwrap_or_else(|e| {
        // TODO: Change to error log
        println!("[ERROR] unable to send tx details: {e}");
    });
}

fn component_list_to_map(list: Vec<&Component>) -> BTreeMap<String, ToolInfo> {
    let mut map = BTreeMap::new();

    for comp in list {
        println!("component: {comp:#?}");
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            close_window,
            finish,
            default_install_dir,
            select_folder,
            get_component_list,
            install_toolchain
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

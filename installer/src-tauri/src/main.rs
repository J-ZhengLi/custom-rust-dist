// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use tauri::api::dialog::blocking::FileDialogBuilder;
use xuanwu_installer::components::{get_component_list_from_manifest, Component};
use xuanwu_installer::config::get_default_install_dir;

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
    get_default_install_dir()
}

#[tauri::command]
fn select_folder() -> Option<String> {
    FileDialogBuilder::new()
        .pick_folder()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_component_list() -> Vec<Component> {
    // 这里可以放置生成组件列表的逻辑
    get_component_list_from_manifest()
}

#[tauri::command(rename_all = "snake_case")]
fn install_toolchain(
    window: tauri::Window,
    components_list: Vec<Component>,
    install_dir: String,
) -> Result<(), String> {
    println!("{:?}", components_list);
    println!("{:?}", install_dir);
    // 使用 Arc 来共享 window
    let window = Arc::new(window);
    let (tx_progress, rx_progress) = mpsc::channel();
    let (tx_details, rx_details) = mpsc::channel();

    // 在一个新线程中执行安装过程
    {
        let window_clone = Arc::clone(&window); // 克隆 Arc
        thread::spawn(move || {
            for i in 0..100 {
                // 模拟进度
                thread::sleep(Duration::from_millis(50));

                // 发送进度
                let _ = tx_progress.send(i);

                // 发送详细信息
                let detail_message = format!("正在安装... {}%", i);
                let _ = tx_details.send(detail_message);
            }

            // 安装完成后，发送安装完成事件
            let _ = window_clone.emit("install-complete", ());
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

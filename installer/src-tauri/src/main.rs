// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, OnceLock};
use std::thread;
use std::time::Duration;

use anyhow::Context;
use custom_rust_dist::cli::{parse_cli, CliOpt};
use custom_rust_dist::manifest::{baked_in_manifest, ToolInfo};
use custom_rust_dist::{try_it, utils, EnvConfig, InstallConfiguration};
use tauri::api::dialog::FileDialogBuilder;
use xuanwu_installer::components::{get_component_list_from_manifest, Component};
use xuanwu_installer::Result;

static CLI_ARGS: OnceLock<CliOpt> = OnceLock::new();
static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

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
    CLI_ARGS
        .get()
        .and_then(|opt| opt.install_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(custom_rust_dist::default_install_dir)
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

macro_rules! steps_counter {
    ($($info:expr);+) => {{
        let __dummy_str__ = stringify!($($info;)*);
        __dummy_str__.chars().filter(|c| *c == ';').count()
    }};
}
/// The ultimate macro to perform installation steps.
///
/// The inputs to this macro are:
///
/// - `redir` expression - represent the [`Redirect`] object, which is used to redirect outputs.
/// - `info_sender` ident - the sender variable that is used to send infomation across threads.
/// - `progress_sender` ident - similar to `info_sender`, but sends progress as integer.
/// - (`info`, `step`); - This whole thing is a list of steps to perform.
macro_rules! steps {
    ($redir:expr, $progress_sender:ident, $(($info:expr, $($step:tt)+));+) => {
        let __steps_count__ = steps_counter!($($info);*);
        let __step__ =  100_f32 / __steps_count__ as f32;
        let mut __cur_progress__ = 0_f32;
        $(
            println!("{}", $info);
            $($step)*
            __cur_progress__ += __step__;
            send(&$progress_sender, __cur_progress__.ceil().min(100_f32));
        )*
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

    // TODO: Download manifest form remote server for online build
    let mut manifest = baked_in_manifest()?;
    manifest.adjust_paths()?;

    // 使用 Arc 来共享 window
    let window = Arc::new(window);
    let (tx_progress, rx_progress) = mpsc::channel();

    // 在一个新线程中执行安装过程
    {
        let window_clone = Arc::clone(&window); // 克隆 Arc
        thread::spawn(move || -> anyhow::Result<()> {
            println!("in installation thread");
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

            let init_info = format!("Initalizing & Creating directory '{install_dir}'...");
            let config_info = "Configuring environment variables...".to_string();
            let cargo_config_info = "Writing cargo configuration...".to_string();
            let req_install_info = "Installing dependencies & standalone tools...".to_string();
            let tc_install_info = "Installing rust toolchain components...".to_string();
            let cargo_install_info = "Installing cargo tools...".to_string();

            // TODO: Use continuous progress
            steps! {
                redirect,
                tx_progress,
                (init_info, let mut config = InstallConfiguration::init(install_dir.into(), false)?;);
                (config_info, config.config_rustup_env_vars()?;);
                (cargo_config_info, config.config_cargo()?;);
                // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
                (req_install_info, config.install_set_of_tools(toolset_components.iter().collect())?;);
                (tc_install_info, config.install_rust_with_optional_components(&manifest, Some(toolchain_components.iter().collect()))?;);
                // install third-party tools via cargo that got installed by rustup
                (cargo_install_info, config.cargo_install_set_of_tools(toolset_components.iter().collect())?;)
            };

            // Manually drop this, to tell instruct the thread stop capturing output.
            drop(drop_with_care);

            // 安装完成后，发送安装完成事件
            window_clone.emit("install-complete", ())?;

            Ok(())
        });
    }

    // 在主线程中接收进度并发送事件
    {
        let window_clone = Arc::clone(&window); // 克隆 Arc
        thread::spawn(move || -> anyhow::Result<()> {
            let mut existing_detail = String::new();

            loop {
                // Install log should be created once the installation started,
                // that's where we should start showing progress.
                let Some(mut log_file) = LOG_FILE
                    .get()
                    .and_then(|path| fs::OpenOptions::new().read(true).open(path).ok())
                else {
                    continue;
                };

                // 接收进度
                if let Ok(progress) = rx_progress.recv() {
                    let _ = window_clone.emit("install-progress", progress);
                }
                // 接收详细信息
                // Try reading log file and output it in the detail box
                // FIXME: When running `rustup` or `cargo` to install toolchain or cargo tools,
                // their output was printed in the desired log file, but this thread cannot read them continuously,
                // it appears that this thread was blocked thus unable to update when
                // installing toolchain and cargo tools.
                let mut new_detail = String::new();
                log_file.read_to_string(&mut new_detail)?;
                if new_detail.len() > existing_detail.len() {
                    let detail = &new_detail[existing_detail.len()..];
                    let _ = window_clone.emit("install-details", detail.to_string());
                    existing_detail = new_detail;
                }

                thread::sleep(Duration::from_millis(50));
            }
        });
    }

    Ok(())
}

fn capture_output_to_file(
    file: File,
) -> anyhow::Result<(gag::Redirect<File>, gag::Redirect<File>)> {
    Ok((
        gag::Redirect::stdout(file.try_clone()?)?,
        gag::Redirect::stderr(file)?,
    ))
}

#[tauri::command]
fn run_app(install_dir: Option<String>) -> Result<()> {
    let dir: PathBuf = install_dir.unwrap_or_else(|| default_install_dir()).into();
    try_it(Some(&dir))?;
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

fn main() -> Result<()> {
    let cli = parse_cli();

    if !cli.no_gui {
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                close_window,
                finish,
                default_install_dir,
                select_folder,
                get_component_list,
                install_toolchain,
                run_app
            ])
            .run(tauri::generate_context!())
            .context("error while running tauri application")?;
    } else {
        cli.execute()?;
    }

    Ok(())
}

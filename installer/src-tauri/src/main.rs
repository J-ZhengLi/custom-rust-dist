// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, OnceLock};
use std::time::Duration;
use std::{env, thread};

use anyhow::Context;
use custom_rust_dist::cli::{parse_installer_cli, parse_manager_cli, Installer};
use custom_rust_dist::manifest::{baked_in_manifest, ToolInfo};
use custom_rust_dist::utils::MultiThreadProgress;
use custom_rust_dist::{try_it, utils, EnvConfig, InstallConfiguration};
use indexmap::IndexMap;
use tauri::api::dialog::FileDialogBuilder;
use xuanwu_installer::components::{get_component_list_from_manifest, Component};
use xuanwu_installer::Result;

static CLI_ARGS: OnceLock<Installer> = OnceLock::new();
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
    ($redir:expr, $detail_sender:ident, $progress_sender:ident, $(($info:expr, $p:expr, $($step:tt)+));+) => {
        let __steps_count__ = steps_counter!($($info);*);
        let mut __cur_step__ =  1_usize;
        $(
            println!("{}", &$info);
            send(&$detail_sender, format!("(Step {__cur_step__}/{__steps_count__}) {}", &$info));
            $($step)*;
            if let Some(__prog__) = $p {
                send(&$progress_sender, __prog__);
            }
            __cur_step__ += 1;
        )*
        send(&$progress_sender, 100_usize);
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

    // FIXME: Don't use manifest here, instead, load everything we need to `component`
    let manifest = baked_in_manifest()?;

    // 使用 Arc 来共享 window
    let window = Arc::new(window);
    // 克隆 Arc
    let install_thread_window_clone = Arc::clone(&window);
    let main_thread_window_clone = Arc::clone(&window);

    let (tx_progress, rx_progress) = mpsc::channel();
    let (tx_detail, rx_detail) = mpsc::channel();

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

        let init_info = format!("Initalizing & Creating directory '{install_dir}'...");
        let config_info = "Configuring environment variables...".to_string();
        let cargo_config_info = "Writing cargo configuration...".to_string();
        let req_install_info = "Installing dependencies & standalone tools...".to_string();
        let tc_install_info =
            "Installing rust minimal toolchain and extra components...".to_string();
        let cargo_install_info = "Installing cargo tools...".to_string();

        // Initialize a progress sender.
        // NOTE: the first 10 percent is not sended by this helper struct.
        let mut progress_sender = MultiThreadProgress::new(&tx_detail, &tx_progress, 10);

        // TODO: Use continuous progress
        steps! {
            redirect,
            tx_detail,
            tx_progress,
            (init_info, Some(5), let mut config = InstallConfiguration::init(Path::new(&install_dir), false)?);
            (config_info, Some(7), config.config_rustup_env_vars()?);
            (cargo_config_info, Some(10), config.config_cargo()?);
            // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
            (req_install_info, None, {
                progress_sender.val = 30;
                config.install_set_of_tools(&toolset_components, &mut progress_sender, manifest.proxy.as_ref())?;
            });
            (tc_install_info, None, {
                progress_sender.val = 30;
                config.install_rust_with_optional_components(&manifest, Some(toolchain_components.as_slice()), &mut progress_sender)?;
            });
            // install third-party tools via cargo that got installed by rustup
            (cargo_install_info, None, {
                progress_sender.val = 30;
                config.cargo_install_set_of_tools(&toolset_components, &mut progress_sender)?;
            })
        };

        // Manually drop this, to tell instruct the thread stop capturing output.
        drop(drop_with_care);

        // 安装完成后，发送安装完成事件
        install_thread_window_clone.emit("install-complete", ())?;

        Ok(())
    });

    // 在主线程中接收进度并发送事件
    let gui_update_thread = thread::spawn(move || -> anyhow::Result<()> {
        let mut existing_log = String::new();
        loop {
            // 接收进度
            if let Ok(progress) = rx_progress.try_recv() {
                main_thread_window_clone.emit("install-progress", progress)?;
            }
            if let Ok(detail) = rx_detail.try_recv() {
                main_thread_window_clone.emit("install-details", detail)?;
            }

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
                main_thread_window_clone.emit("install-details", new_content)?;
            }

            if install_thread.is_finished() {
                return if let Err(known_error) = install_thread
                    .join()
                    .expect("unexpected error occurs when running installation thread.")
                {
                    let error_str = known_error.to_string();

                    // Write this error to log file
                    log_file.write_all(error_str.as_bytes())?;

                    main_thread_window_clone
                        .emit("install-failed", format!("ERROR: {error_str}"))?;
                    Err(known_error)
                } else {
                    Ok(())
                };
            }

            thread::sleep(Duration::from_millis(500));
        }
    });

    if gui_update_thread.is_finished() {
        gui_update_thread
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
        if new_stuff.trim().starts_with("info") {
            return Some(new_stuff);
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

fn send<T>(sender: &Sender<T>, msg: T) {
    sender.send(msg).unwrap_or_else(|e| {
        // TODO: Change to error log
        println!("[ERROR] unable to send tx details: {e}");
    });
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

fn main() -> Result<()> {
    match utils::lowercase_program_name() {
        Some(s) if s.starts_with("xuanwu-manager") => {
            let cli = parse_manager_cli();
            if !cli.no_gui {
                // TODO: Add manager UI to manage install toolchain/tools, including
                // update, add, remove, etc...
            } else {
                cli.execute()?;
            }
        }
        _ => {
            // fallback to installer mode
            let cli = parse_installer_cli();
            if !cli.no_gui {
                gui_main()?;
            } else {
                cli.execute()?;
            }
        }
    }

    Ok(())
}

fn gui_main() -> Result<()> {
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
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

#[macro_use]
extern crate rust_i18n;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, OnceLock};
use std::time::Duration;
use std::{env, thread};

use anyhow::Context;
use indexmap::IndexMap;
use tauri::api::dialog::FileDialogBuilder;

use rim::cli::{parse_installer_cli, parse_manager_cli, Installer, Manager};
use rim::manifest::{baked_in_manifest, ToolInfo};
use rim::utils::MultiThreadProgress;
use rim::{
    get_component_list_from_manifest, try_it, utils, Component, EnvConfig, InstallConfiguration,
};
use rim_gui::Result;

static INSTALL_DIR: OnceLock<PathBuf> = OnceLock::new();
static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

i18n!("../../locales", fallback = "en");

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

    Ok(get_component_list_from_manifest(&manifest)?)
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

        let init_info = t!("install_init", dir = install_dir);
        let config_info = t!("install_env_config");
        let cargo_config_info = t!("install_cargo_config");
        let req_install_info = t!("install_tools");
        let tc_install_info = t!("install_toolchain");
        let cargo_install_info = t!("install_via_cargo");

        // Initialize a progress sender.
        // NOTE: the first 10 percent is not sended by this helper struct.
        let mut progress_sender = MultiThreadProgress::new(&tx_detail, &tx_progress, 10);

        // TODO: Use continuous progress
        steps! {
            redirect,
            tx_detail,
            tx_progress,
            (init_info, Some(5), let mut config = InstallConfiguration::init(Path::new(&install_dir), false)?);
            (config_info, Some(7), config.config_env_vars(&manifest)?);
            (cargo_config_info, Some(10), config.config_cargo()?);
            // This step taking cares of requirements, such as `MSVC`, also third-party app such as `VS Code`.
            (req_install_info, None, {
                progress_sender.val = 30;
                config.install_tools_with_progress(&manifest, &toolset_components, &mut progress_sender)?;
            });
            (tc_install_info, None, {
                progress_sender.val = 30;
                config.install_rust_with_progress(&manifest, &toolchain_components, &mut progress_sender)?;
            });
            // install third-party tools via cargo that got installed by rustup
            (cargo_install_info, None, {
                progress_sender.val = 30;
                config.cargo_install_with_progress(&toolset_components, &mut progress_sender)?;
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
    sender
        .send(msg)
        .unwrap_or_else(|e| println!("{}", t!("channel_communicate_err", sum = e)));
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

enum Mode {
    Manager(Box<Manager>),
    Installer(Box<Installer>),
}

impl Mode {
    /// Determine which mode to run
    fn detect() -> Self {
        let manager_mode = || Mode::Manager(Box::new(parse_manager_cli()));
        let installer_mode = || {
            let cli = parse_installer_cli();
            if let Some(dir) = cli.install_dir() {
                _ = INSTALL_DIR.set(dir.to_path_buf());
            }
            Mode::Installer(Box::new(cli))
        };

        match env::var("MODE").as_deref() {
            Ok("manager") => manager_mode(),
            // fallback to installer mode
            Ok(_) => installer_mode(),
            Err(_) => match utils::lowercase_program_name() {
                Some(s) if s.contains("manager") => manager_mode(),
                // fallback to installer mode
                _ => installer_mode(),
            },
        }
    }

    fn run(&self) -> Result<()> {
        match self {
            Mode::Manager(cli) if cli.no_gui => cli.execute()?,
            Mode::Manager(_) => gui_manager()?,
            Mode::Installer(cli) if cli.no_gui => cli.execute()?,
            Mode::Installer(_) => gui_main()?,
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    Mode::detect().run()
}

fn gui_main() -> Result<()> {
    hide_console();

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
        .setup(|app| {
            let version = env!("CARGO_PKG_VERSION");
            let _installer_window = tauri::WindowBuilder::new(
                app,
                "installer_window",
                tauri::WindowUrl::App("index.html/#/installer".into()),
            )
            .title(format!("玄武 Rust 安装工具 v{}", version))
            .build()
            .unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

fn gui_manager() -> Result<()> {
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
        .setup(|app| {
            let version = env!("CARGO_PKG_VERSION");
            let _manager_window = tauri::WindowBuilder::new(
                app,
                "manager_window",
                tauri::WindowUrl::App("index.html/#/manager".into()),
            )
            .title(format!("玄武 Rust 管理工具 v{}", version))
            .build()
            .unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .context("unknown error occurs while running tauri application")?;
    Ok(())
}

/// Prevents additional console window on Windows in release
fn hide_console() {
    #[cfg(all(windows, not(debug_assertions)))]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
}

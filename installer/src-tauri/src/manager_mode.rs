use crate::error::Result;
use anyhow::Context;

pub(super) fn main() -> Result<()> {
    super::hide_console();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
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

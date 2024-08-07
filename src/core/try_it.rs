use crate::{core::install::VSCODE_FAMILY, utils};
use anyhow::Result;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

/// Export an example `cargo` project, then open it with `VSCode` editor or `file explorer`.
pub(crate) fn try_it(path: Option<&Path>) -> Result<()> {
    let path_to_init = if let Some(p) = path {
        p.to_path_buf()
    } else {
        env::current_dir()?
    };

    let example_sources = ExampleTemplate::load();
    // Export the example to user selected location
    let example_dir = example_sources.export(&path_to_init)?;

    // attempts to open the directory with `VS-Code`, if that didn't work
    // open directory using file explorer.
    // **smh** this does not work on devices without desktop environment ofc.

    #[cfg(target_os = "windows")]
    let file_explorer = "explorer.exe";
    #[cfg(target_os = "linux")]
    let file_explorer = "xdg-open";
    #[cfg(target_os = "macos")]
    let file_explorer = "open";

    let mut programs_to_try = VSCODE_FAMILY.to_vec();
    programs_to_try.push(file_explorer);
    for program in programs_to_try {
        let status = Command::new(program).arg(&example_dir).status();
        if matches!(status, Ok(s) if s.success()) {
            return Ok(());
        }
    }

    println!(
        "unable to open example directory with `VSCode` or `file explorer`, \
        try open it manually: {}",
        example_dir.display()
    );
    Ok(())
}

struct ExampleTemplate<'a> {
    src_main: &'a str,
    cargo_toml: &'a str,
    vscode_config: &'a str,
}

impl<'a> ExampleTemplate<'a> {
    fn load() -> Self {
        Self {
            src_main: include_str!("../../templates/example/src/main.rs"),
            cargo_toml: include_str!("../../templates/example/Cargo.toml"),
            vscode_config: include_str!("../../templates/example/.vscode/launch.json"),
        }
    }

    fn export(&self, dest: &Path) -> Result<PathBuf> {
        let root = dest.join("example_project");
        let src_dir = root.join("src");
        let vscode_dir = root.join(".vscode");
        utils::mkdirs(&src_dir)?;
        utils::mkdirs(&vscode_dir)?;

        let main_fp = src_dir.join("main.rs");
        let cargo_toml_fp = root.join("Cargo.toml");
        let vscode_config_fp = vscode_dir.join("launch.json");

        // write source files
        utils::write_file(&main_fp, self.src_main, false)?;
        utils::write_file(&cargo_toml_fp, self.cargo_toml, false)?;
        utils::write_file(&vscode_config_fp, self.vscode_config, false)?;

        Ok(root)
    }
}

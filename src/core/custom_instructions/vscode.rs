//! Custom install method for `Visual Studio Code`.
//! 
//! Because we are using the archive version instead of the official installer,
//! we need to extract it into the tools directory, set path variable with it,
//! and then create a desktop shortcut. The last part is a bit harder to do,
//! there's currently no suitable solution other than execute some commands to hack it.

use std::path::Path;
use crate::core::InstallConfiguration;
use crate::{core::os::add_to_path, utils};
use anyhow::Result;
use dirs::desktop_dir;

#[cfg(windows)]
pub(super) fn install(path: &Path, config: &InstallConfiguration) -> Result<()> {
    // Step 1: Move the root of the directory into `tools` directory
    let vscode_dir = config.tools_dir().join("vscode");
    utils::move_to(path, &vscode_dir)?;
    // Step 2: Add the `bin/` folder to path
    let bin_dir = vscode_dir.join("bin");
    add_to_path(&bin_dir)?;
    // Step 3: Create a desktop shortcut
    // TODO: (?) do we need to create a start menu shortcut as well?
    let shortcut_path = if let Some(dir) = desktop_dir() {
        dir.join("Visual Studio Code.lnk")
    } else {
        println!("unable to determine desktop directory, skip creating desktop shortcut.");
        return Ok(());
    };
    let target_path = vscode_dir.join("Code.exe");
    let weird_powershell_cmd = format!(
        "$s=(New-Object -COM WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.Save()",
        utils::path_to_str(&shortcut_path)?,
        utils::path_to_str(&target_path)?,
    );
    let _ = utils::stdout_output("powershell.exe", &[weird_powershell_cmd])?;

    Ok(())
}

// TODO: Add install instructions for linux
#[cfg(not(windows))]
pub(super) fn install(_path: &Path, _config: &InstallConfiguration) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn uninstall() -> Result<()> {
    Ok(())
}

//! Custom install method for `Visual Studio Code`.
//! 
//! Because we are using the archive version instead of the official installer,
//! we need to extract it into the tools directory, set path variable with it,
//! and then create a desktop shortcut. The last part is a bit harder to do,
//! there's currently no suitable solution other than execute some commands to hack it.

use std::path::Path;
use crate::core::install::InstallConfiguration;
use anyhow::Result;

#[cfg(windows)]
pub(super) fn install(path: &Path, config: &InstallConfiguration) -> Result<()> {
    use crate::{core::os::add_to_path, utils};
    use dirs::desktop_dir;

    // Stop 0: Check if vs-code already exist
    let already_exist = utils::cmd_exist("code");
    if already_exist {
        println!("skipping Visual Studio Code installation, no need to re-install");
        return Ok(());
    }

    // Step 1: Move the root of the directory into `tools` directory
    let vscode_dir = config.tools_dir().join("vscode");
    utils::move_to(path, &vscode_dir, true)?;
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
    utils::execute("powershell.exe", &[weird_powershell_cmd])?;

    Ok(())
}

// TODO: Add install instructions for linux
#[cfg(not(windows))]
pub(super) fn install(_path: &Path, _config: &InstallConfiguration) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn uninstall() -> Result<()> {
    use crate::core::os::install_dir_from_exe_path;
    use crate::core::os::windows::remove_from_path;

    // We've added a path for VSCode at `<InstallDir>/tools/vscode/bin`, try removing it from `PATH`.
    let mut vscode_path = install_dir_from_exe_path()?;
    vscode_path.push("tools");
    vscode_path.push("vscode");
    vscode_path.push("bin");
    remove_from_path(&vscode_path)?;

    // TODO: Remove desktop shortcut and `%USERPROFILE%/.vscode`.
    // We need to see if the shortcut has the correct target before removing it,
    // and we also need to ask user if they want to remove the user profile
    // before doing so, since that folder might be shared with other vscode varients.

    Ok(())
}

#[cfg(not(windows))]
pub(super) fn uninstall() -> Result<()> {
    // TODO: Remove shortcut, remove from PATH
    Ok(())
}

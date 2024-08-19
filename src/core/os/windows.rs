use std::process::Command;

use super::install_dir_from_exe_path;
use crate::core::install::InstallConfiguration;
use crate::core::uninstall::{UninstallConfiguration, Uninstallation};
use crate::core::EnvConfig;
use anyhow::Result;

pub(crate) use rustup::*;

impl EnvConfig for InstallConfiguration {
    fn config_rustup_env_vars(&self) -> Result<()> {
        let vars_raw = self.env_vars()?;
        for (key, val) in vars_raw {
            set_env_var(key, val.encode_utf16().collect())?;
        }

        update_env();

        Ok(())
    }
}

impl Uninstallation for UninstallConfiguration {
    fn remove_rustup_env_vars(&self) -> Result<()> {
        // Remove the `<InstallDir>/.cargo/bin` which is added by rustup
        let mut cargo_bin_dir = install_dir_from_exe_path()?;
        cargo_bin_dir.push(".cargo");
        cargo_bin_dir.push("bin");
        remove_from_path(&cargo_bin_dir)?;

        for var_to_remove in crate::core::ALL_VARS {
            set_env_var(var_to_remove, vec![])?;
        }

        update_env();

        Ok(())
    }

    fn remove_self(&self) -> Result<()> {
        // On windows, we cannot delete an executable that is currently running.
        // Therefore, we are spawning a child process that runs `rmdir` and hope for the best.
        // `rustup` did something like this, it creates a "self-destructable" clone called `rustup-gc`,
        // and it is far more safe than this primitive way of hack, if this method has problem,
        // use the rustup way.
        remove_self_()?;
        Ok(())
    }
}

/// Remove the installation directory, including the binary of this program.
// FIXME: This is such a mess, but it works. However, when uninstall from `control panel`,
// a window flashs, the env vars are removed but nothing has been removed.
fn remove_self_() -> Result<()> {
    /// Execute a command then heads out.
    fn yolo(cmd: &mut Command) -> ! {
        let _yolo = cmd.spawn();
        std::process::exit(0)
    }

    let installed_dir = install_dir_from_exe_path()?;
    let mut rmdir_cmd = Command::new("cmd.exe");
    let cmd = rmdir_cmd
        .args(["/C", "rmdir", "/s", "/q"])
        .arg(&installed_dir);

    do_remove_from_programs(uninstall_entry())?;

    yolo(cmd);
}

/// Module containing functions that are modified from `rustup`.
pub(crate) mod rustup {
    use std::env;
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::Path;
    use std::sync::OnceLock;

    use anyhow::{anyhow, Context, Result};
    use winapi::shared::minwindef;
    use winapi::um::winuser;
    use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

    static UNINSTALL_ENTRY: OnceLock<String> = OnceLock::new();

    pub(super) fn uninstall_entry() -> &'static str {
        UNINSTALL_ENTRY.get_or_init(|| {
            format!(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
                env!("CARGO_PKG_NAME")
            )
        })
    }

    pub(crate) fn do_add_to_programs(bin_dir: &Path) -> Result<()> {
        use std::path::PathBuf;

        let key = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey(uninstall_entry())
            .context("Failed creating uninstall key")?
            .0;

        // Don't overwrite registry if Rustup is already installed
        let prev = key
            .get_raw_value("UninstallString")
            .map(|val| from_winreg_value(&val));
        if let Ok(Some(s)) = prev {
            let mut path = PathBuf::from(OsString::from_wide(&s));
            path.pop();
            if path.exists() {
                return Ok(());
            }
        }

        let cur_exe_path = env::current_exe()?;
        let exe_name = cur_exe_path
            .file_name()
            .unwrap_or_else(|| unreachable!("executable should always have a filename"));
        let path = bin_dir.join(exe_name);

        let mut uninstall_cmd = OsString::from("\"");
        uninstall_cmd.push(path);
        uninstall_cmd.push("\"");

        // FIXME: Remove this if the GUI app supports uninstallation with ui.
        #[cfg(feature = "gui")]
        uninstall_cmd.push(" --no-gui");

        uninstall_cmd.push(" uninstall all");

        let reg_value = RegValue {
            bytes: to_winreg_bytes(uninstall_cmd.encode_wide().collect()),
            vtype: RegType::REG_SZ,
        };

        key.set_raw_value("UninstallString", &reg_value)
            .context("Failed to set `UninstallString`")?;
        key.set_value("DisplayName", &"XuanWu Rust Installation Manager")
            .context("Failed to set `DisplayName`")?;

        Ok(())
    }

    /// This is used to decode the value of HKCU\Environment\PATH. If that key is
    /// not REG_SZ | REG_EXPAND_SZ then this returns None. The winreg library itself
    /// does a lossy unicode conversion.
    fn from_winreg_value(val: &winreg::RegValue) -> Option<Vec<u16>> {
        use std::slice;

        match val.vtype {
            RegType::REG_SZ | RegType::REG_EXPAND_SZ => {
                // Copied from winreg
                let mut words = unsafe {
                    slice::from_raw_parts(val.bytes.as_ptr().cast::<u16>(), val.bytes.len() / 2)
                        .to_owned()
                };
                while words.last() == Some(&0) {
                    words.pop();
                }
                Some(words)
            }
            _ => None,
        }
    }

    /// Convert a vector UCS-2 chars to a null-terminated UCS-2 string in bytes
    fn to_winreg_bytes(mut v: Vec<u16>) -> Vec<u8> {
        v.push(0);
        unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 2).to_vec() }
    }

    pub(crate) fn do_remove_from_programs(entry: &str) -> Result<()> {
        match RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(entry) {
            Ok(()) => Ok(()),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    fn environment() -> Result<RegKey> {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .context("Failed opening Environment key")
    }

    // Get the windows PATH variable out of the registry as a String. If
    // this returns None then the PATH variable is not a string and we
    // should not mess with it.
    pub(super) fn get_windows_path_var() -> Result<Option<Vec<u16>>> {
        let environment = environment()?;

        let reg_value = environment.get_raw_value("PATH");
        match reg_value {
            Ok(val) => {
                if let Some(s) = from_winreg_value(&val) {
                    Ok(Some(s))
                } else {
                    println!(
                        "the registry key HKEY_CURRENT_USER\\Environment\\PATH is not a string. \
                        Not modifying the PATH variable"
                    );
                    Ok(None)
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Some(Vec::new())),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub(super) fn set_env_var(key: &str, val: Vec<u16>) -> Result<()> {
        let env = environment()?;

        if val.is_empty() {
            // Don't do anything if the variable doesn't exist
            if env.get_raw_value(key).is_err() {
                return Ok(());
            }
            env.delete_value(key)?;
        } else {
            // Set for current process
            env::set_var(key, OsString::from_wide(&val));

            let reg_value = RegValue {
                bytes: to_winreg_bytes(val),
                vtype: RegType::REG_EXPAND_SZ,
            };
            // Set for persist user environment
            env.set_raw_value(key, &reg_value)?;
        }

        Ok(())
    }

    /// Broadcast environment changes to other processes,
    /// required after making env changes.
    pub(super) fn update_env() {
        unsafe {
            winuser::SendMessageTimeoutA(
                winuser::HWND_BROADCAST,
                winuser::WM_SETTINGCHANGE,
                0 as minwindef::WPARAM,
                "Environment\0".as_ptr() as minwindef::LPARAM,
                winuser::SMTO_ABORTIFHUNG,
                5000,
                std::ptr::null_mut(),
            );
        }
    }

    pub(crate) fn add_to_path(path: &Path) -> Result<()> {
        let Some(old_path) = get_windows_path_var()? else {
            return Ok(());
        };
        let path_bytes = path.as_os_str().encode_wide().collect::<Vec<_>>();

        let mut new_path = path_bytes;
        new_path.push(b';' as u16);
        new_path.extend_from_slice(&old_path);

        // Apply the new path
        set_env_var("PATH", new_path)?;
        // Sync changes
        update_env();

        Ok(())
    }

    pub(crate) fn remove_from_path(path: &Path) -> Result<()> {
        let Some(old_path) = get_windows_path_var()? else {
            return Ok(());
        };
        let path_bytes = path.as_os_str().encode_wide().collect::<Vec<_>>();

        let Some(idx) = old_path
            .windows(path_bytes.len())
            .position(|path| path == path_bytes)
        else {
            // The path is not added, return without doing anything.
            return Ok(());
        };
        // If there's a trailing semicolon (likely, since we probably added one
        // during install), include that in the substring to remove. We don't search
        // for that to find the string, because if it's the last string in the path,
        // there may not be.
        let mut len = path_bytes.len();
        if old_path.get(idx + path_bytes.len()) == Some(&(b';' as u16)) {
            len += 1;
        }

        let mut new_path = old_path[..idx].to_owned();
        new_path.extend_from_slice(&old_path[idx + len..]);
        // Don't leave a trailing ; though, we don't want an empty string in the
        // path.
        if new_path.last() == Some(&(b';' as u16)) {
            new_path.pop();
        }

        // Apply the new path
        set_env_var("PATH", new_path)?;
        // Sync changes
        update_env();

        Ok(())
    }
}

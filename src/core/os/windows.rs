// FIXME: Too much repeating code, generalize some of the trait methods.

use std::{os::windows::ffi::OsStrExt, path::Path, process::Command};

use crate::{
    core::{
        cargo_config::CargoConfig, InstallConfiguration, Installation, TomlParser,
        UninstallConfiguration, Uninstallation,
    },
    utils,
};
use anyhow::Result;
use winapi::shared::minwindef;
use winapi::um::winuser;

use super::install_dir_from_exe_path;

impl Installation for InstallConfiguration {
    fn config_rustup_env_vars(&self) -> Result<()> {
        let vars_raw = self.env_vars()?;
        for (key, val) in vars_raw {
            rustup::set_env_var(key, val.encode_utf16().collect())?;
        }

        update_env();

        Ok(())
    }

    fn config_cargo(&self) -> Result<()> {
        let mut config = CargoConfig::new();
        if let Some((name, url)) = &self.cargo_registry {
            config.add_source(name, url.to_owned(), true);
        }

        let config_toml = config.to_toml()?;
        if !config_toml.trim().is_empty() {
            // make sure cargo_home dir exists
            let cargo_home = self.cargo_home();
            utils::mkdirs(&cargo_home)?;

            let config_path = cargo_home.join("config.toml");
            utils::write_file(config_path, &config_toml, false)?;
        }

        Ok(())
    }
}

impl Uninstallation for UninstallConfiguration {
    fn remove_rustup_env_vars(&self) -> Result<()> {
        for var_to_remove in crate::core::ALL_VARS {
            rustup::set_env_var(var_to_remove, vec![])?;
        }

        update_env();

        Ok(())
    }

    fn remove_self(&self) -> Result<()> {
        // TODO: Run `rustup self uninstall` first
        // TODO: Remove possibly installed extensions for other software, such as `vs-code` plugins.

        // On windows, we cannot delete an executable that is currently running.
        // Therefore, we are spawning a child process that runs `rmdir` and hope for the best.
        // `rustup` did something like this, it creates a "self-destructable" clone called `rustup-gc`,
        // and it is far more safe than this primitive way of hack, if this method has problem,
        // use the rustup way.
        remove_self_()?;
        Ok(())
    }
}

/// Broadcast environment changes to other processes,
/// required after making env changes.
fn update_env() {
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

    rustup::do_remove_from_programs()?;

    yolo(cmd);
}

pub(super) fn add_to_path(path: &Path) -> Result<()> {
    let Some(old_path) = rustup::get_windows_path_var()? else {
        return Ok(());
    };
    let path_bytes = path.as_os_str().encode_wide().collect::<Vec<_>>();
    let mut new_path = path_bytes;
    new_path.push(b';' as u16);
    new_path.extend_from_slice(&old_path);

    // Apply the new path
    rustup::set_env_var("PATH", new_path)?;

    // Sync changes
    update_env();

    Ok(())
}

pub(crate) mod rustup {
    use std::env;
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::sync::OnceLock;

    use anyhow::{anyhow, Context, Result};
    use winreg::enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::{RegKey, RegValue};

    static UNINSTALL_ENTRY: OnceLock<String> = OnceLock::new();

    fn uninstall_entry() -> &'static str {
        UNINSTALL_ENTRY.get_or_init(|| {
            format!(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{}",
                env!("CARGO_PKG_NAME")
            )
        })
    }

    pub(crate) fn do_add_to_programs() -> Result<()> {
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

        let path = env::current_exe()?;
        let mut uninstall_cmd = OsString::from("\"");
        uninstall_cmd.push(path);
        uninstall_cmd.push("\" uninstall all");

        let reg_value = RegValue {
            bytes: to_winreg_bytes(uninstall_cmd.encode_wide().collect()),
            vtype: RegType::REG_SZ,
        };

        key.set_raw_value("UninstallString", &reg_value)
            .context("Failed to set `UninstallString`")?;
        key.set_value("DisplayName", &"Rust installation manager")
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

    pub(super) fn do_remove_from_programs() -> Result<()> {
        match RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(uninstall_entry()) {
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
            let reg_value = RegValue {
                bytes: to_winreg_bytes(val),
                vtype: RegType::REG_EXPAND_SZ,
            };
            env.set_raw_value(key, &reg_value)?;
        }

        Ok(())
    }
}

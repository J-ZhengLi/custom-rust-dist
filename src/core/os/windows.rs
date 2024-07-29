// FIXME: Too much repeating code, generalize some of the trait methods.

use std::process::Command;

use crate::{
    core::{
        cargo_config::CargoConfig, InstallConfiguration, Installation, TomlParser,
        UninstallConfiguration, Uninstallation,
    },
    utils,
};
use anyhow::{bail, Result};
use winapi::shared::minwindef;
use winapi::um::winuser;

use super::{ensure_init_call, install_dir_from_exe_path, INIT_ONCE};

impl Installation for InstallConfiguration {
    fn init(&self) -> Result<()> {
        // Create a new folder to hold installation
        let folder = &self.install_dir;
        utils::mkdirs(folder)?;

        // Create a copy of this binary to install dir
        let self_exe = std::env::current_exe()?;
        let cargo_bin_dir = self.cargo_home().join("bin");
        utils::mkdirs(&cargo_bin_dir)?;
        utils::copy_to(self_exe, &cargo_bin_dir)?;

        // Create registry entry to add this program into "installed programs".
        rustup::do_add_to_programs()?;

        INIT_ONCE.get_or_init(|| ());
        Ok(())
    }

    fn config_rustup_env_vars(&self) -> Result<()> {
        ensure_init_call();

        let vars_raw = self.env_vars()?;
        for (key, val) in vars_raw {
            // Setting env var by calling `setx` command.
            // NB: If `setx` ever causes problem, replace this with a much safer way,
            // which is using `winapi`+`winreg` like what rustup does.
            // Note calling `setx` and `reg` is significantly slower than using windows api.
            let output = utils::output("setx", &[key, &val])?;
            if !output.status.success() {
                bail!(
                    "unable to set environment variables: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        update_env();

        Ok(())
    }

    fn config_cargo(&self) -> Result<()> {
        ensure_init_call();

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
    // On windows, we are calling `reg delete` to remove a specific variable
    // that we have set on user environment.
    // NB: If `reg delete` ever causes problem, replace this with a much safer way,
    // which is using `winapi`+`winreg` like what rustup does.
    fn remove_rustup_env_vars(&self) -> Result<()> {
        for var_to_remove in crate::core::ALL_VARS {
            let query_output = utils::output(
                "reg",
                &[
                    "query",
                    "HKEY_CURRENT_USER\\Environment",
                    "/v",
                    var_to_remove,
                ],
            )?;
            // Remove only if it could be found, meaning we did actually set it.
            if query_output.status.success() {
                let _ = utils::stdout_output(
                    "reg",
                    &[
                        "delete",
                        "HKEY_CURRENT_USER\\Environment",
                        "/f",
                        "/v",
                        var_to_remove,
                    ],
                )?;
            }
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

mod rustup {
    use std::env;
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::sync::OnceLock;

    use anyhow::{anyhow, Context, Result};
    use winreg::enums::{RegType, HKEY_CURRENT_USER};
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

    pub(super) fn do_add_to_programs() -> Result<()> {
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
}

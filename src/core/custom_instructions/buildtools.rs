use std::path::{Path, PathBuf};
use anyhow::Result;
use cc::windows_registry;
use crate::core::directories::RimDir;
use crate::core::install::InstallConfiguration;

pub(super) fn install(path: &Path, config: &InstallConfiguration) -> Result<Vec<PathBuf>> {
    use std::path::PathBuf;
    use crate::utils;
    use anyhow::anyhow;

    fn any_existing_child_path(root: &Path, childs: &[&str]) -> Option<PathBuf> {
        childs.iter().find_map(|child| {
            let child_path = root.join(child);
            child_path.exists().then_some(child_path)
        })
    }
    
    // VS Build Tools changed their installer binary name to `CamelCase` at some point.
    let buildtools_exe = any_existing_child_path(path, &["vs_BuildTools.exe", "vs_buildtools.exe"])
        .ok_or_else(|| anyhow!("unable to find the build tools installer binary."))?;

    let mut cmd = vec![
        "--wait",
        "--noWeb",
        "--nocache",
        "--passive",
        "--norestart",
        "--focusedUi",
    ];
    for component in required_components() {
        cmd.push("--add");
        cmd.push(component.component_id());
    }

    // Step 2: Make a copy of this installer to the `tools` directory,
    // which is later used for uninstallation.
    let installer_dir = config.tools_dir().join("buildtools");
    utils::ensure_dir(&installer_dir)?;
    utils::copy_file_to(&buildtools_exe, &installer_dir)?;

    // Step 3: Invoke the install command.
    println!("{}", t!("installing_msvc_info"));
    let exit_code: VSExitCode = utils::execute_for_ret_code(buildtools_exe, &cmd)?.into();
    match exit_code {
        VSExitCode::Success => {
            println!("info: {}", exit_code);
        }
        VSExitCode::RebootRequired | VSExitCode::RebootInitiated => {
            println!("warn: {}", exit_code);
        }
        _ => {
            return Err(anyhow!("unable to install VS buildtools: {}", exit_code));
        }
    }
    Ok(vec![installer_dir])
}

pub(super) fn uninstall(_config: &crate::core::uninstall::UninstallConfiguration) -> Result<()> {
    // TODO: Navigate to the vs_buildtools exe that we copied when installing, then execute it with:
    // .\vs_BuildTools.exe uninstall --productId Microsoft.VisualStudio.Product.BuildTools --channelId VisualStudio.17.Release --wait
    // But we need to ask the user if they want to uninstall this or not.
    Ok(())
}

pub(super) fn already_installed() -> bool {
    // Other targets don't need MSVC, so assume it has already installed
    if !env!("TARGET").contains("msvc") {
        return true;
    }

    windows_registry::find_tool(env!("TARGET"), "cl.exe").is_some()
}

#[derive(Debug, Clone, Copy)]
enum BuildToolsComponents {
    Msvc,
    WinSDK,
}

impl BuildToolsComponents {
    // FIXME: (?) Id might change depending on the version etc.
    fn component_id(&self) -> &'static str {
        use BuildToolsComponents::*;
        match self {
            Msvc => "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            WinSDK => "Microsoft.VisualStudio.Component.Windows11SDK.22000",
        }
    }
}

fn required_components() -> Vec<BuildToolsComponents> {
    let is_windows_sdk_installed = if let Some(paths) = std::env::var_os("lib") {
        std::env::split_paths(&paths)
            .any(|path| {
                path.join("kernel32.lib").exists()
            })
    } else {
        false
    };

    if is_windows_sdk_installed {
        vec![BuildToolsComponents::Msvc]
    } else {
        vec![BuildToolsComponents::Msvc, BuildToolsComponents::WinSDK]
    }
}

macro_rules! integer_enum_with_fallback {
    ($name:ident ( $itype:ty ) { $fallback_var:ident : $fs:expr, $($varient:ident : $s:expr => ($($val:tt)+)),+ }) => {
        #[non_exhaustive]
        enum $name {
            $fallback_var($itype),
            $($varient),+
        }
        impl From<$itype> for $name {
            fn from(value: $itype) -> $name {
                match value {
                    $(
                        $($val)+ => $name::$varient,
                    )*
                    n => $name::$fallback_var(n),
                }
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use $name::*;
                match self {
                    $(
                        $varient => write!(f, "{}", $s),
                    )*
                    $fallback_var(n) => write!(f, "{} {n}", $fs)
                }
            }
        }
    };
}

integer_enum_with_fallback! {
    VSExitCode (i32) {
        Unknown: "Unknown error",
        Success: "Operation completed successfully" => (0),
        RequireElevation: "Elevation required" => (740),
        VSInstallerRunning: "Visual Studio installer process is running" => (1001),
        VSProcessRunning: "Visual Studio processes are running" => (8006),
        VSInUse: "Visual Studio is in use" => (1003),
        VSInstallerTerminated: "Microsoft Visual Studio Installer was terminated (by the user or external process)" => (-1073741510),
        AnotherInstallerRunning: "Another installation running" => (1618),
        RebootInitiated: "Operation completed successfully, and reboot was initiated" => (1641),
        RebootRequired: "Operation completed successfully, but install requires reboot before it can be used" => (3010),
        ArgParseError: "Bootstrapper command-line parse error" => (5005),
        OperationCanceled: "Operation was canceled" => (1602 | 5004),
        OperationBlocked: "Operation was blocked - the computer does not meet the requirements" => (5007),
        ArmCheckFailure: "Arm machine check failure" => (8001),
        DownloadPrecheckFailure: "Background download precheck failure" => (8002),
        ComponentOutOfSupport: "Out of support selectable failure" => (8003),
        TargetDirectoryFailure: "Target directory failure" => (8004),
        PayloadVerifyFailure: "Verifying source payloads failure" => (8005),
        UnsupportedOS: "Operating System not supported" => (8010),
        ConnectivityFailure: "Connectivity failure" => (-1073720687)
    }
}

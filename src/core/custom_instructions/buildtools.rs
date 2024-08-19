use std::path::Path;
use anyhow::Result;
use crate::core::install::InstallConfiguration;

#[cfg(windows)]
pub(super) fn install(path: &Path, config: &InstallConfiguration) -> Result<()> {
    use std::path::PathBuf;
    use crate::utils;
    use anyhow::{anyhow, bail};

    fn any_existing_child_path(root: &Path, childs: &[&str]) -> Option<PathBuf> {
        for child in childs {
            let child_path = root.join(child);
            if child_path.exists() {
                return Some(child_path);
            }
        }
        None
    }

    // Step 1: Check if user has already installed any of the two MSVC component.
    // FIXME: This should be checked before extraction instead.
    let missing_components = windows_related::get_missing_build_tools_components();

    // Step 2: Make an install command, or no command if everything are already installed.
    // TODO: Check version to make sure the newest version are installed.
    if missing_components.is_empty() {
        println!("skipping build tools installation, no need to re-install");
        return Ok(());
    }
    // VS Build Tools changed their installer binary name to `CamelCase` at some point.
    let buildtools_exe = any_existing_child_path(path, &["vs_BuildTools.exe", "vs_buildtools.exe"])
        .ok_or_else(|| anyhow!("unable to find the build tools installer binary."))?;

    let mut cmd = vec![
        "--wait",
        "--noWeb",
        "--nocache",
        "--passive",
        "--focusedUi",
    ];
    for component in missing_components {
        cmd.push("--add");
        cmd.push(component.component_id());
    }

    // Step 2.5: Make a copy of this installer to the `tools` directory,
    // which is later used for uninstallation.
    let installer_dir = config.tools_dir().join("buildtools");
    utils::mkdirs(&installer_dir)?;
    utils::copy_file_to(&buildtools_exe, &installer_dir)?;

    // Step 3: Invoke the install command.
    println!("running VS BuildTools installer...");
    utils::execute(buildtools_exe, &cmd)?;

    Ok(())
}

#[allow(unused)]
#[cfg(not(windows))]
pub(super) fn install(_path: &Path, _config: &InstallConfiguration) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn _uninstall() -> Result<()> {
    // TODO: Navigate to the vs_buildtools exe that we copied when installing, then execute it with:
    // .\vs_BuildTools.exe uninstall --productId Microsoft.VisualStudio.Product.BuildTools --channelId VisualStudio.17.Release --wait
    Ok(())
}

#[allow(unused)]
#[cfg(not(windows))]
pub(super) fn _uninstall() -> Result<()> {
    Ok(())
}

#[cfg(windows)]
// TODO: move these code that are copied... *ahem* inspired from `rustup` into `utils`
mod windows_related {
    use crate::utils::HostTriple;
    use cc::windows_registry;

    #[derive(Debug, Clone, Copy)]
    pub(crate) enum BuildToolsComponents {
        Msvc,
        WinSDK,
    }

    impl BuildToolsComponents {
        // FIXME: (?) Id might change depending on the version etc.
        pub(crate) fn component_id(&self) -> &'static str {
            use BuildToolsComponents::*;
            match self {
                Msvc => "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                WinSDK => "Microsoft.VisualStudio.Component.Windows11SDK.22000",
            }
        }
    }

    pub(crate) fn get_missing_build_tools_components() -> Vec<BuildToolsComponents> {
        let host_triple = HostTriple::from_host().map_or_else(String::new, |t| t.to_string());
        let installing_msvc = host_triple.contains("msvc");

        if !installing_msvc {
            return vec![];
        }

        let mut missing_comps = vec![];

        let have_msvc = windows_registry::find_tool(&host_triple, "cl.exe").is_some();
        if !have_msvc  {
            missing_comps.push(BuildToolsComponents::Msvc);
        }

        let have_windows_sdk_libs = || {
            if let Some(paths) = std::env::var_os("lib") {
                for mut path in std::env::split_paths(&paths) {
                    path.push("kernel32.lib");
                    if path.exists() {
                        return true;
                    }
                }
            }
            false
        };
        if !have_windows_sdk_libs() {
            missing_comps.push(BuildToolsComponents::WinSDK);
        }

        missing_comps
    }
}

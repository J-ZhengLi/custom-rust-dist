use std::path::Path;
use anyhow::Result;
use crate::core::install::InstallConfiguration;

#[cfg(windows)]
pub(super) fn install(path: &Path, config: &InstallConfiguration) -> Result<()> {
    use std::path::PathBuf;
    use crate::utils;
    use anyhow::anyhow;

    fn any_existing_child_path(root: &Path, childs: &[&str]) -> Option<PathBuf> {
        fn inner_(root: &Path, childs: &[&str]) -> Option<PathBuf> {
            childs.iter().find_map(|child| {
                let child_path = root.join(child);
                child_path.exists().then_some(child_path)
            })
        }

        if let Some(found) = inner_(root, childs) {
            Some(found)
        } else {
            // Keep looking in sub dir.
            // TODO: This is due to the fact that we have poor zip extraction function atm.
            // Since it doesn't skip common prefix, we have to manually look for matches
            // by getting into sub directories at depth 1. Delete this branch once it can skip prefix.
            let Ok(entries) = utils::walk_dir(root, false) else { return None };
            for sub_dir in entries.iter().filter(|p| p.is_dir()) {
                if let Some(found) = inner_(sub_dir.as_path(), childs) {
                    return Some(found);
                }
            }
            None
        }
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
    for component in windows_related::required_components() {
        cmd.push("--add");
        cmd.push(component.component_id());
    }

    // Step 2: Make a copy of this installer to the `tools` directory,
    // which is later used for uninstallation.
    let installer_dir = config.tools_dir().join("buildtools");
    utils::ensure_dir(&installer_dir)?;
    utils::copy_file_to(&buildtools_exe, &installer_dir)?;

    // Step 3: Invoke the install command.
    println!("running VS BuildTools installer...");
    utils::execute(buildtools_exe, &cmd)?;

    Ok(())
}

#[cfg(not(windows))]
pub(super) fn install(_path: &Path, _config: &InstallConfiguration) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn uninstall() -> Result<()> {
    // TODO: Navigate to the vs_buildtools exe that we copied when installing, then execute it with:
    // .\vs_BuildTools.exe uninstall --productId Microsoft.VisualStudio.Product.BuildTools --channelId VisualStudio.17.Release --wait
    // But we need to ask the user if they want to uninstall this or not.
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn uninstall() -> Result<()> {
    Ok(())
}

#[cfg(windows)]
pub(super) fn already_installed() -> bool {
    windows_related::is_msvc_installed()
}

#[cfg(not(windows))]
pub(super) fn already_installed() -> bool {
    true
}

#[cfg(windows)]
// TODO: move these code that are copied... *ahem* inspired from `rustup` into `utils`
mod windows_related {
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

    pub(super) fn is_msvc_installed() -> bool {
        // Other targets don't need MSVC, so assume it has already installed
        if !env!("TARGET").contains("msvc") {
            return true;
        }

        windows_registry::find_tool(env!("TARGET"), "cl.exe").is_some()
    }

    fn is_windows_sdk_installed() -> bool {
        if let Some(paths) = std::env::var_os("lib") {
            std::env::split_paths(&paths)
                .any(|path| {
                    path.join("kernel32.lib").exists()
                })
        } else {
            false
        }
    }

    pub(crate) fn required_components() -> Vec<BuildToolsComponents> {
        if is_windows_sdk_installed() {
            vec![BuildToolsComponents::Msvc]
        } else {
            vec![BuildToolsComponents::Msvc, BuildToolsComponents::WinSDK]
        }
    }
}

use std::{cell::OnceCell, collections::HashMap};

use crate::utils;

macro_rules! declare_instrcutions {
    ($($name:ident),+) => {
        $(pub(crate) mod $name;)*
        pub(crate) static SUPPORTED_TOOLS: &[&str] = &[$(stringify!($name)),+];

        pub(crate) fn install(tool: &str, path: &std::path::Path, config: &super::install::InstallConfiguration) -> anyhow::Result<Vec<std::path::PathBuf>> {
            match tool.replace('-', "_").as_str() {
                $(
                    stringify!($name) => $name::install(path, config),
                )*
                _ => anyhow::bail!("no custom install instruction for '{tool}'")
            }
        }

        pub(crate) fn uninstall(tool: &str, config: &super::uninstall::UninstallConfiguration) -> anyhow::Result<()> {
            match tool.replace('-', "_").as_str() {
                $(
                    stringify!($name) => $name::uninstall(config),
                )*
                _ => anyhow::bail!("no custom uninstall instruction for '{tool}'")
            }
        }

        fn supported_tool_is_installed(tool: &str) -> bool {
            match tool.replace('-', "_").as_str() {
                $(
                    stringify!($name) => $name::already_installed(),
                )*
                // Is not supported, assume not installed for now
                _ => false
            }
        }
    };
}

#[cfg(windows)]
declare_instrcutions!(buildtools, vscode);
#[cfg(not(windows))]
declare_instrcutions!(vscode);

pub(crate) fn is_supported(name: &str) -> bool {
    SUPPORTED_TOOLS.contains(&name.replace('-', "_").as_str())
}

/// Checking if a certain tool is installed by:
///
/// 1. If it has it's on module, it should be detemined there, see list: [`SUPPORTED_TOOLS`].
/// 2. Looking up the same name in path.
/// 3. Looking up a pre-defined list related to the given tool, to see if
///     those are all in the path.
pub(crate) fn is_installed(name: &str) -> bool {
    if supported_tool_is_installed(name) || utils::cmd_exist(utils::exe!(name)) {
        return true;
    }

    // This is a map with toolname and a list of programs to check.
    // Since the list to check is highly rely on tool's name, let's calling it `semi-supported` for now.
    let semi_supported_tools = OnceCell::new();
    let programs = semi_supported_tools
        .get_or_init(|| HashMap::from([("mingw64", &["gcc", "ld"])]))
        .get(name);
    if let Some(list) = programs {
        list.iter().all(|p| utils::cmd_exist(utils::exe!(p)))
    } else {
        // Still have no idea, assuming not installed
        false
    }
}

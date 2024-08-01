use super::{manifest::ToolInfo, InstallConfiguration, CARGO_HOME, RUSTUP_HOME};
use crate::{
    core::install_instructions,
    utils::{self, Extractable},
};
use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// TODO: Write version info after installing each tool,
// which is later used for updating.
pub(crate) fn install_tool(
    config: &InstallConfiguration,
    name: &str,
    tool: &ToolInfo,
) -> Result<()> {
    match tool {
        ToolInfo::Version(ver) if config.cargo_is_installed => {
            let output = utils::output_with_env(
                "cargo",
                &["install", name, "--version", ver],
                [
                    (CARGO_HOME, utils::path_to_str(&config.cargo_home())?),
                    (RUSTUP_HOME, utils::path_to_str(&config.rustup_home())?),
                ],
            )?;
            utils::forward_output(output)?;
        }
        ToolInfo::Git {
            git,
            branch,
            tag,
            rev,
        } if config.cargo_is_installed => {
            let branch_opt = branch
                .as_ref()
                .map(|s| format!("--branch {s}"))
                .unwrap_or_default();
            let tag_opt = tag
                .as_ref()
                .map(|s| format!("--tag {s}"))
                .unwrap_or_default();
            let rev_opt = rev
                .as_ref()
                .map(|s| format!("--rev {s}"))
                .unwrap_or_default();

            let output = utils::output_with_env(
                "cargo",
                &[
                    "install",
                    "--git",
                    git.as_str(),
                    &branch_opt,
                    &tag_opt,
                    &rev_opt,
                ],
                [
                    (CARGO_HOME, utils::path_to_str(&config.cargo_home())?),
                    (RUSTUP_HOME, utils::path_to_str(&config.rustup_home())?),
                ],
            )?;
            utils::forward_output(output)?;
        }
        ToolInfo::Path { path, .. } => try_install_from_path(config, name, path)?,
        // FIXME: Have a dedicated download folder, do not use temp dir to store downloaded artifacts,
        // so then we can have the `resume download` feature.
        ToolInfo::Url { url, .. } => {
            // TODO: Download first
            let temp_dir = create_temp_dir(config, "download")?;

            let downloaded_file_name = url
                .path_segments()
                .ok_or_else(|| anyhow!("unsupported url format '{url}'"))?
                .last()
                // Sadly, a path segment could be empty string, so we need to filter that out
                .filter(|seg| !seg.is_empty())
                .ok_or_else(|| anyhow!("'{url}' doesn't appear to be a downloadable file"))?;

            let dest = temp_dir.path().join(downloaded_file_name);

            utils::download_from_start(name, url, &dest)?;
            // TODO: Then do the `extract or copy to` like `ToolInfo::Path`
            try_install_from_path(config, name, &dest)?;
        }
        // Don't try to install tools that requires `cargo install` if `cargo` isn't even installed.
        _ => (),
    }
    Ok(())
}

fn try_install_from_path(config: &InstallConfiguration, name: &str, path: &Path) -> Result<()> {
    let temp_dir = create_temp_dir(config, name)?;

    let tool_installer_path = extract_or_copy_to(path, temp_dir.path())?;

    let tool_installer = ToolInstaller::from_path(name, &tool_installer_path)
        .with_context(|| format!("no install method for tool '{name}'"))?;
    tool_installer.install(config)?;
    Ok(())
}

fn create_temp_dir(config: &InstallConfiguration, prefix: &str) -> Result<TempDir> {
    let root = config.temp_root();
    // Ensure temp directory
    utils::mkdirs(&root)?;

    tempfile::Builder::new()
        .prefix(&format!("{prefix}_"))
        .tempdir_in(&root)
        .with_context(|| format!("unable to create temp directory under '{}'", root.display()))
}

/// Perform extraction or copy action base on the given path.
///
/// If `maybe_file` is a path to compressed file, this will try to extract it to `dest`;
/// otherwise this will copy that file into dest.
fn extract_or_copy_to(maybe_file: &Path, dest: &Path) -> Result<PathBuf> {
    if let Ok(extractable) = Extractable::try_from(maybe_file) {
        extractable.extract_to(dest)?;
        Ok(dest.to_path_buf())
    } else {
        utils::copy_to(maybe_file, dest)
    }
}

/// Representing the structure of an (extracted) tool's directory.
enum ToolInstaller<'a> {
    /// Pre-built executable files.
    /// i.e.:
    /// ```text
    /// ├─── some_binary.exe
    /// ├─── cargo-some_binary.exe
    /// ```
    Executables(Vec<PathBuf>),
    /// Plugin file, such as `.vsix` files for Visual Studio.
    // TODO: Have a plugin kind, instead of just for visual studio.
    Plugin(&'a Path),
    /// Directory containing `bin` subfolder:
    /// ```text
    /// tool/
    /// ├─── bin/
    /// ├─── ...
    /// ```
    /// Path to `root` representing the `tool/` directory. Which is the dir
    /// containing `bin/`.
    DirWithBin { root: &'a Path, bin_dir: PathBuf },
    /// We have a custom "script" for how to deal with such directory.
    Custom(&'a str),
}

impl<'a> ToolInstaller<'a> {
    fn from_path(name: &'a str, path: &'a Path) -> Result<Self> {
        // Step 1: Looking for custom install instruction
        if install_instructions::SUPPORTED_TOOLS.contains(&name) {
            return Ok(Self::Custom(name));
        }

        // Step 2: Identify from file extension (if it's a file ofc).
        if utils::is_executable(path) {
            return Ok(Self::Executables(vec![path.to_path_buf()]));
        } else if path.is_file() {
            let maybe_extension = path.extension();
            if let Some(ext) = maybe_extension.and_then(|ext| ext.to_str()) {
                match ext {
                    "vsix" => {
                        // TODO: When installing, invoke `vscode` plugin install command,
                        // this must be handled after `VS-Code` has been installed,
                        // we might need a `requirements` field in the manifest.
                        return Ok(Self::Plugin(path));
                    }
                    _ => bail!("failed to install '{name}': unknown file format '{ext}'"),
                }
            }
        }
        // TODO: Well, we got a directory, things are getting complicated, there could be one of this scenarios:
        // 1. Directory contains some executable files and nothing else
        //      Throw these executable files into cargo bin folder
        // 2. Directory contains sub-directory, which look like `bin/ lib/ etc/ ...`
        //      Throw and merge this directories into cargo home. (might be bad, therefore we need a `Manifest.in`!!!)
        // 3. Directory doesn't fit both the previous characteristics but there is
        //    a custom installation instruction in the code base
        //      Use the custom instruction to install this tool
        // 4. Directory doesn't fit all previous characteristics.
        //      We don't know how to install this tool, throw an error instead.
        else {
            // Step 3: read directory to find characteristics.
            let entries = utils::walk_dir(path)?;
            // Check if there is any folder that looks like `bin`
            // Then assuming this is `UsrDirs` type installer.
            if let Some(bin_dir) = entries.iter().find(|path| path.ends_with("bin")) {
                return Ok(Self::DirWithBin {
                    root: path,
                    bin_dir: bin_dir.to_owned(),
                });
            }
            // If no sub folder exists, and there are binaries lays directly in the folder
            if !entries.iter().any(|path| path.is_dir()) {
                let assumed_binaries = entries
                    .iter()
                    .filter_map(|path| utils::is_executable(path).then_some(path.to_path_buf()));
                return Ok(Self::Executables(assumed_binaries.collect()));
            }
        }

        bail!("installing for '{name}' is not supported")
    }

    fn install(&self, config: &InstallConfiguration) -> Result<()> {
        match self {
            Self::Executables(exes) => {
                for exe in exes {
                    utils::copy_file_to(exe, config.cargo_bin())?;
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}

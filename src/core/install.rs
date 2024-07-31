use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};
use tempfile::TempDir;

use crate::utils;

use super::{manifest::ToolInfo, InstallConfiguration, CARGO_HOME, RUSTUP_HOME};

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

            utils::cli::download_from_start(name, url, &dest)?;
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

    if tool_installer_path.is_file() {
        let maybe_extension = tool_installer_path.extension();
        if let Some(ext) = maybe_extension {
            // TODO: this might be binary, or might be software extension, perform action base on this extension.
            match ext.to_str() {
                #[cfg(windows)]
                Some("exe") => {
                    utils::copy_file_to(&tool_installer_path, config.cargo_bin())?;
                }
                Some("vsix") => {
                    // TODO: invoke `vscode` plugin install command, this must be handled after `VS-Code` has
                    // been installed, we might need a `requirements` field in the manifest.
                }
                _ => bail!(
                    "failed to install '{name}': unknown file format '{}'",
                    ext.to_string_lossy()
                ),
            }
        } else {
            // assuming it is a executable, throw it in cargo bin directory
            utils::copy_file_to(&tool_installer_path, config.cargo_bin())?;
        }
    } else {
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
    }

    Ok(())
}

fn create_temp_dir(config: &InstallConfiguration, prefix: &str) -> Result<TempDir> {
    let root = config.temp_root();
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
        extractable.extract_to(dest)
    } else {
        utils::copy_to(maybe_file, dest)
    }
}

// TODO(?): Move this and `Extractable` to `utils`.
#[derive(Debug, Clone, Copy)]
enum ExtractableKind {
    Gz,
    Xz,
    Zip,
}

impl FromStr for ExtractableKind {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "gz" => Ok(Self::Gz),
            "xz" => Ok(Self::Xz),
            "zip" => Ok(Self::Zip),
            _ => Err(anyhow!("'{s}' is not a supported extrable file format")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Extractable<'a> {
    path: &'a Path,
    kind: ExtractableKind,
}

impl<'a> TryFrom<&'a Path> for Extractable<'a> {
    type Error = anyhow::Error;
    fn try_from(value: &'a Path) -> std::result::Result<Self, Self::Error> {
        let ext = value
            .extension()
            .ok_or_else(|| anyhow!("path '{}' is not extractable because it appears to have no file extension", value.display()))?
            .to_str()
            .ok_or_else(|| anyhow!("path '{}' is not extractable because it's path contains invalid unicode characters", value.display()))?;

        let kind: ExtractableKind = ext.parse()?;
        Ok(Self { path: value, kind })
    }
}

impl Extractable<'_> {
    /// Extract current file into a specific `root`.
    fn extract_to(&self, root: &Path) -> Result<PathBuf> {
        Ok(PathBuf::new())
    }
}

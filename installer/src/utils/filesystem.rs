use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Context, Result};

/// Get a path to user's "home" directory.
///
/// # Panic
///
/// Will panic if such directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub(crate) fn home_dir() -> PathBuf {
    home::home_dir().expect("aborting because the home directory cannot be determined.")
}

/// Get a path to the root directory of this program.
///
/// # Panic
///
/// Will panic if the home directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub(crate) fn installer_home() -> PathBuf {
    home_dir().join(env!("CARGO_PKG_NAME"))
}

/// Wrapper to [`std::fs::read_to_string`] but with additional error context.
pub(crate) fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read '{}'", path.as_ref().display()))
}

pub(crate) fn stringify_path<P: AsRef<Path>>(path: P) -> Result<String> {
    path.as_ref()
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "failed to stringify path '{}'",
                path.as_ref().to_string_lossy().to_string()
            )
        })
}

pub(crate) fn mkdirs<P: AsRef<Path>>(path: P) -> Result<()> {
    fs::create_dir_all(path.as_ref()).with_context(|| {
        format!(
            "unable to create specified directory '{}'",
            path.as_ref().display()
        )
    })
}

pub(crate) fn to_nomalized_abspath<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<PathBuf> {
    if path.as_ref().is_absolute() {
        return Ok(path.into());
    }
    let raw = env::current_dir()
        .context("current directory cannot be determined")
        .map(|mut cd| {
            cd.push(path);
            cd
        })?;
    // Remove any `.` and `..` from origin path
    let mut nomalized_path = PathBuf::new();
    for path_component in raw.components() {
        match path_component {
            Component::CurDir => (),
            Component::ParentDir => {
                nomalized_path.pop();
            }
            _ => nomalized_path.push(path_component),
        }
    }

    Ok(nomalized_path)
}

//! Utility functions/types to use across the whole crate.
//!
//! NOTE: Most of these are moved from the `experimental` branch,
//! some of them might turns out to be unused, so remember to clean those after version `1.0`.

mod download;
mod extraction;
mod file_system;
mod process;
mod progress_bar;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub use download::download;
pub use extraction::Extractable;
pub use file_system::*;
pub use process::*;
pub use progress_bar::Progress;

use anyhow::Result;
use url::Url;

#[cfg(not(windows))]
pub const EXE_EXT: &str = "";
#[cfg(windows)]
pub const EXE_EXT: &str = ".exe";

/// Forcefully parsing a `&str` to [`Url`].
///
/// # Panic
///
/// Causes panic if the given string cannot be parsed as `Url`.
pub fn force_parse_url(url: &str) -> Url {
    Url::parse(url).unwrap_or_else(|e| panic!("failed to parse url '{url}': {e}"))
}

/// Basically [`Url::join`], but will insert a forward slash (`/`) to the root if necessary.
///
/// [`Url::join`] will replace the last part of a root if the root does not have trailing slash,
/// and this function is to make sure of that, so the `root` will always join with `s`.
pub fn force_url_join(root: &Url, s: &str) -> Result<Url> {
    let result = if root.as_str().ends_with('/') {
        root.join(s)?
    } else {
        Url::parse(&format!("{}/{s}", root.as_str()))?
    };

    Ok(result)
}

pub fn path_to_str(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| {
        anyhow::anyhow!(
            "path '{}' cannot be convert to str as it may contains invalid unicode characters.",
            path.display()
        )
    })
}

/// Get the binary name of current executing binary, a.k.a `arg[0]`.
pub fn lowercase_program_name() -> Option<String> {
    let program_executable = std::env::args().next().map(PathBuf::from)?;
    let program_name = program_executable
        .file_name()
        .and_then(|oss| oss.to_str())?;
    Some(program_name.to_lowercase())
}

/// Lossy convert any [`OsStr`] representation into [`String`].
///
/// Check [`OsStr::to_string_lossy`] for detailed conversion.
pub fn to_string_lossy<S: AsRef<OsStr>>(s: S) -> String {
    s.as_ref().to_string_lossy().to_string()
}

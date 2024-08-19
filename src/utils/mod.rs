//! Utility functions/types to use across the whole crate.
//!
//! NOTE: Most of these are moved from the `experimental` branch,
//! some of them might turns out to be unused, so remember to clean those after version `1.0`.

mod download;
mod extraction;
mod file_system;
mod process;
mod progress_bar;

use std::path::Path;

pub use download::download_from_start;
pub use extraction::{Extractable, ExtractableKind};
pub use file_system::*;
pub use process::*;

use anyhow::Result;
use url::Url;

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

/// Flip `Option<Result<T, E>>` to `Result<Option<T>, E>`
pub fn flip_option_result<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}

pub fn path_to_str(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| {
        anyhow::anyhow!(
            "path '{}' cannot be convert to str as it may contains invalid unicode characters.",
            path.display()
        )
    })
}

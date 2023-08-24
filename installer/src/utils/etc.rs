use anyhow::{Context, Result};
use url::Url;

pub(crate) fn parse_url(url: &str) -> Result<Url> {
    Url::parse(url).with_context(|| format!("failed to parse url: {url}"))
}

/// Flip `Option<Result<T, E>>` to `Result<Option<T>, E>`
pub(crate) fn flip_option_result<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}

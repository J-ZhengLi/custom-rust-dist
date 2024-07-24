//! Module defining types that could be serialized to a working `config.toml` for cargo.

use serde::{ser::SerializeMap, Serialize};
use std::collections::BTreeMap;
use toml::ser;
use url::Url;

/// A simple struct representing the fields in `config.toml`.
///
/// Only covers a small range of options we need to configurate.
/// Fwiw, the full set of configuration options can be found
/// in the [Cargo Configuration Book](https://doc.rust-lang.org/cargo/reference/config.html).
#[derive(Debug, Default, Serialize)]
pub(crate) struct CargoConfig {
    net: Option<CargoNetConfig>,
    http: Option<CargoHttpConfig>,
    #[serde(serialize_with = "serialize_source_map")]
    source: BTreeMap<String, Source>,
}

// FIXME: remove this `allow` before 0.1.0 release.
#[allow(unused)]
impl CargoConfig {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn git_fetch_with_cli(&mut self, yes: bool) -> &mut Self {
        self.net = Some(CargoNetConfig {
            git_fetch_with_cli: Some(yes),
        });
        self
    }

    pub(crate) fn check_revoke(&mut self, yes: bool) -> &mut Self {
        self.http = Some(CargoHttpConfig {
            check_revoke: Some(yes),
        });
        self
    }

    /// Insert a source.
    ///
    /// NB: The first `add_source` call will also insert a `crates-io` source.
    ///
    /// - `key` is the name of the source.
    /// - `url` is the registry url.
    /// - `as_default` specify whether this source is used as a replaced source of `crates-io`,
    ///     note the first `add_source` call will always be default.
    pub(crate) fn add_source(&mut self, key: &str, url: Url, as_default: bool) -> &mut Self {
        self.source
            .entry("crates-io".to_string())
            .and_modify(|s| {
                if as_default {
                    s.replace_with = Some(key.to_string());
                }
            })
            .or_insert(Source {
                replace_with: Some(key.to_string()),
                ..Default::default()
            });

        self.source.insert(
            key.to_string(),
            Source {
                registry: Some(url),
                ..Default::default()
            },
        );

        self
    }

    pub(crate) fn to_toml(&self) -> anyhow::Result<String> {
        Ok(ser::to_string(self)?)
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoNetConfig {
    git_fetch_with_cli: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoHttpConfig {
    check_revoke: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Source {
    pub(crate) replace_with: Option<String>,
    #[serde(serialize_with = "serialize_url_opt")]
    pub(crate) registry: Option<Url>,
}

// Serialize empty map to an empty string.
fn serialize_source_map<S>(map: &BTreeMap<String, Source>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if map.is_empty() {
        serializer.serialize_none()
    } else {
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            ser_map.serialize_entry(k, v)?;
        }
        ser_map.end()
    }
}

fn serialize_url_opt<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match url.to_owned() {
        Some(u) => serializer.serialize_some(u.as_str()),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::CargoConfig;

    #[test]
    fn cargo_config_default_serialize() {
        // serized default should be an empty toml
        let default = CargoConfig::default();

        assert_eq!(default.to_toml().unwrap(), "");
    }

    #[test]
    fn cargo_config_serialize() {
        let config = CargoConfig::new()
            .git_fetch_with_cli(true)
            .check_revoke(false)
            .add_source(
                "mirror",
                "https://example.com/registry".try_into().unwrap(),
                true,
            )
            .to_toml()
            .unwrap();

        assert_eq!(
            config,
            r#"[net]
git-fetch-with-cli = true

[http]
check-revoke = false

[source.crates-io]
replace-with = "mirror"

[source.mirror]
registry = "https://example.com/registry"
"#
        );
    }
}

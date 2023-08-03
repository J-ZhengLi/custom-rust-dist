use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{de, Deserialize};

use super::TomlTable;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct CargoInstallTrackerLite {
    pub v1: BTreeMap<CargoPackageIdLite, BTreeSet<String>>,
}

impl TomlTable for CargoInstallTrackerLite {}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CargoPackageIdLite {
    pub name: String,
    pub version: String,
}

struct PackageIdVisitor;

impl<'de> de::Visitor<'de> for PackageIdVisitor {
    type Value = CargoPackageIdLite;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a space separated string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let get_name_and_ver = || -> Option<(String, String)> {
            let mut splited = v.split_whitespace();
            Some((splited.next()?.into(), splited.next()?.into()))
        };
        let (name, version) = get_name_and_ver()
            .ok_or_else(|| E::custom(format!("invalid package id syntax: {v}")))?;
        Ok(Self::Value { name, version })
    }
}

impl<'de> de::Deserialize<'de> for CargoPackageIdLite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PackageIdVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{CargoInstallTrackerLite, CargoPackageIdLite, TomlTable};

    #[test]
    fn deserialize_cargo_installation_toml() {
        let toml = r#"[v1]
        "crate_a 0.1.0 (..)" = ["crate_a"]
        "crate_b 0.2.0 (..)" = ["crate_b"]
        "#;

        let installation = CargoInstallTrackerLite::from_toml(toml).unwrap();
        assert_eq!(
            installation,
            CargoInstallTrackerLite {
                v1: BTreeMap::from([
                    (
                        CargoPackageIdLite {
                            name: "crate_a".into(),
                            version: "0.1.0".into()
                        },
                        BTreeSet::from(["crate_a".to_string()]),
                    ),
                    (
                        CargoPackageIdLite {
                            name: "crate_b".into(),
                            version: "0.2.0".into()
                        },
                        BTreeSet::from(["crate_b".to_string()]),
                    )
                ]),
            }
        )
    }
}

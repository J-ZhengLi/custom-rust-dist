use super::TomlParser;
use semver::Version;
use serde::{de, Deserialize, Deserializer};

/// A type designed to contain the information about the newest `manager` release.
///
/// This only contains software `version` for now.
#[derive(Debug, Deserialize)]
pub(crate) struct ReleaseInfo {
    #[serde(deserialize_with = "de_version")]
    pub(crate) version: Version,
}

fn de_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Version::parse(&s)
        .map_err(|e| de::Error::custom(format!("invalid semantic version, reason: {e}")))
}

impl TomlParser for ReleaseInfo {
    const FILENAME: &str = "release.toml";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_info() {
        let input = "version = '1.2.3-beta.1'";
        let release = ReleaseInfo::from_str(input).unwrap();
        let version = release.version;

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre.as_str(), "beta.1");
    }

    #[test]
    #[should_panic(expected = "invalid semantic version")]
    fn bad_version() {
        let input = "version = 'stable'";
        let _release = ReleaseInfo::from_str(input).unwrap();
    }
}

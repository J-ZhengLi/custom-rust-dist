use std::str::FromStr;

use clap::builder::PossibleValue;

#[non_exhaustive]
pub enum Language {
    CN,
    EN,
}

impl Language {
    pub fn possible_values() -> [Language; 2] {
        [Self::CN, Self::EN]
    }
    /// Returns the string representation of this enum,
    /// this will be the same one that parsed from commandline input.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CN => "cn",
            Self::EN => "en",
        }
    }
    /// This is the `str` used for setting locale,
    /// make sure the values match the filenames under `<root>/locales`.
    pub fn locale_str(&self) -> &str {
        match self {
            Self::CN => "zh-CN",
            Self::EN => "en",
        }
    }
}

impl FromStr for Language {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cn" => Ok(Self::CN),
            "en" => Ok(Self::EN),
            _ => Err(anyhow::anyhow!(
                "invalid or unsupported language option: {s}"
            )),
        }
    }
}

// We just need this to satisfy clap's parser, and it doesn't work other way around anyway.
#[allow(clippy::from_over_into)]
impl Into<PossibleValue> for Language {
    fn into(self) -> PossibleValue {
        PossibleValue::new(self.as_str())
    }
}

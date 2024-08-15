pub mod components;
pub type Result<T> = core::result::Result<T, InstallerError>;

use std::fmt::Display;

#[derive(Debug)]
pub enum InstallerError {
    Anyhow(anyhow::Error),
}

impl Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anyhow(..) => write!(f, "anyhow error"),
        }
    }
}

impl std::error::Error for InstallerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::Anyhow(ref e) => Some(e.root_cause()),
        }
    }
}

impl From<anyhow::Error> for InstallerError {
    fn from(value: anyhow::Error) -> Self {
        Self::Anyhow(value)
    }
}

impl From<InstallerError> for tauri::InvokeError {
    fn from(value: InstallerError) -> Self {
        match value {
            InstallerError::Anyhow(ah) => tauri::InvokeError::from_anyhow(ah),
        }
    }
}

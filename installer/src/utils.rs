use std::path::PathBuf;
use std::env;
use anyhow::Result;

pub(crate) fn home_dir() -> PathBuf {
    home::home_dir().expect("aborting because the home directory cannot be determined.")
}

/// Directory of current executable
pub(crate) fn exe_dir() -> Result<PathBuf> {
    env::current_exe()?
        .parent()
        .map(ToOwned::to_owned)
        .ok_or_else(|| unreachable!("path of current executable always has parent"))
}

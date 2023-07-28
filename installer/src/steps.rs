//! Definition of functions which will be called during initialization.
//!
//! NB: `steps` here is a placeholder name, which might be changed later.
//!
//! When running this application, it will performs the following:
//! 1. Try to load configuration from a dedicated location.
//! 2. Try gathering settings from various source, such as env vars, or file contents.
//! 3.  a. If configuration does not exist:
//!         Guide the user to generate one, with the gathered settings if available.
//!     b. If configuration exists but is different with gathered settings:
//!         Ask user if they want to update.
//!     c. Otherwise continue to next step.
//! 4. Try to load installation information from a dedicated location.
//! 5.  a. If installation toml does not exist or exist but rustup is not installed:
//!         Guide the user through rustup-init configurations and then run it with args.
//!         After that, update this installation toml.
//!     b. If toml exists and rustup is installed:
//!         Try to gather toolchain/tool info by accessing various files,
//!         and SYNC those info into installation.toml.

use crate::parser::{self, Configuration, Settings};
use anyhow::Result;

pub(crate) fn load_config() -> Result<Configuration> {
    let config_path = crate::config_path();
    parser::load_config(config_path)
}

pub(crate) fn update_settings(setts: Settings) -> Result<()> {
    let mut existing_cfg = load_config()?;
    existing_cfg.settings = setts;
    Ok(())
}

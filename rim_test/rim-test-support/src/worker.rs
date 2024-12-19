use std::path::PathBuf;

pub trait SnapboxCommandExt {
    fn rim_cli() -> Self;
    fn rim_dev() -> Self;
}

impl SnapboxCommandExt for snapbox::cmd::Command {
    fn rim_cli() -> Self {
        Self::new(rim_cli())
    }

    fn rim_dev() -> Self {
        Self::new(rim_dev())
    }
}

/// Path to the rim-cli binary
fn rim_cli() -> PathBuf {
    snapbox::cmd::cargo_bin("rim-cli")
}

/// Path to rim-dev binary
fn rim_dev() -> PathBuf {
    snapbox::cmd::cargo_bin("rim-dev")
}

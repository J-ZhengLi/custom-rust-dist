//! Module to create a fake installation root, useful to test the `manager` utilities.

use crate::common;

use super::TOOLKIT_NAME;
use anyhow::{bail, Result};
use std::{env::consts::EXE_SUFFIX, fs, path::PathBuf, process::Command};

struct FakeInstallation {
    manager_bin: Option<PathBuf>,
}

impl FakeInstallation {
    fn new() -> Self {
        Self { manager_bin: None }
    }

    fn fingerprint_content(&self, ver: &str) -> String {
        format!(
            "
name = '{TOOLKIT_NAME}'
version = 'stable-{ver}'
root = '{0}'

[rust]
version = '{ver}'
components = [\"llvm-tools\", \"rustc-dev\"]

[tools.mingw64]
kind = 'dir-with-bin'
version = '13.0.0'
paths = ['{0}/tools/mingw64']
",
            super::install_dir().display()
        )
    }

    fn generate_manager_bin(&mut self, no_gui: bool) -> Result<()> {
        let cargo_args = if no_gui {
            ["build"].to_vec()
        } else {
            ["tauri", "build", "--debug", "-b", "none"].to_vec()
        };

        if !no_gui {
            common::install_gui_deps();
        }

        // build rim
        let build_status = Command::new("cargo").args(cargo_args).status()?;
        if !build_status.success() {
            bail!("failed to build manager binary");
        }

        // make a copy of rim as manager binary to the fake installation root
        let (src_bin, dest_bin) = if no_gui {
            (
                format!("rim-cli{EXE_SUFFIX}"),
                format!("manager-cli{EXE_SUFFIX}"),
            )
        } else {
            (
                format!("rim-gui{EXE_SUFFIX}"),
                format!("manager{EXE_SUFFIX}"),
            )
        };
        let build_artifact = super::debug_dir().join(src_bin);
        let dest_path = super::install_dir().join(dest_bin);
        fs::copy(build_artifact, &dest_path)?;

        self.manager_bin = Some(dest_path);

        Ok(())
    }

    fn generate_meta_files(&self) -> Result<()> {
        let fingerprint_path = super::install_dir().join(".fingerprint.toml");
        let manifest_path = super::install_dir().join("toolset-manifest.toml");

        // don't write if the path already exists
        if !fingerprint_path.exists() {
            fs::write(fingerprint_path, self.fingerprint_content("1.0.0"))?;
        }
        let manifest = include_str!("../../../resources/toolset_manifest.toml");
        if !manifest_path.exists() {
            fs::write(manifest_path, manifest)?;
        }

        Ok(())
    }
}

pub(crate) fn generate_and_run_manager(no_gui: bool, args: &[String]) -> Result<()> {
    let mut fake = FakeInstallation::new();
    fake.generate_meta_files()?;
    fake.generate_manager_bin(no_gui)?;

    // `fake.manager_bin` cannot be none if the previous `generate_manager_bin`
    // succeeded, so it's safe to unwrap
    let manager = &fake.manager_bin.unwrap();

    let mocked_dist_server = super::server_dir_url();
    // run the manager copy
    let status = Command::new(manager)
        .args(args)
        .env("RIM_DIST_SERVER", mocked_dist_server.as_str())
        .status()?;
    println!(
        "\nmanager exited with status code: {}",
        status.code().unwrap_or(-1)
    );
    Ok(())
}

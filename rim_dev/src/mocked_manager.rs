use anyhow::Result;
use std::{
    env::consts::EXE_SUFFIX,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

static MANAGER_DIR: OnceLock<PathBuf> = OnceLock::new();

struct FakeRim {
    main_rs: String,
    cargo_toml: String,
    version: String,
}

impl FakeRim {
    fn new(version: &str) -> Self {
        let main_rs = format!(
            "
fn main() {{
    if std::env::args().any(|arg| arg == \"--version\") {{
        println!(\"rim {version}\");
    }}
}}"
        );
        let cargo_toml = format!(
            "
[package]
name = \"rim\"
version = \"{version}\"
edition = \"2021\"
[workspace]"
        );

        Self {
            main_rs,
            cargo_toml,
            version: version.into(),
        }
    }

    fn build(self) -> Result<()> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let temp_dir_root = manifest_dir.with_file_name("target").join("tmp");
        fs::create_dir_all(&temp_dir_root)?;

        let temp_dir = temp_dir_root.join("mocked_rim");
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir)?;
        fs::write(src_dir.join("main.rs"), self.main_rs)?;
        fs::write(temp_dir.join("Cargo.toml"), self.cargo_toml)?;

        // Build the mocked crate
        let mut c = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&temp_dir)
            .spawn()?;
        c.wait()?;

        // Collect built artifact
        let mut binary_path = temp_dir.join("target");
        binary_path.push("release");
        binary_path.push(format!("rim{}", std::env::consts::EXE_SUFFIX));
        let mut dest_dir = manager_dir().join("archive");
        dest_dir.push(&self.version);
        dest_dir.push(env!("TARGET"));
        fs::create_dir_all(&dest_dir)?;

        let gui_name = format!("{}-manager{EXE_SUFFIX}", t!("vendor_en"));
        let cli_name = format!("{}-manager-cli{EXE_SUFFIX}", t!("vendor_en"));
        fs::copy(&binary_path, dest_dir.join(gui_name))?;
        fs::copy(&binary_path, dest_dir.join(cli_name))?;

        Ok(())
    }
}

fn manager_dir() -> &'static Path {
    MANAGER_DIR.get_or_init(|| {
        let mut m_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("resources");
        m_dir.push("mock");
        m_dir.push("manager");
        m_dir
    })
}

/// Generate a `release.toml` for self update, that the version will always be newer.
fn gen_release_toml(version: &str) -> Result<()> {
    let release_toml = manager_dir().join("release.toml");

    let desired_content = format!("version = '{version}'");
    fs::write(release_toml, desired_content)?;
    Ok(())
}

/// Generate a target version base on the current workspace version.
///
/// The target version will always be one major release ahead of the current version,
/// so if the current version is `1.0.0`, the target version will be `2.0.0`.
fn mocked_ws_version() -> String {
    let ws_manifest_content = include_str!("../../Cargo.toml");
    let cur_ver = ws_manifest_content
        .lines()
        .find_map(|line| {
            let trimed = line.trim();
            if let Some((_, ver_with_quote)) = trimed
                .starts_with("version")
                .then_some(trimed.split_once('='))
                .flatten()
            {
                Some(ver_with_quote.trim_matches([' ', '\'', '"']))
            } else {
                None
            }
        })
        .unwrap_or_else(|| unreachable!("'version' field is required in any cargo manifest"));

    // safe to unwrap the below lines, otherwise cargo would fails the build.
    let (major, rest) = cur_ver.split_once('.').unwrap();
    let major_number: usize = major.parse().unwrap();

    format!("{}.{rest}", major_number + 1)
}

/// Generate mocked manager binary for self updating tests.
pub(crate) fn generate() -> Result<()> {
    let target_ver = mocked_ws_version();

    gen_release_toml(&target_ver)?;
    // Generate mocked binaries
    FakeRim::new(&target_ver).build()?;

    Ok(())
}

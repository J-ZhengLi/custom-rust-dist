use anyhow::{bail, Result};
use std::{fs, path::PathBuf, process::Command, str::FromStr};

pub(super) const VENDOR_HELP: &str = r#"
Download packages that are specified in `resource/packages.txt`

Usage: cargo dev vendor
"#;

pub(super) fn vendor() -> Result<()> {
    let res_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name("resources");
    let pkg_dir = res_dir.join("packages");
    let pkg_list = res_dir.join("packages.txt");

    if !pkg_list.is_file() {
        return Ok(());
    }

    fs::create_dir_all(&pkg_dir)?;

    let list_content = fs::read_to_string(pkg_list)?;
    let list = list_content.split_ascii_whitespace().collect::<Vec<_>>();

    for entry in list {
        let Ok(pkg_src) = entry.parse::<PackageSource>() else {
            continue;
        };
        let full_path = pkg_dir.join(&pkg_src.relpath);

        if full_path.exists() {
            continue;
        }

        if pkg_src.is_dir {
            fs::create_dir_all(&full_path)?;
            continue;
        }

        // download missing packages
        if let Some(url) = pkg_src.source {
            Command::new("curl")
                .arg("-o")
                .arg(full_path)
                .arg(url)
                .status()?;
        }
    }

    Ok(())
}

struct PackageSource {
    is_dir: bool,
    relpath: PathBuf,
    source: Option<String>,
}

impl FromStr for PackageSource {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let trimed = s.trim();
        if trimed.is_empty() {
            bail!("invalid package source, no empty line allowed");
        }

        let splited = trimed.split_once('@');

        if let Some((relpath, source)) = splited {
            let normalized = relpath.replace('/', "\\");
            Ok(Self {
                is_dir: false,
                relpath: normalized.into(),
                source: Some(source.to_string()),
            })
        } else {
            let normalized = trimed.replace('/', "\\");
            Ok(Self {
                is_dir: true,
                relpath: normalized.into(),
                source: None,
            })
        }
    }
}

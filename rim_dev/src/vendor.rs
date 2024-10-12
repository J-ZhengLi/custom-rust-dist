use anyhow::{bail, Result};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

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
            println!("downloading {url}");
            let status = Command::new("curl")
                .arg("-Lo")
                .arg(full_path)
                .arg(url)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;
            if !status.success() {
                bail!("download failed");
            }
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

        let normalize = |path_str: &str| -> PathBuf {
            #[cfg(windows)]
            let res = path_str.replace('/', "\\").into();
            #[cfg(not(windows))]
            let res = path_str.into();
            res
        };
        let splited = trimed.split_once('@');

        let (relpath, source) = splited
            .map(|(path, url)| (normalize(path), Some(url.into())))
            .unwrap_or_else(|| (normalize(trimed), None));

        Ok(Self {
            is_dir: source.is_none(),
            relpath,
            source,
        })
    }
}

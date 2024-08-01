use anyhow::{anyhow, Result};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use zip::ZipArchive;

use super::progress_bar::ProgressIndicator;

#[derive(Debug, Clone, Copy)]
pub enum ExtractableKind {
    Gz,
    Xz,
    Zip,
}

impl FromStr for ExtractableKind {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "gz" => Ok(Self::Gz),
            "xz" => Ok(Self::Xz),
            "zip" => Ok(Self::Zip),
            _ => Err(anyhow!("'{s}' is not a supported extrable file format")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Extractable<'a> {
    path: &'a Path,
    kind: ExtractableKind,
}

impl<'a> TryFrom<&'a Path> for Extractable<'a> {
    type Error = anyhow::Error;
    fn try_from(value: &'a Path) -> std::result::Result<Self, Self::Error> {
        let ext = value
            .extension()
            .ok_or_else(|| anyhow!("path '{}' is not extractable because it appears to have no file extension", value.display()))?
            .to_str()
            .ok_or_else(|| anyhow!("path '{}' is not extractable because it's path contains invalid unicode characters", value.display()))?;

        let kind: ExtractableKind = ext.parse()?;
        Ok(Self { path: value, kind })
    }
}

impl Extractable<'_> {
    /// Extract current file into a specific directory.
    ///
    /// This will extract file under the `root`, make sure it's an empty folder before using this function.
    pub fn extract_to(&self, root: &Path) -> Result<()> {
        let indicator = ProgressIndicator::new();

        match self.kind {
            ExtractableKind::Zip => {
                let file = std::fs::File::open(self.path)?;
                let mut zip_archive = ZipArchive::new(file)?;
                let zip_len = zip_archive.len();

                // Init progress
                let bar = (indicator.start)(
                    zip_len.try_into()?,
                    format!("extracting file '{}'", self.path.display()),
                )?;

                for i in 0..zip_len {
                    let mut zip_file = zip_archive.by_index(i)?;

                    let out_path = match zip_file.enclosed_name() {
                        Some(path) => root.join(path),
                        None => continue,
                    };

                    if zip_file.is_dir() {
                        super::mkdirs(&out_path)?;
                    } else {
                        if let Some(p) = out_path.parent() {
                            if !p.exists() {
                                super::mkdirs(p)?;
                            }
                        }
                        let mut out_file = std::fs::File::create(&out_path)?;
                        std::io::copy(&mut zip_file, &mut out_file)?;
                    }

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Some(mode) = zip_file.unix_mode() {
                            std::fs::set_permissions(
                                &out_path,
                                std::fs::Permissions::from_mod(mode),
                            )?;
                        }
                    }

                    (indicator.update)(&bar, i.try_into()?);
                }
                (indicator.stop)(&bar, "extraction complete.".into());
            }
            _ => unimplemented!("extracting tarball is not implemented"),
        }

        Ok(())
    }
}

use anyhow::{anyhow, Context, Result};
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use super::progress_bar::ProgressIndicator;

#[derive(Debug, Clone, Copy)]
pub enum ExtractableKind {
    /// `7-zip` compressed files, ended with `.7z`
    SevenZ,
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
            "7z" => Ok(Self::SevenZ),
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
            ExtractableKind::Zip => extract_zip(self.path, root, indicator),
            ExtractableKind::SevenZ => extract_7z(self.path, root, indicator),
            _ => unimplemented!("extracting tarball is not implemented"),
        }
    }
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(p) = path.parent() {
        if !p.exists() {
            super::mkdirs(p)?;
        }
    }
    Ok(())
}

fn extract_zip<T: Sized>(path: &Path, root: &Path, indicator: ProgressIndicator<T>) -> Result<()> {
    use zip::ZipArchive;

    let file = std::fs::File::open(path)?;
    let mut zip_archive = ZipArchive::new(file)?;
    let zip_len = zip_archive.len();

    // Init progress
    let bar = (indicator.start)(
        zip_len.try_into()?,
        format!("extracting file '{}'", path.display()),
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
            ensure_parent_dir(&out_path)?;
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut zip_file, &mut out_file)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = zip_file.unix_mode() {
                std::fs::set_permissions(&out_path, std::fs::Permissions::from_mod(mode))?;
            }
        }

        (indicator.update)(&bar, i.try_into()?);
    }
    (indicator.stop)(&bar, "extraction complete.".into());

    Ok(())
}

fn extract_7z<T: Sized>(path: &Path, root: &Path, indicator: ProgressIndicator<T>) -> Result<()> {
    use sevenz_rust::{Password, SevenZReader};

    // Open the given 7z file, there shouldn't be any password protected files tho,
    // if there is, just let it fail for now.
    let mut sz_reader = SevenZReader::open(path, Password::empty())
        .with_context(|| format!("failed to read 7z archive '{}'", path.display()))?;
    let sz_len: u64 = sz_reader
        .archive()
        .files
        .iter()
        .filter_map(|e| e.has_stream().then_some(e.size()))
        .sum();
    let mut extracted_len: u64 = 0;

    // Init progress bar
    let bar = (indicator.start)(sz_len, format!("extracting file '{}'", path.display()))?;

    sz_reader.for_each_entries(|entry, reader| {
        let mut buf = [0_u8; 1024];

        let out_path = root.join(entry.name());

        if entry.is_directory() {
            super::mkdirs(&out_path).map_err(|_| {
                sevenz_rust::Error::other(format!(
                    "unable to create entry directory '{}'",
                    out_path.display()
                ))
            })?;
            Ok(true)
        } else {
            ensure_parent_dir(&out_path).map_err(|_| {
                sevenz_rust::Error::other(format!(
                    "unable to create parent directory for '{}'",
                    out_path.display()
                ))
            })?;

            let mut out_file = std::fs::File::create(&out_path)?;
            loop {
                let read_size = reader.read(&mut buf)?;
                if read_size == 0 {
                    break Ok(true);
                }
                out_file.write_all(&buf[..read_size])?;
                extracted_len += read_size as u64;
                // Update progress bar
                (indicator.update)(&bar, extracted_len);
            }
        }
        // NB: sevenz-rust does not support `unix-mode` like `zip` does, so we might ended up
        // mess up the extracted file's permission... let's hope that never happens.
    })?;

    // Stop progress bar's progress
    (indicator.stop)(&bar, "extraction complete.".into());

    Ok(())
}

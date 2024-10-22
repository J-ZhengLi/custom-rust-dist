use anyhow::{anyhow, bail, Context, Result};
use common_path::common_path_all;
use flate2::read::GzDecoder;
use log::info;
use sevenz_rust::{Password, SevenZReader};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use xz2::read::XzDecoder;
use zip::ZipArchive;

use crate::utils::progress_bar::Style;

use super::progress_bar::CliProgress;

enum ExtractableKind {
    /// `7-zip` compressed files, ended with `.7z`
    SevenZ(SevenZReader<File>),
    Gz(tar::Archive<GzDecoder<File>>),
    Xz(tar::Archive<XzDecoder<File>>),
    Zip(ZipArchive<File>),
}

pub struct Extractable<'a> {
    path: &'a Path,
    kind: ExtractableKind,
}

impl<'a> Extractable<'a> {
    pub fn load(path: &'a Path) -> Result<Self> {
        let ext = path
            .extension()
            .ok_or_else(|| {
                anyhow!(
                    "'{}' is not extractable because it appears to have no file extension",
                    path.display()
                )
            })?
            .to_str()
            .ok_or_else(|| {
                anyhow!(
                "'{}' is not extractable because its extension contains invalid unicode characters",
                path.display()
            )
            })?;

        let kind = match ext {
            "7z" => {
                info!(
                    "{}",
                    t!("loading_archive_info", kind = ext, path = path.display())
                );
                ExtractableKind::SevenZ(SevenZReader::open(path, Password::empty())?)
            }
            "zip" => {
                info!(
                    "{}",
                    t!("loading_archive_info", kind = ext, path = path.display())
                );
                ExtractableKind::Zip(ZipArchive::new(File::open(path)?)?)
            }
            "gz" => {
                info!(
                    "{}",
                    t!("loading_archive_info", kind = ext, path = path.display())
                );
                let tar_gz = GzDecoder::new(File::open(path)?);
                ExtractableKind::Gz(tar::Archive::new(tar_gz))
            }
            "xz" => {
                info!(
                    "{}",
                    t!("loading_archive_info", kind = ext, path = path.display())
                );
                let tar_xz = XzDecoder::new(File::open(path)?);
                ExtractableKind::Xz(tar::Archive::new(tar_xz))
            }
            _ => bail!("'{ext}' is not a supported extractable file format"),
        };

        Ok(Self { path, kind })
    }

    /// Extract current file into a specific directory.
    ///
    /// This will extract file under the `root`, make sure it's an empty folder before using this function.
    pub fn extract_to(&mut self, root: &Path) -> Result<()> {
        let indicator = CliProgress::new();

        let helper = ExtractHelper {
            file_path: self.path,
            output_dir: root,
            indicator,
        };

        match &mut self.kind {
            ExtractableKind::Zip(archive) => helper.extract_zip(archive),
            ExtractableKind::SevenZ(archive) => helper.extract_7z(archive),
            ExtractableKind::Gz(archive) => helper.extract_tar(archive),
            ExtractableKind::Xz(archive) => helper.extract_tar(archive),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ExtractHelper<'a, T: Sized> {
    file_path: &'a Path,
    output_dir: &'a Path,
    indicator: CliProgress<T>,
}

impl<'a, T: Sized> ExtractHelper<'a, T> {
    fn start_progress_bar(&self, len: u64, style: Style) -> Result<T> {
        (self.indicator.start)(
            len,
            format!("extracting file '{}'", self.file_path.display()),
            style,
        )
    }

    fn update_progress_bar(&self, bar: &T, prog: u64) {
        (self.indicator.update)(bar, prog);
    }

    fn end_progress_bar(&self, bar: &T) {
        (self.indicator.stop)(bar, "extraction complete.".into());
    }

    fn count_prefixes_for_tar<R: Read>(entries: &[tar::Entry<R>]) -> Result<usize> {
        if entries.len() < 2 {
            return Ok(0);
        }

        let mut all_files = vec![];
        for entry in entries {
            if entry.header().entry_type().is_file() {
                all_files.push(entry.path()?.to_path_buf());
            }
        }
        let common_prefix = common_path_all(all_files.iter().map(|c| c.borrow()));
        Ok(common_prefix
            .map(|p| p.components().count())
            .unwrap_or_default())
    }

    fn count_prefixes_for_zip(entries: &Vec<&str>) -> Result<usize> {
        if entries.len() < 2 {
            return Ok(0);
        }

        let all_files = entries
            .iter()
            .filter_map(|p| (!p.ends_with('/')).then_some(Path::new(p)));
        let common_prefix = common_path_all(all_files);
        Ok(common_prefix
            .map(|p| p.components().count())
            .unwrap_or_default())
    }

    fn count_prefixes_for_7z(entries: &[sevenz_rust::SevenZArchiveEntry]) -> Result<usize> {
        if entries.len() < 2 {
            return Ok(0);
        }

        // NB: `.archive().files` is a fxxking lie, it contains directories as well,
        // so we still need to filter directories.
        let all_files = entries
            .iter()
            .filter_map(|p| (!p.is_directory()).then_some(Path::new(p.name())));
        let common_prefix = common_path_all(all_files);
        Ok(common_prefix
            .map(|p| p.components().count())
            .unwrap_or_default())
    }

    fn extract_zip(&self, archive: &mut ZipArchive<File>) -> Result<()> {
        let zip_len = archive.len();
        let prefix_count = Self::count_prefixes_for_zip(&archive.file_names().collect())?;

        // Init progress
        let bar = self.start_progress_bar(zip_len.try_into()?, Style::Len)?;

        for i in 0..zip_len {
            let mut zip_file = archive.by_index(i)?;
            let out_path = match zip_file.enclosed_name() {
                Some(path) => {
                    let skipped = path.components().skip(prefix_count).collect::<PathBuf>();
                    if skipped == PathBuf::new() {
                        continue;
                    }
                    self.output_dir.join(skipped)
                }
                None => continue,
            };

            if zip_file.is_dir() {
                super::ensure_dir(&out_path)?;
            } else {
                super::ensure_parent_dir(&out_path)?;
                let mut out_file = std::fs::File::create(&out_path)?;
                std::io::copy(&mut zip_file, &mut out_file)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = zip_file.unix_mode() {
                    std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
                }
            }

            self.update_progress_bar(&bar, i.try_into()?);
        }
        self.end_progress_bar(&bar);

        Ok(())
    }

    fn extract_7z(&self, archive: &mut SevenZReader<File>) -> Result<()> {
        let entries = &archive.archive().files;
        let prefix_count = Self::count_prefixes_for_7z(entries)?;
        let sz_len: u64 = entries
            .iter()
            .filter_map(|e| e.has_stream().then_some(e.size()))
            .sum();
        let mut extracted_len: u64 = 0;

        // Init progress bar
        let bar = self.start_progress_bar(sz_len, Style::Bytes)?;

        archive.for_each_entries(|entry, reader| {
            let mut buf = [0_u8; 1024];
            let mut entry_path = PathBuf::from(entry.name());
            entry_path = entry_path.components().skip(prefix_count).collect();

            if entry_path == PathBuf::new() {
                return Ok(true);
            }
            let out_path = self.output_dir.join(&entry_path);

            if entry.is_directory() {
                super::ensure_dir(&out_path).map_err(|_| {
                    sevenz_rust::Error::other(format!(
                        "unable to create entry directory '{}'",
                        out_path.display()
                    ))
                })?;
                Ok(true)
            } else {
                super::ensure_parent_dir(&out_path).map_err(|_| {
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
                    self.update_progress_bar(&bar, extracted_len);
                }
            }
            // NB: sevenz-rust does not support `unix-mode` like `zip` does, so we might ended up
            // mess up the extracted file's permission... let's hope that never happens.
        })?;

        self.end_progress_bar(&bar);
        Ok(())
    }

    fn extract_tar<R: Read>(&self, archive: &mut tar::Archive<R>) -> Result<()> {
        #[cfg(unix)]
        archive.set_preserve_permissions(true);

        let mut entries = vec![];
        for maybe_entry in archive.entries()? {
            let entry = maybe_entry?;
            entries.push(entry);
        }

        let prefix_count = Self::count_prefixes_for_tar(&entries)?;
        let total_len: u64 = entries.len().try_into()?;
        // Init progress bar
        let bar = self.start_progress_bar(total_len, Style::Len)?;

        for (idx, mut entry) in entries.into_iter().enumerate() {
            let entry_path = entry.path()?;
            let skipped = entry_path
                .components()
                .skip(prefix_count)
                .collect::<PathBuf>();
            if skipped == PathBuf::new() {
                continue;
            }

            let out_path = self.output_dir.join(&skipped);

            if entry.header().entry_type().is_dir() {
                super::ensure_dir(&out_path).with_context(|| {
                    format!(
                        "failed to create directory when extracting '{}'",
                        self.file_path.display()
                    )
                })?;
            } else {
                super::ensure_parent_dir(&out_path).with_context(|| {
                    format!(
                        "failed to create directory when extracting '{}'",
                        self.file_path.display()
                    )
                })?;

                let mut out_file = std::fs::File::create(&out_path)?;
                std::io::copy(&mut entry, &mut out_file)?;
            }

            // Update progress bar
            self.update_progress_bar(&bar, u64::try_from(idx)? + 1);
        }

        // Stop progress bar's progress
        self.end_progress_bar(&bar);

        Ok(())
    }
}

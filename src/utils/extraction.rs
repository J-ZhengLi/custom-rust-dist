use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use log::info;
use sevenz_rust::{Password, SevenZReader};
use std::ffi::OsStr;
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

    /// Extract file into a specific root like [`extract_to`](Extractable::extract_to),
    /// then look for **solo** nested directory and return the last one.
    ///
    /// This works similar to skipping common prefixes, except this does not
    /// handle common prefixes when extracting. ~~(because I don't know how)~~
    ///
    /// If `stop` contains a folder name, this function will stop when encountered that folder and
    /// return the full extracted path of **its parent** instead.
    ///
    /// # Example:
    /// Suppose we have an archive with entries like this:
    /// ```text
    /// Foo
    ///  |- a
    ///     |- b
    ///        |- c
    ///           |- d1
    ///           |- d2
    /// ```
    /// Then after calling this function, the path to `c` will be returned,
    /// because it's the last solo directory in the archive
    /// ```ignore
    /// let dir = Extractable::load("/path/to/foo.tar.gz")?
    ///     .extract_then_skip_solo_dir("/path/to/foo", None)?;
    /// assert_eq!(dir, PathBuf::from("/path/to/foo/a/b/c"));
    /// ```
    pub fn extract_then_skip_solo_dir<S: AsRef<OsStr>>(
        &mut self,
        root: &Path,
        stop: Option<S>,
    ) -> Result<PathBuf> {
        fn inner_<S: AsRef<OsStr>>(root: &Path, stop: Option<S>) -> Result<PathBuf> {
            if let [sub_dir] = super::walk_dir(root, false)?.as_slice() {
                if matches!(stop, Some(ref keyword) if filename_matches_keyword(sub_dir, keyword)) {
                    Ok(root.to_path_buf())
                } else {
                    inner_(sub_dir, stop)
                }
            } else {
                Ok(root.to_path_buf())
            }
        }

        // first we need to extract the tarball
        self.extract_to(root)?;
        // then find the last solo dir recursively
        inner_(root, stop)
    }
}

fn filename_matches_keyword<S: AsRef<OsStr>>(path: &Path, keyword: S) -> bool {
    if let Some(name) = path.file_name() {
        name == keyword.as_ref()
    } else {
        false
    }
}

#[derive(Debug, Clone, Copy)]
struct ExtractHelper<'a, T: Sized> {
    file_path: &'a Path,
    output_dir: &'a Path,
    indicator: CliProgress<T>,
}

impl<T: Sized> ExtractHelper<'_, T> {
    fn start_progress_bar(&self, style: Style) -> Result<T> {
        (self.indicator.start)(
            format!("extracting file '{}'", self.file_path.display()),
            style,
        )
    }

    fn update_progress_bar(&self, bar: &T, prog: Option<u64>) {
        (self.indicator.update)(bar, prog);
    }

    fn end_progress_bar(&self, bar: &T) {
        (self.indicator.stop)(bar, "extraction complete.".into());
    }

    fn extract_zip(&self, archive: &mut ZipArchive<File>) -> Result<()> {
        let zip_len = archive.len();

        // Init progress
        let bar = self.start_progress_bar(Style::Len(zip_len.try_into()?))?;

        for i in 0..zip_len {
            let mut zip_file = archive.by_index(i)?;
            let Some(out_path) = zip_file
                .enclosed_name()
                .map(|path| self.output_dir.join(path))
            else {
                continue;
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

            self.update_progress_bar(&bar, Some(i.try_into()?));
        }
        self.end_progress_bar(&bar);

        Ok(())
    }

    fn extract_7z(&self, archive: &mut SevenZReader<File>) -> Result<()> {
        let entries = &archive.archive().files;
        let sz_len: u64 = entries
            .iter()
            .filter_map(|e| e.has_stream().then_some(e.size()))
            .sum();
        let mut extracted_len: u64 = 0;

        // Init progress bar
        let bar = self.start_progress_bar(Style::Bytes(sz_len))?;

        archive.for_each_entries(|entry, reader| {
            let mut buf = [0_u8; 1024];
            let entry_path = PathBuf::from(entry.name());
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
                    self.update_progress_bar(&bar, Some(extracted_len));
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

        let entries = archive.entries()?;

        // Init progress bar, use spinner because the length of entries cannot be retrieved.
        // NB: DO NOT consume the entries, like collect it into a vec or something,
        // as it will corrupt the contents within it, causing data loss in some of the files:
        // https://github.com/J-ZhengLi/rim/issues/161
        let bar = self.start_progress_bar(Style::Spinner)?;

        for (idx, mut entry) in entries.into_iter().filter_map(|e| e.ok()).enumerate() {
            let entry_path = entry.path()?;
            let out_path = self.output_dir.join(&entry_path);

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
            self.update_progress_bar(&bar, Some(u64::try_from(idx)? + 1));
        }

        // Stop progress bar's progress
        self.end_progress_bar(&bar);

        Ok(())
    }
}

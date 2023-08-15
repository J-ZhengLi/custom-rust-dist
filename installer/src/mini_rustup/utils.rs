use std::{env, fs};
use std::path::Path;

use anyhow::{Context, Result};
use sha2::Sha256;
use url::Url;

pub fn download_file(url: &Url, path: &Path, hasher: Option<&mut Sha256>) -> Result<()> {
    download_file_with_resume(url, path, hasher, false)
}

pub(crate) fn download_file_with_resume(
    url: &Url,
    path: &Path,
    hasher: Option<&mut Sha256>,
    resume_from_partial: bool,
) -> Result<()> {
    use download::download_to_path_with_backend;
    use download::{Backend, Event, TlsBackend};
    use sha2::Digest;
    use std::cell::RefCell;

    let hasher = RefCell::new(hasher);

    // This callback will write the download to disk and optionally
    // hash the contents, ~~then forward the notification up the stack~~
    let callback: &dyn Fn(Event<'_>) -> download::Result<()> = &|msg| {
        if let Event::DownloadDataReceived(data) = msg {
            if let Some(h) = hasher.borrow_mut().as_mut() {
                h.update(data);
            }
        }

        Ok(())
    };

    // Download the file
    let res = download_to_path_with_backend(
        Backend::Reqwest(TlsBackend::Rustls),
        url,
        path,
        resume_from_partial,
        Some(callback),
    );

    res
}

pub(crate) fn make_executable(path: &Path) -> Result<()> {
    #[allow(clippy::unnecessary_wraps)]
    #[cfg(windows)]
    fn inner(_: &Path) -> Result<()> {
        Ok(())
    }
    #[cfg(not(windows))]
    fn inner(path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path)?;
        let mut perms = metadata.permissions();
        let mode = perms.mode();
        let new_mode = (mode & !0o777) | 0o755;

        // Check if permissions are ok already - #1638
        if mode == new_mode {
            return Ok(());
        }

        perms.set_mode(new_mode);
        set_permissions(path, perms)
    }

    inner(path)
}

#[cfg(not(windows))]
fn set_permissions(path: &Path, perms: fs::Permissions) -> Result<()> {
    fs::set_permissions(path, perms)
}

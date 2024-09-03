use anyhow::bail;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

/// Get a path to user's "home" directory.
///
/// # Panic
///
/// Will panic if such directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub fn home_dir() -> PathBuf {
    home::home_dir().expect("home directory cannot be determined.")
}

/// Wrapper to [`std::fs::read_to_string`] but with additional error context.
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read '{}'", path.as_ref().display()))
}

pub fn stringify_path<P: AsRef<Path>>(path: P) -> Result<String> {
    path.as_ref()
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "failed to stringify path '{}'",
                path.as_ref().to_string_lossy().to_string()
            )
        })
}

pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if !path.as_ref().is_dir() {
        fs::create_dir_all(path.as_ref()).with_context(|| {
            format!(
                "unable to create specified directory '{}'",
                path.as_ref().display()
            )
        })?;
    }
    Ok(())
}

pub fn ensure_parent_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    if let Some(p) = path.as_ref().parent() {
        ensure_dir(p)?;
    }
    Ok(())
}

pub fn to_nomalized_abspath<P: AsRef<Path>>(path: P, root: Option<&Path>) -> Result<PathBuf> {
    let abs_pathbuf = if path.as_ref().is_absolute() {
        path.as_ref().to_path_buf()
    } else {
        root.map(|p| Ok(p.to_path_buf()))
            .unwrap_or_else(|| env::current_dir().context("current directory cannot be determined"))
            .map(|mut cd| {
                cd.push(path);
                cd
            })?
    };
    // Remove any `.` and `..` from origin path
    let mut nomalized_path = PathBuf::new();
    for path_component in abs_pathbuf.components() {
        match path_component {
            Component::CurDir => (),
            Component::ParentDir => {
                nomalized_path.pop();
            }
            _ => nomalized_path.push(path_component),
        }
    }

    Ok(nomalized_path)
}

pub fn write_file<P: AsRef<Path>>(path: P, content: &str, append: bool) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    if append {
        options.append(true);
    } else {
        options.truncate(true).write(true);
    }
    let mut file = options.create(true).open(path)?;
    writeln!(file, "{content}")?;
    file.sync_data()?;
    Ok(())
}

pub fn write_bytes<P: AsRef<Path>>(path: P, content: &[u8], append: bool) -> Result<()> {
    let mut options = fs::OpenOptions::new();
    if append {
        options.append(true);
    } else {
        options.truncate(true).write(true);
    }
    let mut file = options.create(true).open(path)?;
    file.write_all(content)?;
    file.sync_data()?;
    Ok(())
}

/// Copy a file into an existing directory.
///
/// Returns the path to pasted file.
///
/// # Errors
/// Return `Err` if `to` location does not exist, or [`fs::copy`] operation fails.
pub fn copy_file_to<P, Q>(from: P, to: Q) -> Result<PathBuf>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    assert!(
        from.as_ref().is_file(),
        "Interal Error: '{}' is not a path to file, \
        but `copy_file_to` only works with file path, try using `copy_to` instead.",
        from.as_ref().display()
    );

    copy_into(from, to)
}

/// Copy file or directory into an existing directory.
///
/// Similar to [`copy_file_to`], except this will recursively copy directory as well.
pub fn copy_into<P, Q>(from: P, to: Q) -> Result<PathBuf>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    if !to.as_ref().is_dir() {
        bail!("'{}' is not a directory", to.as_ref().display());
    }

    let dest = to.as_ref().join(from.as_ref().file_name().ok_or_else(|| {
        anyhow!(
            "path '{}' does not have a file name",
            from.as_ref().display()
        )
    })?);

    copy_as(from, &dest)?;
    Ok(dest)
}

/// Copy file or directory to a specified path.
pub fn copy_as<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    fn copy_dir_(src: &Path, dest: &Path) -> Result<()> {
        fs::create_dir(dest)?;
        for maybe_entry in src.read_dir()? {
            let entry = maybe_entry?;
            let src = entry.path();
            let dest = dest.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_dir_(&src, &dest)?;
            } else {
                fs::copy(&src, &dest)?;
            }
        }
        Ok(())
    }

    if !from.as_ref().exists() {
        bail!(
            "failed to copy '{}': path does not exist",
            from.as_ref().display()
        );
    }

    if from.as_ref().is_file() {
        fs::copy(&from, &to).with_context(|| {
            format!(
                "could not copy file '{}' to '{}'",
                from.as_ref().display(),
                to.as_ref().display()
            )
        })?;
    } else {
        copy_dir_(from.as_ref(), to.as_ref()).with_context(|| {
            format!(
                "could not copy directory '{}' to '{}'",
                from.as_ref().display(),
                to.as_ref().display()
            )
        })?;
    }
    Ok(())
}

/// Set file permissions (executable)
/// rwxr-xr-x: 0o755
#[cfg(not(windows))]
pub fn create_executable_file(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    // 设置文件权限为可执行
    let metadata = std::fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755); // rwxr-xr-x
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(windows)]
pub fn create_executable_file(_path: &Path) -> Result<()> {
    Ok(())
}

/// Attempts to read a directory path, then return a list of paths
/// that are inside the given directory, may or may not including sub folders.
pub fn walk_dir(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    fn collect_paths_(dir: &Path, paths: &mut Vec<PathBuf>, recursive: bool) -> Result<()> {
        for dir_entry in dir.read_dir()?.flatten() {
            paths.push(dir_entry.path());
            if recursive && matches!(dir_entry.file_type(), Ok(ty) if ty.is_dir()) {
                collect_paths_(&dir_entry.path(), paths, true)?;
            }
        }
        Ok(())
    }
    let mut paths = vec![];
    collect_paths_(dir, &mut paths, recursive)?;
    Ok(paths)
}

pub fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    #[cfg(windows)]
    let is_executable_ext = matches!(
        path.as_ref().extension().and_then(|ext| ext.to_str()),
        Some("exe")
    );
    #[cfg(not(windows))]
    let is_executable_ext = path.as_ref().extension().is_none();

    path.as_ref().is_file() && is_executable_ext
}

pub fn remove<P: AsRef<Path>>(src: P) -> Result<()> {
    if src.as_ref().is_file() {
        fs::remove_file(&src)
            .with_context(|| format!("unable to remove file '{}'", src.as_ref().display()))?;
    } else if src.as_ref().is_dir() {
        fs::remove_dir_all(&src)
            .with_context(|| format!("unable to remove directory '{}'", src.as_ref().display()))?;
    }
    Ok(())
}

/// Move `src` path to `dest`.
pub fn move_to(src: &Path, dest: &Path, force: bool) -> Result<()> {
    if force && dest.exists() {
        remove(dest)?;
    }

    const RETRY_TIMES: u8 = 20;
    for _ in 0..RETRY_TIMES {
        match fs::rename(src, dest) {
            Ok(()) => return Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            Err(err) => return Err(err.into()),
        }
    }
    // If removing doesn't work, because some stupid problem caused by anti-virus software,
    // try copy and delete.
    copy_as(src, dest)?;
    if remove(src).is_err() {
        println!("{}", t!("remove_path_fail_warn", path = src.display()));
    }

    Ok(())
}

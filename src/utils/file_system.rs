use anyhow::bail;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

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

pub fn mkdirs<P: AsRef<Path>>(path: P) -> Result<()> {
    fs::create_dir_all(path.as_ref()).with_context(|| {
        format!(
            "unable to create specified directory '{}'",
            path.as_ref().display()
        )
    })
}

pub fn to_nomalized_abspath<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<PathBuf> {
    let abs_pathbuf = if path.as_ref().is_absolute() {
        path.into()
    } else {
        env::current_dir()
            .context("current directory cannot be determined")
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

/// Copy a path to an existing directory.
///
/// # Errors
/// Return `Err` if `to` location does not exist, or [`fs::copy`] operation fails.
pub fn copy_to<P, Q>(from: P, to: Q) -> Result<()>
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
    fs::copy(from, dest)?;
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
pub fn create_executable_file(path: &str, content: &str) -> Result<()> {
    Ok(())
}

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context, Result};
use url::Url;

/// Get a path to user's "home" directory.
///
/// # Panic
///
/// Will panic if such directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub(crate) fn home_dir() -> PathBuf {
    home::home_dir().expect("aborting because the home directory cannot be determined.")
}

/// Get a path to the root directory of this program.
///
/// # Panic
///
/// Will panic if the home directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub(crate) fn installer_home() -> PathBuf {
    home_dir().join(env!("CARGO_PKG_NAME"))
}

macro_rules! exec_err {
    ($p:expr, $args:expr, $ext_msg:literal) => {
        anyhow::anyhow!(
            "error occured when executing command `{:?} {:?}`{}",
            $p,
            $args
                .map(|oss| oss.as_ref().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" "),
            $ext_msg
        )
    };
}

/// Execute a command as child process, wait for it to finish then collect its std output.
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
/// 3. The standard output contains non-UTF8 characteres thus cannot be parsed from bytes.
pub(crate) fn standard_output<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let bytes = Command::new(program.as_ref())
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .ok_or_else(|| exec_err!(program.as_ref(), args.iter(), ""))?
        .stdout;

    Ok(String::from_utf8(bytes)?)
}

/// Similar to [`standard_output`], but get first line of the output as string instead
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
/// 3. The standard output contains non-UTF8 characteres thus cannot be parsed from bytes.
/// 4. The output string was empty.
pub(crate) fn standard_output_first_line_only<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = standard_output(program.as_ref(), args)?;
    output
        .lines()
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| exec_err!(program.as_ref(), args.iter(), ": empty output"))
}

/// Wrapper to [`std::fs::read_to_string`] but with additional error context.
pub(crate) fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read '{}'", path.as_ref().display()))
}

pub(crate) fn parse_url(url: &str) -> Result<Url> {
    Url::parse(url).with_context(|| format!("failed to parse url: {url}"))
}

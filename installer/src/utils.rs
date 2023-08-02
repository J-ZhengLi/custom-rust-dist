use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};

use anyhow::{anyhow, bail, Context, Result};

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

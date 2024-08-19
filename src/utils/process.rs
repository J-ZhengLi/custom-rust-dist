use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::process::{Command, Output, Stdio};

use anyhow::{bail, Context, Result};
use cfg_if::cfg_if;

macro_rules! exec_err {
    ($p:expr, $args:expr, $ext_msg:expr) => {
        anyhow::anyhow!(
            "error occured when executing command `{} {}`{}",
            $p.as_ref().to_string_lossy().to_string(),
            $args
                .iter()
                .map(|oss| oss.as_ref().to_string_lossy().to_string())
                .collect::<std::vec::Vec<_>>()
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
pub fn stdout_output<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = output(program.as_ref(), args)?;
    if !output.status.success() {
        bail!(
            "executing `{} {}` returns error: {}",
            program.as_ref().to_string_lossy().to_string(),
            args.iter()
                .map(|oss| oss.as_ref().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" "),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn output_with_env<'a, P, A, I>(program: P, args: &[A], env: I) -> Result<Output>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    Command::new(program.as_ref())
        .args(args)
        .envs(env)
        .output()
        .with_context(|| exec_err!(program, args, ""))
}

/// Execute a command as child process, wait for it to finish, and return its [`Output`].
///
/// # Errors
/// This will return errors if the specific command cannot be execute.
pub fn output<P, A>(program: P, args: &[A]) -> Result<Output>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = output_with_env(program, args, [])?;
    Ok(output)
}

pub fn cmd_exist(cmd: &str) -> bool {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(cmd))
        .any(|p| p.exists())
}

/// Execute a command as child process, wait for it to finish.
pub fn execute<P, A>(program: P, args: &[A]) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
{
    execute_with_env(program, args, [])
}

pub fn execute_with_env<'a, P, A, I>(program: P, args: &[A], envs: I) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let status = Command::new(program.as_ref())
        .args(args)
        .envs(envs)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| exec_err!(program, args, ""))?;

    // 检查子进程的退出状态
    if !status.success() {
        return Err(exec_err!(program, args, ""));
    }

    Ok(())
}

/// Execute by invoking shell program, such as `sh` on Unix, `cmd` on Windows.
pub fn shell_execute<P, A>(program: P, args: &[A]) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
{
    cfg_if! {
        if #[cfg(windows)] {
            let shell = "cmd.exe";
            let start_arg = "/C";
        } else {
            let shell = "sh";
            let start_arg = "-c";
        }
    }

    let status = Command::new(shell)
        .arg(start_arg)
        .arg(&program)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| exec_err!(program, args, ""))?;

    // 检查子进程的退出状态
    if !status.success() {
        return Err(exec_err!(program, args, ""));
    }

    Ok(())
}

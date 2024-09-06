use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        const SHELL: &str = "cmd.exe";
        const START_ARG: &str = "/C";
    } else {
        const SHELL: &str = "sh";
        const START_ARG: &str = "-c";
    }
}

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

/// Check if a command/program exist in the `PATH`.
pub fn cmd_exist(cmd: &str) -> bool {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(cmd))
        .any(|p| p.exists())
}

pub fn execute_for_ret_code<P, A>(program: P, args: &[A]) -> Result<i32>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
{
    shell_execute_with_env(program, args, [], false)
}

/// Execute a commands using [`Command`] api.
///
/// # Platform specific behaviors:
/// - On Windows, this will launch a `cmd.exe` process and invoke the command there.
/// - On Linux, this invoke the command directly.
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
pub fn execute<P, A>(program: P, args: &[A]) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
{
    #[cfg(windows)]
    shell_execute_with_env(program, args, [], true)?;
    #[cfg(not(windows))]
    execute_program_with_env(program, args, [])?;

    Ok(())
}

/// Execute a commands using [`Command`] api, with environment variables.
///
/// # Platform specific behaviors:
/// - On Windows, this will launch a `cmd.exe` process and invoke the command there.
/// - On Linux, this invoke the command directly.
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
pub fn execute_with_env<'a, P, A, I>(program: P, args: &[A], envs: I) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    #[cfg(windows)]
    shell_execute_with_env(program, args, envs, true)?;
    #[cfg(not(windows))]
    execute_program_with_env(program, args, envs)?;

    Ok(())
}

/// Execute commands by directly invoking program, with environment variables.
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
pub fn execute_program_with_env<'a, P, A, I>(program: P, args: &[A], envs: I) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut command = Command::new(program.as_ref());
    command
        .args(args)
        .envs(envs)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Prevent CMD window popup
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);
    }

    let output = command
        .output()
        .with_context(|| exec_err!(program, args, ""))?;
    // 检查子进程的退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(exec_err!(program, args, stderr));
    }

    Ok(())
}

/// Execute commands by invoking shell program, such as `sh` on Unix, `cmd` on Windows.
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
fn shell_execute_with_env<'a, P, A, I>(
    program: P,
    args: &[A],
    vars: I,
    expect_success: bool,
) -> Result<i32>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut command = Command::new(SHELL);
    command
        .arg(START_ARG)
        .arg(&program)
        .args(args)
        .envs(vars)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Prevent CMD window popup
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);
    }

    let output = command
        .output()
        .with_context(|| exec_err!(program, args, ""))?;

    if !expect_success {
        output.status.code().ok_or_else(|| {
            anyhow!(
                "failed to retrive exit code because the program {:?} was terminated by a signal",
                program.as_ref()
            )
        })
    } else {
        // 检查子进程的退出状态
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(exec_err!(program, args, stderr))
    }
}

use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::io::Write;
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result};

macro_rules! exec_err {
    ($p:expr, $args:expr, $ext_msg:expr) => {
        anyhow::anyhow!(
            "error occured when executing command `{:?} {:?}`{}",
            $p.as_ref(),
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
pub fn output<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = Command::new(program.as_ref()).args(args).output()?;
    if !output.status.success() {
        return Err(exec_err!(program, args, "execution failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Similar to [`output`], but get first line of the output as string instead
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
/// 3. The standard output contains non-UTF8 characteres thus cannot be parsed from bytes.
/// 4. The output string was empty.
pub fn output_first_line<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = output(program.as_ref(), args)?;
    output
        .lines()
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| exec_err!(program, args, ": empty output"))
}

pub fn output_with_env<P, A, K, V, I>(program: P, args: &[A], env: I) -> Result<Output>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
    I: IntoIterator<Item = (K, V)>,
{
    Command::new(program.as_ref())
        .args(args)
        .envs(env)
        .output()
        .with_context(|| exec_err!(program, args, ""))
}

pub fn forward_output(output: Output) -> Result<()> {
    std::io::stdout().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;
    Ok(())
}

pub fn cmd_exist(cmd: &str) -> bool {
    let cmd = format!("{cmd}{}", env::consts::EXE_SUFFIX);
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(&cmd))
        .any(|p| p.exists())
}

/// Execute a command as child process, wait for it to finish then collect its std output.
pub fn cmd_output<P, A>(program: P, args: &[A]) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
{
    let child = Command::new(program.as_ref())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to spawn {:?} process.", program));

    let output = child.wait_with_output().expect("Failed to read stdout");

    // 检查子进程的退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "{:?} failed with error: {}",
            program,
            stderr
        ));
    }

    Ok(())
}

/// Execute a command as child process with input, wait for it to finish then collect its std output.
pub fn cmd_output_with_input<P, A>(program: P, args: &[A], input: &[u8]) -> Result<()>
where
    P: AsRef<OsStr> + Debug,
    A: AsRef<OsStr>,
{
    let mut child = Command::new(program.as_ref())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to spawn {:?} process.", program));

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input)
            .unwrap_or_else(|_| panic!("Failed to spawn {:?} process.", program))
    }

    let output = child.wait_with_output().expect("Failed to read stdout");

    // 检查子进程的退出状态
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "{:?} failed with error: {}",
            program,
            stderr
        ));
    }

    Ok(())
}

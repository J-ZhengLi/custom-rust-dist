use log::{error, info, warn};
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus};
use std::{env, io};

use anyhow::Result;

/// Convenient macro to run a [`Command`], check [`cmd`] for help of the syntax.
macro_rules! run {
    ([$($key:tt = $val:expr),*] $program:expr $(, $arg:expr )* $(,)?) => {{
        let cmd__ = $crate::utils::cmd!([$($key=$val),*] $program $(,$arg)*);
        $crate::utils::execute(cmd__).map(|_| ())
    }};
    ($program:expr $(, $arg:expr )* $(,)?) => {
        $crate::utils::run!([] $program $(, $arg)*)
    };
}
pub(crate) use run;

/// Convenient macro to create a [`Command`], using shell-like command syntax.
///
/// # Example
/// ```ignore
/// # use rim::utils::cmd;
/// cmd!("echo", "$HOME/.profile");
///
/// let program = "cargo";
/// cmd!(program, "build", "--release");
///
/// // With env vars
/// cmd!(["FOO"="foo", "BAR"="bar"] program, "cargo", "build");
/// ```
macro_rules! cmd {
    ([$($key:tt = $val:expr),*] $program:expr $(, $arg:expr )* $(,)?) => {{
        let mut cmd__ = std::process::Command::new($program);
        $(cmd__.arg($arg);)*
        $(cmd__.env($key, $val);)*
        cmd__
    }};
    ($program:expr) => {
        std::process::Command::new($program)
    };
    ($program:expr $(, $arg:expr )* $(,)?) => {
        $crate::utils::cmd!([] $program $(, $arg)*)
    };
}
pub(crate) use cmd;

pub(crate) fn execute(cmd: Command) -> Result<()> {
    execute_command(cmd, true).map(|_| ())
}

// Only used for `windows` for now, but... who knows.
#[allow(unused)]
pub(crate) fn execute_for_ret_code(cmd: Command) -> Result<i32> {
    execute_command(cmd, false)
}

fn execute_command(mut cmd: Command, expect_success: bool) -> Result<i32> {
    let (mut reader, stdout) = os_pipe::pipe()?;
    let stderr = stdout.try_clone()?;

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            // Prevent CMD window popup
            use winapi::um::winbase::CREATE_NO_WINDOW;
            let mut child = cmd
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()?;
        } else {
            let mut child = cmd
                .stdout(stdout)
                .stderr(stderr)
                .spawn()?;
        }
    }

    let cmd_content = cmd_to_string(cmd);
    output_to_log(Some(&mut reader));

    let status = child.wait()?;
    let ret_code = get_ret_code(&status);
    if expect_success && !status.success() {
        anyhow::bail!(
            "programm exited with code {ret_code}. \n\
            Command: {cmd_content}"
        );
    } else {
        Ok(ret_code)
    }
}

/// Consumes a [`Command`] and turn it into string using debug formatter.
///
/// It is important to call this before reading the output from `os_pipe`,
/// otherwise there will be deadlock. More information can be found in
/// [`os_pipe`'s documentation](https://docs.rs/os_pipe/1.2.1/os_pipe/#common-deadlocks-related-to-pipes)
fn cmd_to_string(cmd: Command) -> String {
    format!("{cmd:?}")
}

fn get_ret_code(status: &ExitStatus) -> i32 {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            // status code can only be `None` on Unix
            status.code().unwrap()
        } else {
            use std::os::unix::process::ExitStatusExt;
            status.into_raw()
        }
    }
}

/// Log the command output
fn output_to_log<R: io::Read>(from: Option<&mut R>) {
    let Some(out) = from else { return };
    let reader = BufReader::new(out);
    for line in reader.lines().map_while(Result::ok) {
        // prevent double 'info|warn|error:' labels, although this might be a dumb way to do it
        if let Some(info) = line.strip_prefix("info: ") {
            info!("{info}");
        } else if let Some(warn) = line.strip_prefix("warn: ") {
            warn!("{warn}");
        } else if let Some(error) = line.strip_prefix("error: ") {
            error!("{error}");
        } else if !line.is_empty() {
            info!("{line}");
        }
    }
}

/// Check if a command/program exist in the `PATH`.
pub fn cmd_exist<S: AsRef<str>>(cmd: S) -> bool {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(cmd.as_ref()))
        .any(|p| p.exists())
}

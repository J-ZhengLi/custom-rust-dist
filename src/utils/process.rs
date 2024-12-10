use log::{error, info, warn};
use std::ffi::OsStr;
use std::process::{ExitStatus, Stdio};
use std::sync::Mutex;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command as AsyncCommand;

use anyhow::Result;

use super::to_string_lossy;

/// The complete commands in string form, used in error output.
static COMMAND_STRING: Mutex<Vec<String>> = Mutex::new(Vec::new());

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        const SHELL: &str = "cmd.exe";
        const START_ARG: &str = "/C";
    } else {
        const SHELL: &str = "sh";
        const START_ARG: &str = "-c";
    }
}

/// A [`std::process::Command`] wrapper type that supports better error reporting.
pub struct Command {
    cmd_: AsyncCommand,
}

impl Command {
    pub fn new<P: AsRef<OsStr>>(program: P) -> Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        *guard = vec![to_string_lossy(&program)];

        Self {
            cmd_: AsyncCommand::new(program),
        }
    }
    /// Create a command that will be execute using a separated shell.
    ///
    /// This prevents a specific program being shut down because the main process exists.
    pub fn new_shell_command<P: AsRef<OsStr>>(program: P) -> Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        *guard = vec![to_string_lossy(&program)];

        let mut inner = AsyncCommand::new(SHELL);
        inner.arg(START_ARG).arg(program);

        Self { cmd_: inner }
    }
    pub fn arg<A: AsRef<OsStr>>(&mut self, arg: A) -> &mut Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        (*guard).push(to_string_lossy(&arg));

        self.cmd_.arg(arg);
        self
    }
    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> &mut Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        for arg in args {
            (*guard).push(to_string_lossy(arg));
        }

        self.cmd_.args(args);
        self
    }
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let mut guard = COMMAND_STRING.lock().unwrap();
        (*guard).insert(
            0,
            format!("{}={}", to_string_lossy(&key), to_string_lossy(&val)),
        );

        self.cmd_.env(key, val);
        self
    }
    pub async fn run(&mut self) -> Result<()> {
        self.execute_command(true).await?;
        Ok(())
    }

    pub async fn run_with_ret_code(&mut self) -> Result<i32> {
        self.execute_command(false).await
    }

    async fn execute_command(&mut self, expect_success: bool) -> Result<i32> {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                use std::os::windows::process::CommandExt;
                // Prevent CMD window popup
                use winapi::um::winbase::CREATE_NO_WINDOW;
                let mut child = self.cmd_
                    .creation_flags(CREATE_NO_WINDOW)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()?;
            } else {
                let mut child = self.cmd_
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true)
                    .spawn()?;
            }
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let handle = tokio::spawn(async move {
            let status = child.wait().await?;
            let ret_code = get_ret_code(&status);
            if expect_success && !status.success() {
                let command = COMMAND_STRING.lock().unwrap();
                anyhow::bail!(
                    "programm exited with code {ret_code}. \n\
                    Command: {}",
                    (*command).join(" "),
                );
            } else {
                Ok(ret_code)
            }
        });

        output_to_log(stdout).await?;
        output_to_log(stderr).await?;

        handle
            .await
            .expect("async task failed to execute to completion")
    }
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
async fn output_to_log<R: AsyncRead + Unpin>(from: Option<R>) -> Result<()> {
    let Some(out) = from else { return Ok(()) };
    let mut lines = BufReader::new(out).lines();
    while let Some(line) = lines.next_line().await? {
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
    Ok(())
}

/// Check if a command/program exist in the `PATH`.
pub fn cmd_exist<S: AsRef<str>>(cmd: S) -> bool {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(cmd.as_ref()))
        .any(|p| p.exists())
}

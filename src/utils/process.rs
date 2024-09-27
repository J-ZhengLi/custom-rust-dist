use std::env;
use std::ffi::OsStr;
use std::process::{Command as StdCommand, Stdio};
use std::sync::Mutex;

use anyhow::Result;

use super::to_string_lossy;

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
pub struct Command(StdCommand);

impl Command {
    pub fn new<P: AsRef<OsStr>>(program: P) -> Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        *guard = vec![to_string_lossy(&program)];

        Self(StdCommand::new(program))
    }
    /// Create a command that will be execute using a separated shell.
    ///
    /// This prevents a specific program being shut down because the main process exists.
    pub fn new_shell_command<P: AsRef<OsStr>>(program: P) -> Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        *guard = vec![to_string_lossy(&program)];

        let mut inner = StdCommand::new(SHELL);
        inner.arg(START_ARG).arg(program);

        Self(inner)
    }
    pub fn arg<A: AsRef<OsStr>>(&mut self, arg: A) -> &mut Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        (*guard).push(to_string_lossy(&arg));

        self.0.arg(arg);
        self
    }
    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> &mut Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        for arg in args {
            (*guard).push(to_string_lossy(arg));
        }

        self.0.args(args);
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

        self.0.env(key, val);
        self
    }
    /// Use [`Stdio::inherit`] for standard error output.
    ///
    /// For some program, such as `rustup` or `cargo`, putting `info:` messages in `stderr` (WHY!!!),
    /// therefore we can specify this to output those `info` as well, but this will causing
    /// the actually error not showing when error occurs.
    pub fn inherit_stderr(&mut self) -> &mut Command {
        self.0.stderr(Stdio::inherit());
        self
    }

    pub fn run(&mut self) -> Result<()> {
        execute_command(&mut self.0, true)?;
        Ok(())
    }

    pub fn run_with_ret_code(&mut self) -> Result<i32> {
        execute_command(&mut self.0, false)
    }
}

/// Check if a command/program exist in the `PATH`.
pub fn cmd_exist(cmd: &str) -> bool {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(cmd))
        .any(|p| p.exists())
}

fn execute_command(command: &mut StdCommand, expect_success: bool) -> Result<i32> {
    command.stdout(Stdio::inherit());

    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            use winapi::um::winbase::CREATE_NO_WINDOW;
            // Prevent CMD window popup
            let output = command.creation_flags(CREATE_NO_WINDOW).output()?;
            let ret_code = output.status.code().unwrap();
        } else {
            use std::os::unix::process::ExitStatusExt;
            let output = command.output()?;
            let ret_code = output.status.into_raw();
        }
    }

    if expect_success && !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // stderr might be empty if using `Stdio::inherit()`
        let err = if stderr.is_empty() {
            "Undocumented error, check log for more details"
        } else {
            &*stderr
        };
        let command = COMMAND_STRING.lock().unwrap();
        anyhow::bail!(
            "programm exited with code {ret_code}. \n\
            Command: {}
            Error output: {err}",
            (*command).join(" "),
        );
    } else {
        Ok(ret_code)
    }
}

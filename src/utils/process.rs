use std::ffi::OsStr;
use std::process::Command as StdCommand;
use std::sync::Mutex;
use std::{env, fs, io};

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
    cmd_: StdCommand,
    log_output: bool,
}

impl Command {
    pub fn new<P: AsRef<OsStr>>(program: P) -> Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        *guard = vec![to_string_lossy(&program)];

        Self {
            cmd_: StdCommand::new(program),
            log_output: false,
        }
    }
    /// Create a command that will be execute using a separated shell.
    ///
    /// This prevents a specific program being shut down because the main process exists.
    pub fn new_shell_command<P: AsRef<OsStr>>(program: P) -> Self {
        let mut guard = COMMAND_STRING.lock().unwrap();
        *guard = vec![to_string_lossy(&program)];

        let mut inner = StdCommand::new(SHELL);
        inner.arg(START_ARG).arg(program);

        Self {
            cmd_: inner,
            log_output: false,
        }
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
    /// Set a flag to write command output (including `stdout` and `stderr`)
    /// to a [log file](super::log_file_path).
    pub fn output_to_file(&mut self) -> &mut Command {
        self.log_output = true;
        self
    }

    pub fn run(&mut self) -> Result<()> {
        self.execute_command(true)?;
        Ok(())
    }

    pub fn run_with_ret_code(&mut self) -> Result<i32> {
        self.execute_command(false)
    }

    fn execute_command(&mut self, expect_success: bool) -> Result<i32> {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                use std::os::windows::process::CommandExt;
                use winapi::um::winbase::CREATE_NO_WINDOW;
                // Prevent CMD window popup
                let output = self.cmd_.creation_flags(CREATE_NO_WINDOW).output()?;
                let ret_code = output.status.code().unwrap();
            } else {
                use std::os::unix::process::ExitStatusExt;
                let output = self.cmd_.output()?;
                let ret_code = output.status.into_raw();
            }
        }

        // manually copy output to standard pipeline.
        io::copy(&mut output.stderr.as_slice(), &mut io::stderr())?;
        io::copy(&mut output.stdout.as_slice(), &mut io::stdout())?;

        if self.log_output {
            let mut log_file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(super::log_file_path()?)?;
            io::copy(&mut output.stderr.as_slice(), &mut log_file)?;
            io::copy(&mut output.stdout.as_slice(), &mut log_file)?;
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
}

/// Check if a command/program exist in the `PATH`.
pub fn cmd_exist<S: AsRef<str>>(cmd: S) -> bool {
    let path = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&path)
        .map(|p| p.join(cmd.as_ref()))
        .any(|p| p.exists())
}

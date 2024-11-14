use anyhow::Result;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Logger {
    output_sender: Option<Sender<String>>,
    dispatcher_: fern::Dispatch,
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    pub fn new() -> Self {
        #[cfg(not(debug_assertions))]
        let level = LevelFilter::Info;
        #[cfg(debug_assertions)]
        let level = LevelFilter::Debug;

        Self {
            output_sender: None,
            dispatcher_: fern::Dispatch::new().level(level),
        }
    }
    /// Set verbose output, this will print `debug!` messages as well.
    pub fn verbose(mut self, v: bool) -> Self {
        if v {
            self.dispatcher_ = self.dispatcher_.level(LevelFilter::Debug);
        }
        self
    }
    /// Ignore most output, keep only the `error` messages.
    pub fn quiet(mut self, q: bool) -> Self {
        if q {
            self.dispatcher_ = self.dispatcher_.level(LevelFilter::Error);
        }
        self
    }
    /// Send output using a specific sender rather than printing on `stdout`.
    pub fn sender(mut self, sender: Sender<String>) -> Self {
        self.output_sender = Some(sender);
        self
    }

    /// Setup logger using [`log`] and [`fern`], this must be called first before
    /// any of the `info!`, `warn!`, `trace!`, `debug!`, `error!` macros.
    ///
    /// - If [`verbose`](Logger::verbose) was called with `true`, this will output more
    ///     detailed log messages including `debug!`.
    /// - If [`quiet`](Logger::quiet) was called with `true`, this will not output any message
    ///     on `stdout`, but will still output them into log file.
    pub fn setup(self) -> Result<()> {
        // decide if `Sender` or `Stdout` should be used as message medium.
        let output = if let Some(sender) = self.output_sender {
            self.dispatcher_
                .format(|out, msg, rec| {
                    out.finish(format_args!(
                        "{}: {msg}",
                        rec.level().to_string().to_lowercase()
                    ));
                })
                .chain(sender)
        } else {
            self.dispatcher_
                .format(|out, msg, rec| {
                    out.finish(format_args!(
                        "{}: {msg}",
                        ColoredLevelConfig::new()
                            .info(Color::BrightBlue)
                            .debug(Color::Magenta)
                            .color(rec.level())
                            .to_string()
                            .to_lowercase(),
                    ));
                })
                .chain(io::stdout())
        };

        let file_config = fern::Dispatch::new()
            .format(|out, msg, rec| {
                out.finish(format_args!(
                    "[{} {}] {msg}",
                    Local::now().to_rfc3339(),
                    rec.target(),
                ))
            })
            .chain(fern::log_file(log_file_path()?)?);

        output.chain(file_config).apply()?;
        Ok(())
    }
}

static LOG_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();
/// Get the path to log file to write.
///
/// We put the log directory besides current binary, so that it should be easier for users to find them.
/// If for some reason the path to current binary cannot be found, we'll assume the user is running this
/// binary in their current working dir, and create a log dir there.
///
/// Note: the log file might not exists.
///
/// # Error
///
/// Because this will attemp to create a directory named `log` to place the actual log file,
/// this function might fail if it cannot be created.
pub fn log_file_path() -> Result<&'static Path> {
    let mut log_dir = super::parent_dir_of_cur_exe().unwrap_or(PathBuf::from("."));
    log_dir.push("log");
    super::ensure_dir(&log_dir)?;

    let bin_name = super::lowercase_program_name().unwrap_or(env!("CARGO_PKG_NAME").to_string());

    Ok(LOG_FILE_PATH
        .get_or_init(|| log_dir.join(format!("{bin_name}-{}.log", Local::now().date_naive()))))
}

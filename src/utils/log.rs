use anyhow::Result;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Logger {
    output_sender: Option<Sender<String>>,
    dispatcher_: fern::Dispatch,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            output_sender: None,
            dispatcher_: fern::Dispatch::new().level(LevelFilter::Info),
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
    /// - If [`log_dir`](Logger::log_dir) was called with a valid directory path, this will attempt
    ///     to create a log file under that directory, and output log messages there.
    /// - If [`verbose`](Logger::verbose) was called with `true`, this will output more
    ///     detailed log messages including `debug!`.
    /// - If [`quiet`](Logger::quiet) was called with `true`, this will not output any message
    ///     on `stdout`, but may output them into log file instead.
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

        // decided where should we output log file as backup.
        let mut log_dir = super::parent_dir_of_cur_exe().unwrap_or(PathBuf::from("."));
        log_dir.push("log");
        super::ensure_dir(&log_dir)?;
        let log_path = log_dir.join(format!("{}.log", Local::now().date_naive()));

        let file_config = fern::Dispatch::new()
            .format(|out, msg, rec| {
                out.finish(format_args!(
                    "[{} {}] {msg}",
                    Local::now().to_rfc3339(),
                    rec.target(),
                ))
            })
            .chain(fern::log_file(log_path)?);

        output.chain(file_config).apply()?;
        Ok(())
    }
}

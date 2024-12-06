use anyhow::Result;
use tracing::level_filters::LevelFilter;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static FILE_LOG_HANDLE: OnceLock<WorkerGuard> = OnceLock::new();
static LOG_DIR_PATH: OnceLock<PathBuf> = OnceLock::new();

pub struct Logger {
    level_filter: LevelFilter,
}

impl Logger {
    pub fn new() -> Self {
        #[cfg(not(debug_assertions))]
        let default_level = LevelFilter::INFO;
        #[cfg(debug_assertions)]
        let default_level = LevelFilter::DEBUG;

        Self {
            level_filter: default_level,
        }
    }
    /// Set verbose output, this will print `debug!` messages as well.
    pub fn verbose(mut self, v: bool) -> Self {
        if v {
            self.level_filter = LevelFilter::DEBUG;
        }
        self
    }
    /// Ignore most output, keep only the `error` messages.
    pub fn quiet(mut self, q: bool) -> Self {
        if q {
            self.level_filter = LevelFilter::ERROR;
        }
        self
    }
    // /// Send output using a specific sender rather than printing on `stdout`.
    // pub fn sender(mut self, sender: Sender<String>) -> Self {
    //     self.output_sender = Some(sender);
    //     self
    // }

    /// Setup logger using [`log`] and [`fern`], this must be called first before
    /// any of the `info!`, `warn!`, `trace!`, `debug!`, `error!` macros.
    ///
    /// - If [`verbose`](Logger::verbose) was called with `true`, this will output more
    ///     detailed log messages including `debug!`.
    /// - If [`quiet`](Logger::quiet) was called with `true`, this will not output any message
    ///     on `stdout`, but will still output them into log file.
    pub fn setup(self) -> Result<()> {
        let registry = Registry::default()
            .with(file_logger()?);
        // decide if `Sender` or `Stdout` should be used as message medium.
        // let output = if let Some(sender) = self.output_sender {
        //     self.dispatcher_
        //         .format(|out, msg, rec| {
        //             out.finish(format_args!(
        //                 "{}: {msg}",
        //                 rec.level().to_string().to_lowercase()
        //             ));
        //         })
        //         .chain(sender)
        // } else {
        //     self.dispatcher_
        //         .format(|out, msg, rec| {
        //             out.finish(format_args!(
        //                 "{}: {msg}",
        //                 ColoredLevelConfig::new()
        //                     .info(Color::BrightBlue)
        //                     .debug(Color::Magenta)
        //                     .color(rec.level())
        //                     .to_string()
        //                     .to_lowercase(),
        //             ));
        //         })
        //         .chain(io::stdout())
        // };

        // let file_config = fern::Dispatch::new()
        //     .format(|out, msg, rec| {
        //         out.finish(format_args!(
        //             "[{} {}] {msg}",
        //             Local::now().to_rfc3339(),
        //             rec.target(),
        //         ))
        //     })
        //     .chain(fern::log_file(log_file_path()?)?);

        // output.chain(file_config).apply()?;
        Ok(())
    }
}

fn file_logger<S>() -> Result<impl Layer<S>>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let prefix = super::lowercase_program_name()
        .unwrap_or(env!("CARGO_PKG_NAME").to_string());
    let appender =tracing_appender::rolling::daily(log_dir(), prefix);
    let (writer, guard) = tracing_appender::non_blocking(appender);

    // The `WorkerGuard` is essential for outputing log to file,
    // we can no longer do that if it ended up being dropped.
    // Therefore we lock it as static, keeping it alive during the entire process.
    FILE_LOG_HANDLE
        .set(guard)
        .expect("internal error: file logger should be configured exactly once");

    Ok(tracing_subscriber::fmt::Layer::default().with_writer(writer))
}

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
pub fn log_dir() -> &'static Path {
    let mut log_dir = super::parent_dir_of_cur_exe().unwrap_or(PathBuf::from("."));
    log_dir.push("log");
    if super::ensure_dir(&log_dir).is_err() {
        panic!(
            "unable to create log directory, \
            try manually create '{}' then run this program again.",
            log_dir.display()
        );
    }

    LOG_DIR_PATH.get_or_init(|| log_dir)
}

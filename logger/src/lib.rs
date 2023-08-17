pub mod color;

use crate::color::ColoredStr;
use log::{LevelFilter, Log};
use time::{format_description, OffsetDateTime};

#[macro_export]
macro_rules! warn {
    ( $ ( $ arg : tt ) * ) => ( log::warn! ( $ ( $ arg ) * ) )
}
#[macro_export]
macro_rules! err {
    ( $ ( $ arg : tt ) * ) => ( log::error! ( $ ( $ arg ) * ) )
}
#[macro_export]
macro_rules! info {
    ( $ ( $ arg : tt ) * ) => ( log::info! ( $ ( $ arg ) * ) )
}
#[macro_export]
macro_rules! debug {
    ( $ ( $ arg : tt ) * ) => ( log::debug! ( $ ( $ arg ) * ) )
}
#[macro_export]
macro_rules! trace {
    ( $ ( $ arg : tt ) * ) => ( log::trace! ( $ ( $ arg ) * ) )
}

pub struct Logger {
    level: LevelFilter,
    ts_format: String,
    colored: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self::new_with_level(LevelFilter::Trace)
    }

    pub fn new_with_level(level: LevelFilter) -> Self {
        Logger {
            level,
            ts_format: "[hour]:[minute]:[second].[subsecond digits:3]".to_string(),
            colored: false,
        }
    }

    pub fn colored(mut self) -> Self {
        self.colored = true;
        self
    }

    pub fn ts_format(mut self, f: &str) -> Self {
        self.ts_format = f.to_string();
        self
    }

    pub fn init(self) -> Result<(), log::SetLoggerError> {
        log::set_max_level(self.level);
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let record_lvl = record.level().to_level_filter();
            // Extra information including timestamp and module level location
            let trace_info = match record_lvl {
                LevelFilter::Trace | LevelFilter::Debug => {
                    let formattable = format_description::parse(&self.ts_format)
                        .expect("Fail to format time string.");
                    let timestamp = OffsetDateTime::now_local()
                        .expect("Fail to get local time")
                        .format(&formattable)
                        .unwrap_or_default();
                    let target = if record.target().is_empty() {
                        record.module_path().unwrap_or_default()
                    } else {
                        record.target()
                    };
                    format!("[{timestamp}]@{target} ")
                }
                _ => String::new(),
            };
            // a info|error|warning|debug|trace: label before messages
            let mut level_string = record.level().to_string().to_lowercase();
            if self.colored {
                let mut colored_str = ColoredStr::from(level_string);
                level_string = match record_lvl {
                    LevelFilter::Error => colored_str.bright().color(color::Color::Red).build(),
                    LevelFilter::Warn => colored_str.bright().color(color::Color::Yellow).build(),
                    LevelFilter::Debug => colored_str.bold().build(),
                    LevelFilter::Info => colored_str.bright().build(),
                    _ => colored_str.build(),
                }
            }

            let msg = record.args().to_string();
            let msg = msg.trim_start_matches('\n');
            match record_lvl {
                LevelFilter::Error => eprintln!("{trace_info}{level_string}: {msg}"),
                _ => println!("{trace_info}{level_string}: {msg}"),
            };
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::color::{Color, ColoredStr};

    #[test]
    fn test_color_value() {
        assert_eq!(Color::Red as u8, 31);
    }

    #[test]
    fn test_color() {
        let colored_string = ColoredStr::new()
            .content("a red string")
            .color(Color::Red)
            .build();
        assert_eq!(colored_string, "\x1b[31ma red string\x1b[0m");
    }
}

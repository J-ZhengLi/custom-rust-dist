//! Progress bar indicator for commandline user interface.

use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

/// Convinent struct with methods that are useful to indicate download progress.
#[derive(Debug, Clone, Copy)]
pub struct ProgressIndicator<T: Sized> {
    /// A start/initializing function which will be called once before downloading.
    pub start: fn(u64, String, Style) -> Result<T>,
    /// A update function that will be called after each downloaded chunk.
    pub update: fn(&T, u64),
    /// A function that will be called once after a successful download.
    pub stop: fn(&T, String),
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Style {
    /// Display the progress base on number of bytes.
    Bytes,
    #[default]
    /// Display the progress base on position & length parameters.
    Len,
}

impl Style {
    fn template_str(&self) -> &str {
        match self {
            Style::Bytes => "{bytes}/{total_bytes}",
            Style::Len => "{pos}/{len}",
        }
    }
}

// TODO: Mark this with cfg(feature = "cli")
impl ProgressIndicator<ProgressBar> {
    /// Create a new progress bar for CLI to indicate download progress.
    ///
    /// `progress_for`: used for displaying what the progress is for.
    /// i.e.: ("downloading", "download"), ("extracting", "extraction"), etc.
    pub fn new() -> Self {
        fn start(total: u64, msg: String, style: Style) -> Result<ProgressBar> {
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::with_template(
                    &format!("{{msg}}\n{{spinner:.green}}] [{{elapsed_precise}}] [{{wide_bar:.cyan/blue}}] {} ({{eta}})", style.template_str())
                )?
                .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).expect("unable to display progress bar")
                })
                .progress_chars("#>-")
            );
            pb.set_message(msg);
            Ok(pb)
        }
        fn update(pb: &ProgressBar, pos: u64) {
            pb.set_position(pos);
        }
        fn stop(pb: &ProgressBar, msg: String) {
            pb.finish_with_message(msg);
        }

        ProgressIndicator {
            start,
            update,
            stop,
        }
    }
}

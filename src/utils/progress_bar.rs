//! Progress bar indicator for commandline user interface.

use std::sync::mpsc::Sender;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

#[derive(Debug, Clone, Copy, Default)]
/// Help to send install progress across threads.
pub struct MultiThreadProgress<'a> {
    msg_sender: Option<&'a Sender<String>>,
    prog_sender: Option<&'a Sender<usize>>,
    pub val: usize,
    cur_progress: usize,
}

impl<'a> MultiThreadProgress<'a> {
    pub fn new(
        msg_sender: &'a Sender<String>,
        progress_sender: &'a Sender<usize>,
        initial_progress: usize,
    ) -> Self {
        Self {
            msg_sender: Some(msg_sender),
            prog_sender: Some(progress_sender),
            cur_progress: initial_progress,
            ..Default::default()
        }
    }
    pub fn send_msg(&self, msg: String) -> Result<()> {
        if let Some(sender) = self.msg_sender {
            sender.send(msg)?;
        }
        Ok(())
    }
    pub fn send_progress(&mut self) -> Result<()> {
        if let Some(sender) = self.prog_sender {
            self.cur_progress = (self.cur_progress + self.val).min(100);
            sender.send(self.cur_progress)?;
        }
        Ok(())
    }
    pub fn send_any_progress(&mut self, prog: usize) -> Result<()> {
        self.val = prog;
        self.send_progress()
    }
}

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

use std::fmt::Write;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::utils::DownloadIndicator;

pub(crate) fn progress_bar_indicator() -> DownloadIndicator<ProgressBar> {
    fn start(total: u64, name: &str) -> Result<ProgressBar> {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template(
                "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )?
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).expect("unable to display progress bar"))
            .progress_chars("#>-")
        );
        pb.set_message(format!("Downloading '{name}'"));
        Ok(pb)
    }
    fn update(pb: &ProgressBar, pos: u64) {
        pb.set_position(pos);
    }
    fn stop(pb: &ProgressBar) {
        pb.finish_with_message("Download finished");
    }

    DownloadIndicator {
        start,
        update,
        stop,
    }
}

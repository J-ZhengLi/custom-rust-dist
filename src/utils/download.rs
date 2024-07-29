use std::cmp::min;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::USER_AGENT;
use reqwest::Proxy;
use url::Url;

fn client_builder() -> ClientBuilder {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .connection_verbose(false)
}

/// Convinent struct with methods that are useful to indicate download progress.
pub struct DownloadIndicator<T: Sized> {
    /// A start/initializing function which will be called once before downloading.
    pub start: fn(u64, &str) -> Result<T>,
    /// A update function that will be called after each downloaded chunk.
    pub update: fn(&T, u64),
    /// A function that will be called once after a successful download.
    pub stop: fn(&T),
}

pub struct DownloadOpt<T: Sized> {
    /// The verbose name of the file to download.
    pub name: String,
    client: Client,
    pub handler: Option<DownloadIndicator<T>>,
}

impl<T: Sized> DownloadOpt<T> {
    pub fn new(
        name: String,
        proxy: Option<Proxy>,
        handler: Option<DownloadIndicator<T>>,
    ) -> Result<Self> {
        let client = if let Some(proxy) = proxy {
            client_builder().proxy(proxy).build()?
        } else {
            client_builder().build()?
        };
        Ok(Self {
            name,
            client,
            handler,
        })
    }
    // TODO: make local file download fancier
    pub fn download_file(&self, url: &Url, path: &Path, resume: bool) -> Result<()> {
        if url.scheme() == "file" {
            fs::copy(
                url.to_file_path().map_err(|_| {
                    anyhow!("unable to convert to file path for url '{}'", url.as_str())
                })?,
                path,
            )?;
            return Ok(());
        }

        let mut resp = self
            .client
            .get(url.as_ref())
            .header(USER_AGENT, env!("CARGO_PKG_NAME"))
            .send()?;
        let status = resp.status();
        if !status.is_success() {
            bail!(
                "failed to receive surver response when downloading '{}': {status}",
                self.name
            );
        }
        let total_size = resp
            .content_length()
            .ok_or_else(|| anyhow!("unable to get file length of '{}'", url.as_str()))?;

        let maybe_indicator = self
            .handler
            .as_ref()
            .and_then(|h| (h.start)(total_size, &self.name).ok());

        let (mut downloaded_len, mut file) = if resume {
            let file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open(path)?;
            (file.metadata()?.len().saturating_sub(1), file)
        } else {
            (
                0,
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)?,
            )
        };

        let mut buffer = vec![0u8; 65535];

        loop {
            let bytes_read = io::Read::read(&mut resp, &mut buffer)?;

            if bytes_read != 0 {
                downloaded_len = min(downloaded_len + bytes_read as u64, total_size);
                if let Some(indicator) = &maybe_indicator {
                    // safe to unwrap, because indicator won't exist if self.handler is none
                    (self.handler.as_ref().unwrap().update)(indicator, downloaded_len);
                }
                file.write_all(&buffer[..bytes_read])?;
            } else {
                if let Some(indicator) = &maybe_indicator {
                    // safe to unwrap, because indicator won't exist if self.handler is none
                    (self.handler.as_ref().unwrap().stop)(indicator);
                }

                return Ok(());
            }
        }
    }
}

pub mod cli {
    use std::path::Path;

    use super::{DownloadIndicator, DownloadOpt, Result};
    use indicatif::{ProgressBar, ProgressState, ProgressStyle};
    use url::Url;

    /// [CLI only] Create a new progress bar to indicate download progress.
    pub fn progress_bar_indicator() -> DownloadIndicator<ProgressBar> {
        fn start(total: u64, name: &str) -> Result<ProgressBar> {
            let pb = ProgressBar::new(total);
            pb.set_style(
                ProgressStyle::with_template(
                    "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
                )?
                .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).expect("unable to display progress bar")
                })
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

    /// Download a file without resuming.
    pub fn download_from_start<S: ToString>(name: S, url: &Url, dest: &Path) -> Result<()> {
        let dl_opt = DownloadOpt::new(
            name.to_string(),
            // Keep proxy to `None` for now.
            None,
            Some(progress_bar_indicator()),
        )?;
        dl_opt.download_file(url, dest, false)
    }
}

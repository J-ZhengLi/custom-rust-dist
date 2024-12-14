use std::cmp::min;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking::{Client, ClientBuilder};
use url::Url;

use super::progress_bar::{CliProgress, Style};
use crate::toolset_manifest::Proxy as CrateProxy;

fn client_builder() -> ClientBuilder {
    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let default_proxy = reqwest::Proxy::custom(|url| env_proxy::for_url(url).to_url())
        .no_proxy(reqwest::NoProxy::from_env());
    Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(30))
        .connection_verbose(false)
        .proxy(default_proxy)
}

pub struct DownloadOpt<T: Sized> {
    /// The verbose name of the file to download.
    pub name: String,
    client: Client,
    pub handler: Option<CliProgress<T>>,
}

impl<T: Sized> DownloadOpt<T> {
    pub fn new<S: ToString>(name: S) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            client: client_builder().build()?,
            handler: None,
        })
    }
    pub fn proxy(&mut self, proxy: Option<CrateProxy>) -> Result<&mut Self> {
        // rebuild client with given proxy, otherwise do nothing
        if let Some(p) = proxy {
            self.client = client_builder().proxy(p.try_into()?).build()?;
        }

        Ok(self)
    }
    pub fn handler(&mut self, progress_handler: Option<CliProgress<T>>) -> &mut Self {
        self.handler = progress_handler;
        self
    }
    /// Retrive text response by sending request to a given url.
    ///
    /// If the `url` is a local file, this will use [`read_to_string`](fs::read_to_string) to
    /// get the text instead.
    pub fn read(&self, url: &Url) -> Result<String> {
        if url.scheme() == "file" {
            let file_url = url
                .to_file_path()
                .map_err(|_| anyhow!("file url does not exist"))?;
            return fs::read_to_string(&file_url).with_context(|| {
                format!(
                    "unable to read {} located in {}",
                    self.name,
                    file_url.display()
                )
            });
        }

        let resp = self
            .client
            .get(url.as_ref())
            .send()
            .with_context(|| format!("failed to receive surver response from '{url}'"))?;
        if resp.status().is_success() {
            Ok(resp.text()?)
        } else {
            bail!(
                "unable to get text content of url '{url}': server responded with error {}",
                resp.status()
            );
        }
    }
    // TODO: make local file download fancier
    pub fn download_file(&self, url: &Url, path: &Path, resume: bool) -> Result<()> {
        if url.scheme() == "file" {
            fs::copy(
                url.to_file_path()
                    .map_err(|_| anyhow!("unable to convert to file path for url '{url}'"))?,
                path,
            )?;
            return Ok(());
        }

        let mut resp = self.client.get(url.as_ref()).send().with_context(|| {
            format!("failed to receive surver response when downloading from '{url}'")
        })?;
        let status = resp.status();
        if !status.is_success() {
            bail!("server returns error when attempting download from '{url}': {status}");
        }
        let total_size = resp
            .content_length()
            .ok_or_else(|| anyhow!("unable to get file length of '{url}'"))?;

        let maybe_indicator = self.handler.as_ref().and_then(|h| {
            (h.start)(
                format!("downloading '{}'", &self.name),
                Style::Bytes(total_size),
            )
            .ok()
        });

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
                    (self.handler.as_ref().unwrap().update)(indicator, Some(downloaded_len));
                }
                file.write_all(&buffer[..bytes_read])?;
            } else {
                if let Some(indicator) = &maybe_indicator {
                    // safe to unwrap, because indicator won't exist if self.handler is none
                    (self.handler.as_ref().unwrap().stop)(
                        indicator,
                        format!("'{}' successfully downloaded.", &self.name),
                    );
                }

                return Ok(());
            }
        }
    }
}

/// Download a file without resuming, with proxy settings.
pub fn download_with_proxy<S: ToString>(
    name: S,
    url: &Url,
    dest: &Path,
    proxy: Option<&CrateProxy>,
) -> Result<()> {
    DownloadOpt::new(name)?
        .proxy(proxy.cloned())?
        .handler(Some(CliProgress::new()))
        .download_file(url, dest, false)
}

/// Download a file without resuming, with default proxy settings (from env).
pub fn download<S: ToString>(name: S, url: &Url, dest: &Path) -> Result<()> {
    DownloadOpt::new(name)?
        .handler(Some(CliProgress::new()))
        .download_file(url, dest, false)
}

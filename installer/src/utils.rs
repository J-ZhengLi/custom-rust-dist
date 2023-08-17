use std::cmp::min;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::{header, Proxy, StatusCode};
use url::Url;

/// Get a path to user's "home" directory.
///
/// # Panic
///
/// Will panic if such directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub(crate) fn home_dir() -> PathBuf {
    home::home_dir().expect("aborting because the home directory cannot be determined.")
}

/// Get a path to the root directory of this program.
///
/// # Panic
///
/// Will panic if the home directory cannot be determined,
/// which could be the result of missing certain environment variable at runtime,
/// check [`home::home_dir`] for more information.
pub(crate) fn installer_home() -> PathBuf {
    home_dir().join(env!("CARGO_PKG_NAME"))
}

macro_rules! exec_err {
    ($p:expr, $args:expr, $ext_msg:expr) => {
        anyhow::anyhow!(
            "error occured when executing command `{:?} {:?}`{}",
            $p.as_ref(),
            $args
                .iter()
                .map(|oss| oss.as_ref().to_string_lossy().to_string())
                .collect::<std::vec::Vec<_>>()
                .join(" "),
            $ext_msg
        )
    };
}

/// Execute a command as child process, wait for it to finish then collect its std output.
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
/// 3. The standard output contains non-UTF8 characteres thus cannot be parsed from bytes.
pub(crate) fn standard_output<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = Command::new(program.as_ref()).args(args).output()?;
    if !output.status.success() {
        return Err(exec_err!(program, args, "execution failed"));
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Similar to [`standard_output`], but get first line of the output as string instead
///
/// # Errors
///
/// This will return errors if:
/// 1. The specific command cannot be execute.
/// 2. The command was executed but failed.
/// 3. The standard output contains non-UTF8 characteres thus cannot be parsed from bytes.
/// 4. The output string was empty.
pub(crate) fn standard_output_first_line_only<P, A>(program: P, args: &[A]) -> Result<String>
where
    P: AsRef<OsStr>,
    A: AsRef<OsStr>,
{
    let output = standard_output(program.as_ref(), args)?;
    output
        .lines()
        .next()
        .map(ToOwned::to_owned)
        .ok_or_else(|| exec_err!(program, args, ": empty output"))
}

/// Wrapper to [`std::fs::read_to_string`] but with additional error context.
pub(crate) fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read '{}'", path.as_ref().display()))
}

pub(crate) fn parse_url(url: &str) -> Result<Url> {
    Url::parse(url).with_context(|| format!("failed to parse url: {url}"))
}

pub(crate) fn execute_for_output_with_env<P, S, I>(program: P, args: &[S], env: I) -> Result<Output>
where
    P: AsRef<OsStr>,
    S: AsRef<OsStr>,
    I: IntoIterator<Item = (S, S)>,
{
    Command::new(program.as_ref())
        .args(args)
        .envs(env)
        .output()
        .with_context(|| exec_err!(program, args, ""))
}

pub(crate) fn stringify_path<P: AsRef<Path>>(path: P) -> Result<String> {
    path.as_ref()
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            anyhow!(
                "failed to stringify path '{}'",
                path.as_ref().to_string_lossy().to_string()
            )
        })
}

pub(crate) fn mkdirs<P: AsRef<Path>>(path: P) -> Result<()> {
    fs::create_dir_all(path.as_ref()).with_context(|| {
        format!(
            "unable to create specified directory '{}'",
            path.as_ref().display()
        )
    })
}

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
    pub fn download_file(&self, url: &Url, path: &Path, resume: bool) -> Result<()> {
        let mut resp = self.client.get(url.as_ref()).send()?;
        let total_size = resp
            .content_length()
            .ok_or_else(|| anyhow!("unable to get file length of '{}'", url.as_str()))?;

        let maybe_indicator = self
            .handler
            .as_ref()
            .and_then(|h| (h.start)(total_size, &self.name).ok());

        let (mut downloaded_len, mut file) = if resume {
            let file = OpenOptions::new().create(true).write(true).open(path)?;
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

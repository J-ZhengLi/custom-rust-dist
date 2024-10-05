use anyhow::{Context, Result};
use reqwest::blocking::Client;
use url::Url;

pub struct OBSOpt {
    // Endpoint
    pub endpoint: Url,
    // Canonicalized Resource
    pub resource_path: String,
}

impl OBSOpt {
    pub fn new(endpoint: Url, resource_path: String) -> Result<Self> {
        Ok(Self {
            endpoint,
            resource_path,
        })
    }

    /// Read an OBS file with a specified path from a public barrel.
    /// Only supporting public barrels.
    pub(crate) fn read(&self) -> Result<String> {
        let client = Client::new();

        let url = self.endpoint.join(&self.resource_path)?;

        let resp = client.get(url).send().with_context(|| {
            format!(
                "failed to read resource with path: \n'{}'",
                self.resource_path
            )
        })?;

        if resp.status().is_success() {
            // 读取响应体并转换为字符串
            let content = resp
                .text()
                .with_context(|| "Failed to read response text")?;
            Ok(content)
        } else {
            Err(anyhow::anyhow!(
                "Failed to download object: {}",
                resp.status()
            ))
        }
    }
}

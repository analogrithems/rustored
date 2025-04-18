use std::path::Path;
use anyhow::{Result, bail};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use reqwest;

/// Restore snapshot file to Elasticsearch using Bulk API or snapshot API
#[allow(unused_variables)]
pub async fn restore_elasticsearch(
    file_path: &Path,
    url: &str,
    username: Option<String>,
    password: Option<String>,
) -> Result<()> {
    // Read bulk file
    let mut file = File::open(file_path).await?;
    let mut body = String::new();
    file.read_to_string(&mut body).await?;
    // Build HTTP client
    let client = reqwest::Client::new();
    // Send bulk request
    let endpoint = format!("{}/_bulk", url.trim_end_matches('/'));
    let mut req = client.post(&endpoint)
        .header("Content-Type", "application/x-ndjson")
        .body(body);
    // Apply basic auth if credentials provided
    if let (Some(u), Some(p)) = (username.clone(), password.clone()) {
        req = req.basic_auth(u, Some(p));
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        let err = resp.text().await?;
        bail!("Elasticsearch bulk restore failed: {}", err);
    }
    Ok(())
}

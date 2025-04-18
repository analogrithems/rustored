use std::path::Path;
use reqwest;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use anyhow::{Result, bail};

#[allow(unused_variables)]
/// Restore snapshot file to Qdrant by indexing data or using snapshot API
pub async fn restore_qdrant(
    file_path: &Path,
    url: &str,
    api_key: Option<String>,
) -> Result<()> {
    // Read NDJSON file content
    let mut file = File::open(file_path).await?;
    let mut body = String::new();
    file.read_to_string(&mut body).await?;
    // Determine collection name from file stem
    let collection = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("default");
    // Build HTTP client with optional API key header
    let client = if let Some(key) = api_key.clone() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("api-key", key.parse()?);
        reqwest::Client::builder().default_headers(headers).build()?
    } else {
        reqwest::Client::new()
    };
    // POST import request to Qdrant
    let endpoint = format!("{}/collections/{}/points/import?wait=true", url.trim_end_matches('/'), collection);
    let resp = client.post(&endpoint)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let err = resp.text().await?;
        bail!("Qdrant import failed: {}", err);
    }
    Ok(())
}

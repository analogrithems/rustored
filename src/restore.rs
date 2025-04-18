use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;
use aws_sdk_s3::Client;
use futures_util::stream::TryStreamExt;
use crate::config::DataStoreConfig;
use anyhow::{Result, Context};

mod postgres;
mod elasticsearch;
mod qdrant;

/// Download a snapshot from S3 to the given destination path
pub async fn download_snapshot(
    s3_client: &Client,
    bucket: &str,
    key: &str,
    dest_path: &std::path::Path,
) -> Result<()> {
    let head = s3_client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;
    let total_size = head.content_length as u64;
    let bar = ProgressBar::new(total_size);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap(),
    );

    let mut stream = s3_client.get_object().bucket(bucket).key(key).send().await?.body;
    let mut file = tokio::fs::File::create(dest_path).await?;
    while let Some(bytes) = stream.try_next().await? {
        file.write_all(&bytes).await?;
        bar.inc(bytes.len() as u64);
    }
    bar.finish_with_message("Download complete");
    Ok(())
}

/// Restore the downloaded file to the datastore
pub async fn restore_to_datastore(
    file_path: &Path,
    ds_cfg: &DataStoreConfig,
) -> Result<()> {
    match ds_cfg {
        DataStoreConfig::Postgres { connection_string, .. } => {
            let conn_str = connection_string.as_ref().context("rustored::restore::restore_to_datastore: Missing DS_POSTGRES_CONN")?;
            postgres::restore_postgres(file_path, conn_str).await
        }
        DataStoreConfig::ElasticSearch { url, username, password, .. } => {
            let url_str = url.as_ref().context("rustored::restore::restore_to_datastore: Missing DS_ES_URL")?;
            elasticsearch::restore_elasticsearch(file_path, url_str, username.clone(), password.clone()).await
        }
        DataStoreConfig::Qdrant { url, api_key, .. } => {
            let url_str = url.as_ref().context("rustored::restore::restore_to_datastore: Missing DS_QDRANT_URL")?;
            qdrant::restore_qdrant(file_path, url_str, api_key.clone()).await
        }
    }
}

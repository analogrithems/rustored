use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;
use aws_sdk_s3::Client;
use futures_util::stream::TryStreamExt;
use std::path::Path;
use crate::config::DataStoreConfig;

mod postgres;
mod elasticsearch;
mod qdrant;

/// Download a snapshot from S3 to the given destination path
pub async fn download_snapshot(
    s3_client: &Client,
    bucket: &str,
    key: &str,
    dest_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match ds_cfg {
        DataStoreConfig::Postgres { connection_string, .. } => {
            postgres::restore_postgres(file_path, connection_string).await
        }
        DataStoreConfig::ElasticSearch { url, username, password, .. } => {
            elasticsearch::restore_elasticsearch(file_path, url, username.clone(), password.clone()).await
        }
        DataStoreConfig::Qdrant { url, api_key, .. } => {
            qdrant::restore_qdrant(file_path, url, api_key.clone()).await
        }
    }
}

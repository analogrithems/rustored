use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;
use aws_sdk_s3::Client;

/// Download a snapshot from S3 to the given destination path
pub async fn download_snapshot(
    s3_client: &Client,
    bucket: &str,
    key: &str,
    dest_path: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
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

    let mut stream = s3_client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?
        .body;

    let mut file = tokio::fs::File::create(dest_path).await?;
    while let Some(bytes) = stream.recv().await? {
        file.write_all(&bytes).await?;
        bar.inc(bytes.len() as u64);
    }
    bar.finish_with_message("Download complete");
    Ok(())
}

/// Restore the downloaded file to the datastore
pub async fn restore_to_datastore(
    file_path: &std::path::Path,
    ds_cfg: &crate::config::DataStoreConfig,
) -> Result<(), Box<dyn Error>> {
    // TODO: implement restore logic per datastore
    println!("Restoring {:?} with config {:?}", file_path, ds_cfg);
    Ok(())
}

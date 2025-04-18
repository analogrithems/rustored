use std::path::Path;
use tokio::process::Command;
use anyhow::{Result, bail};

/// Restore snapshot file to Postgres using psql
pub async fn restore_postgres(file_path: &Path, connection_string: &str) -> Result<()> {
    let output = Command::new("psql")
        .arg(connection_string)
        .arg("-f")
        .arg(file_path)
        .output()
        .await?;
    if !output.status.success() {
        bail!("Postgres restore failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

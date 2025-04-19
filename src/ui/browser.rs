use anyhow::{Result, anyhow};
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::Client as S3Client;

use log::{debug, warn};
use tokio::io::AsyncReadExt;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use crate::ui::models::{S3Config, PopupState, FocusField, BackupMetadata};
use chrono::Utc;

/// Component for S3 snapshot browsing
pub struct SnapshotBrowser {
    // S3 Configuration
    pub s3_config: S3Config,
    
    // S3 Client
    s3_client: Option<S3Client>,
    
    // UI State
    pub focus: FocusField,
    pub input_mode: crate::ui::models::InputMode,
    pub input_buffer: String,
    pub snapshots: Vec<BackupMetadata>,
    pub selected_index: usize,
    pub popup_state: PopupState,
}

impl SnapshotBrowser {
    /// Create a new SnapshotBrowser with default settings
    pub fn new(s3_config: S3Config) -> Self {
        Self {
            s3_config,
            s3_client: None,
            focus: FocusField::SnapshotList,
            input_mode: crate::ui::models::InputMode::Normal,
            input_buffer: String::new(),
            snapshots: Vec::new(),
            selected_index: 0,
            popup_state: PopupState::Hidden,
        }
    }

    /// Initialize the S3 client based on current settings
    pub async fn init_client(&mut self) -> Result<()> {
        self.s3_client = Some(self.s3_config.create_client()?);
        Ok(())
    }

    /// Load snapshots from S3
    pub async fn load_snapshots(&mut self) -> Result<()> {
        debug!("Loading snapshots from S3");
        
        // Initialize client if needed
        if self.s3_client.is_none() {
            debug!("S3 client not initialized, initializing now");
            self.init_client().await?;
        }
        
        if let Some(client) = &self.s3_client {
            debug!("Using S3 client to list objects in bucket: {}", self.s3_config.bucket);
            
            let list_objects_result = client
                .list_objects_v2()
                .bucket(&self.s3_config.bucket)
                .prefix(&self.s3_config.prefix)
                .send()
                .await;
                
            match list_objects_result {
                Ok(output) => {
                    debug!("Successfully listed objects, parsing results");
                    self.parse_s3_output(output);
                    Ok(())
                }
                Err(e) => {
                    debug!("Failed to list objects: {}", e);
                    Err(anyhow!("Failed to list objects: {}", e))
                }
            }
        } else {
            debug!("S3 client not available");
            Err(anyhow!("S3 client not initialized"))
        }
    }

    /// Parse S3 output and populate snapshots
    fn parse_s3_output(&mut self, output: ListObjectsV2Output) {
        self.snapshots.clear();
        self.selected_index = 0;

        if let Some(contents) = output.contents {
            for obj in contents {
                // Skip directory-like objects (ones that end with /)
                if let Some(key) = &obj.key {
                    if key.ends_with('/') {
                        continue;
                    }
                    
                    // Only include objects that match our prefix and have content
                    if key.starts_with(&self.s3_config.prefix) {
                        let metadata = BackupMetadata {
                            key: key.clone(),
                            size: obj.size.unwrap_or(0),
                            last_modified: obj.last_modified
                                .map(|dt| dt.as_secs_f64())
                                .unwrap_or_else(|| Utc::now().timestamp() as f64),
                        };
                        self.snapshots.push(metadata);
                    }
                }
            }
        }
        
        // Sort by most recent first
        self.snapshots.sort_by(|a, b| b.last_modified.partial_cmp(&a.last_modified).unwrap_or(std::cmp::Ordering::Equal));
        
        debug!("Loaded {} snapshots", self.snapshots.len());
    }

    /// Download snapshot to a local file
    pub async fn download_snapshot(
        &mut self,
        snapshot: &BackupMetadata,
        tmp_path: &Path,
    ) -> Result<Option<String>> {
        if let Some(client) = &self.s3_client {
            self.popup_state = PopupState::Downloading(snapshot.clone(), 0.0, 0.0);
            
            // Set popup state for download
            self.popup_state = PopupState::Downloading(snapshot.clone(), 0.0, 0.0);
            
            match client
                .get_object()
                .bucket(&self.s3_config.bucket)
                .key(&snapshot.key)
                .send()
                .await
            {
                Ok(output) => {
                    debug!("Download started for {}", snapshot.key);
                    
                    // Ensure parent directory exists
                    if let Some(parent) = tmp_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    
                    // Save to file
                    if let Ok(file) = File::create(tmp_path) {
                        let tmp_path_str = tmp_path.to_string_lossy().to_string();
                        if let Err(e) = self.save_stream_to_file(output, snapshot, file).await {
                            warn!("Error saving file: {}", e);
                            self.popup_state = PopupState::Error(format!("Download failed: {}", e));
                            // Set error popup state
                            return Ok(None);
                        }
                        return Ok(Some(tmp_path_str));
                    } else {
                        debug!("Could not create file at {:?}", tmp_path);
                        return Ok(None);
                    }
                }
                Err(e) => {
                    debug!("Failed to download snapshot {}: {}", snapshot.key, e);
                    return Ok(None);
                }
            }
        } else {
            debug!("Download attempted but S3 client not initialized");
            return Ok(None);
        }
    }

    /// Save stream to file with progress updates
    async fn save_stream_to_file(
        &mut self,
        output: GetObjectOutput,
        snapshot: &BackupMetadata,
        mut file: File,
    ) -> Result<()> {
        let mut body = output.body.into_async_read();
        let size = snapshot.size as f64;
        let mut downloaded: usize = 0;
        let mut buffer = [0; 1024 * 64]; // 64KB buffer
        let start_time = std::time::Instant::now();
        let mut last_update = std::time::Instant::now();
        
        loop {
            match body.read(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Write to file
                    file.write_all(&buffer[0..n])?;
                    downloaded += n;
                    
                    // Update progress at most 10 times per second
                    let now = std::time::Instant::now();
                    if now.duration_since(last_update).as_millis() > 100 {
                        last_update = now;
                        let elapsed = now.duration_since(start_time).as_secs_f64();
                        let rate = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };
                        let progress = downloaded as f64 / size;
                        
                        // Update popup state
                        self.popup_state = PopupState::Downloading(snapshot.clone(), progress as f32, rate);
                        
                        // Check for user cancel
                        if let PopupState::ConfirmCancel(_, _, _) = self.popup_state {
                            return Err(anyhow!("Download cancelled by user"));
                        }
                    }
                }
                Err(e) => return Err(anyhow!("Error reading from S3: {}", e)),
            }
        }
        
        debug!("Download complete: {}", snapshot.key);
        self.popup_state = PopupState::Success(format!("Download complete: {}", snapshot.key));
        
        Ok(())
    }

    // The restore_snapshot method has been moved to RustoredApp

    // Key handling has been moved to RustoredApp
}

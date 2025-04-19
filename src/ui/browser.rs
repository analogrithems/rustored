use anyhow::{Result, anyhow};
use ratatui::backend::Backend;
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

    /// Handle key events and return a snapshot path if one is downloaded
    pub async fn handle_key_event<B: Backend>(
        &mut self, 
        key: crossterm::event::KeyEvent,
        popup_state: &mut PopupState,
        input_mode: &mut crate::ui::models::InputMode,
        input_buffer: &mut String,
        focus: &mut FocusField,
        s3_config: &mut S3Config
    ) -> Result<Option<String>> {
        use crossterm::event::KeyCode;

        // Handle popup states first
        match popup_state {
            PopupState::ConfirmRestore(snapshot) => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Download the snapshot
                        let tmp_path = std::env::temp_dir().join(format!("rustored_snapshot_{}", snapshot.key.replace("/", "_")));
                        // Note: We can't pass a terminal here since we don't have one in this context
                        // This is fine because the download_snapshot method will handle UI updates through the popup_state
                        return self.download_snapshot(snapshot, &tmp_path).await;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        *popup_state = PopupState::Hidden;
                    }
                    _ => {}
                }
                return Ok(None);
            }
            PopupState::ConfirmCancel(_, _, _) => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        *popup_state = PopupState::Error("Download cancelled".to_string());
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Resume downloading
                        if let PopupState::ConfirmCancel(snapshot, progress, rate) = &*popup_state {
                            *popup_state = PopupState::Downloading(snapshot.clone(), *progress, *rate);
                        }
                    }
                    _ => {}
                }
                return Ok(None);
            }
            PopupState::Downloading(_, _, _) => {
                if key.code == KeyCode::Esc {
                    // Ask for confirmation
                    if let PopupState::Downloading(snapshot, progress, rate) = &*popup_state {
                        *popup_state = PopupState::ConfirmCancel(snapshot.clone(), *progress, *rate);
                    }
                }
                return Ok(None);
            }
            PopupState::Error(_) | PopupState::Success(_) => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                    *popup_state = PopupState::Hidden;
                }
                return Ok(None);
            }
            _ => {}
        }

        // Handle input mode
        if *input_mode == crate::ui::models::InputMode::Editing {
            match key.code {
                KeyCode::Enter => {
                    // Apply the edited value
                    match focus {
                        FocusField::Bucket => s3_config.bucket = input_buffer.clone(),
                        FocusField::Region => s3_config.region = input_buffer.clone(),
                        FocusField::Prefix => s3_config.prefix = input_buffer.clone(),
                        FocusField::EndpointUrl => s3_config.endpoint_url = input_buffer.clone(),
                        FocusField::AccessKeyId => s3_config.access_key_id = input_buffer.clone(),
                        FocusField::SecretAccessKey => s3_config.secret_access_key = input_buffer.clone(),
                        FocusField::PathStyle => {
                            s3_config.path_style = input_buffer.to_lowercase() == "true";
                        }
                        _ => {}
                    }
                    *input_mode = crate::ui::models::InputMode::Normal;
                    
                    // Update S3 client with new settings
                    self.s3_config = s3_config.clone();
                    let _ = self.init_client().await;
                    
                    // Reload snapshots with new settings
                    if let Err(e) = self.load_snapshots().await {
                        debug!("Failed to load snapshots: {}", e);
                    }
                }
                KeyCode::Esc => {
                    // Cancel editing
                    *input_mode = crate::ui::models::InputMode::Normal;
                }
                KeyCode::Char(c) => {
                    // Add character
                    input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    // Remove character
                    input_buffer.pop();
                }
                _ => {}
            }
            return Ok(None);
        }

        // Normal mode
        match key.code {
            KeyCode::Char('q') => {
                // Quit
                return Ok(None);
            }
            KeyCode::Char('r') => {
                // Reload snapshots
                if let Err(e) = self.load_snapshots().await {
                    debug!("Failed to load snapshots: {}", e);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Move down in the list
                if !self.snapshots.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.snapshots.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Move up in the list
                if !self.snapshots.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.snapshots.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            KeyCode::Enter => {
                // Select snapshot
                if !self.snapshots.is_empty() {
                    let snapshot = &self.snapshots[self.selected_index];
                    *popup_state = PopupState::ConfirmRestore(snapshot.clone());
                }
            }
            KeyCode::Tab => {
                // Cycle focus
                *focus = match focus {
                    FocusField::SnapshotList => FocusField::Bucket,
                    FocusField::Bucket => FocusField::Region,
                    FocusField::Region => FocusField::Prefix,
                    FocusField::Prefix => FocusField::EndpointUrl,
                    FocusField::EndpointUrl => FocusField::AccessKeyId,
                    FocusField::AccessKeyId => FocusField::SecretAccessKey,
                    FocusField::SecretAccessKey => FocusField::PathStyle,
                    FocusField::PathStyle => FocusField::SnapshotList,
                    _ => FocusField::SnapshotList,
                };
            }
            KeyCode::Char('e') => {
                // Edit field
                if *focus != FocusField::SnapshotList {
                    *input_mode = crate::ui::models::InputMode::Editing;
                    // Set input buffer to current value
                    *input_buffer = match focus {
                        FocusField::Bucket => s3_config.bucket.clone(),
                        FocusField::Region => s3_config.region.clone(),
                        FocusField::Prefix => s3_config.prefix.clone(),
                        FocusField::EndpointUrl => s3_config.endpoint_url.clone(),
                        FocusField::AccessKeyId => s3_config.access_key_id.clone(),
                        FocusField::SecretAccessKey => s3_config.secret_access_key.clone(),
                        FocusField::PathStyle => s3_config.path_style.to_string(),
                        _ => String::new(),
                    };
                }
            }
            _ => {}
        }

        Ok(None)
    }
}

use anyhow::{anyhow, Result};
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use crossterm::execute;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use libc::{raise, SIGTSTP};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use random_word::{Lang, get as random_word};
use std::io::stdout;
use std::path::Path;
use log::{debug, error, info};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

use crate::ui::models::{S3Config, PostgresConfig, BackupMetadata, PopupState, InputMode, FocusField};
use crate::datastore::{ElasticsearchConfig, QdrantConfig, RestoreTarget};

/// Component for snapshot listing and operations
pub struct SnapshotBrowser {
    pub s3_config: S3Config,
    pub pg_config: PostgresConfig,
    pub s3_client: Option<S3Client>,
    pub snapshots: Vec<BackupMetadata>,
    pub selected_index: usize,
    pub temp_file: Option<String>,
    pub restore_target: RestoreTarget,
    pub es_config: ElasticsearchConfig,
    pub qdrant_config: QdrantConfig,
    pub popup_state: PopupState,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub focus: FocusField,
}

impl std::fmt::Debug for SnapshotBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotBrowser")
            .field("s3_config", &self.s3_config)
            .field("pg_config", &self.pg_config)
            .field("s3_client", &self.s3_client.is_some())
            .field("snapshots", &self.snapshots.len())
            .field("selected_index", &self.selected_index)
            .field("temp_file", &self.temp_file)
            .field("restore_target", &self.restore_target)
            .field("es_config", &self.es_config)
            .field("qdrant_config", &self.qdrant_config)
            .field("popup_state", &self.popup_state)
            .field("input_mode", &self.input_mode)
            .field("input_buffer", &self.input_buffer)
            .field("focus", &self.focus)
            .finish()
    }
}

impl SnapshotBrowser {
    /// Create a new SnapshotBrowser from configs
    pub fn new(s3_config: S3Config, pg_config: PostgresConfig) -> Self {
        Self { 
            s3_config, 
            pg_config, 
            s3_client: None, 
            snapshots: Vec::new(), 
            selected_index: 0,
            temp_file: None,
            restore_target: RestoreTarget::Postgres,
            es_config: ElasticsearchConfig::default(),
            qdrant_config: QdrantConfig::default(),
            popup_state: PopupState::Hidden,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            focus: FocusField::SnapshotList,
        }
    }

    pub async fn load_snapshots(&mut self) -> Result<()> {
        if self.s3_client.is_none() {
            if let Err(e) = self.s3_config.create_client() {
                return Err(e);
            }
        }

        let client = self.s3_client.as_ref().unwrap();

        let mut list_objects_builder = client.list_objects_v2()
            .bucket(&self.s3_config.bucket);

        if !self.s3_config.prefix.is_empty() {
            list_objects_builder = list_objects_builder.prefix(&self.s3_config.prefix);
        }

        match list_objects_builder.send().await {
            Ok(resp) => {
                self.snapshots.clear();

                let contents = resp.contents();
                if !contents.is_empty() {
                    for obj in contents {
                        if let (Some(key), Some(size), Some(last_modified)) = (obj.key(), obj.size(), obj.last_modified()) {
                            self.snapshots.push(BackupMetadata {
                                key: key.to_string(),
                                size: size,
                                last_modified: last_modified.clone(),
                            });
                        }
                    }
                }

                // Sort by last modified, newest first
                self.snapshots.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

                if !self.snapshots.is_empty() {
                    // Ensure selected_index is valid
                    if self.selected_index >= self.snapshots.len() {
                        self.selected_index = self.snapshots.len() - 1;
                    }
                }

                Ok(())
            },
            Err(e) => {
                Err(anyhow!("Failed to list objects: {}", e))
            }
        }
    }

    pub fn next(&mut self) {
        if !self.snapshots.is_empty() {
            if self.selected_index + 1 < self.snapshots.len() {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn prev(&mut self) {
        if !self.snapshots.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.snapshots.len() - 1;
            }
        }
    }

    pub fn selected_snapshot(&self) -> Option<&BackupMetadata> {
        if self.snapshots.is_empty() {
            None
        } else {
            self.snapshots.get(self.selected_index)
        }
    }

    pub async fn download_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, temp_path: &std::path::Path) -> Result<Option<String>> {
        // Clone the necessary data to avoid borrowing issues
        let temp_path_str = temp_path.to_string_lossy().to_string();
        let s3_client = self.s3_client.clone();
        let bucket = self.s3_config.bucket.clone();

        // Start download
        self.temp_file = Some(temp_path_str.clone());

        // Track download rate
        let mut last_update = std::time::Instant::now();
        let mut last_bytes = 0u64;
        let mut current_rate = 0.0;

        // Begin downloading the file
        if let Some(client) = &s3_client {
            let get_obj = client.get_object()
                .bucket(&bucket)
                .key(&snapshot.key)
                .send()
                .await;

            match get_obj {
                Ok(resp) => {
                    if let Some(total_size) = resp.content_length() {
                        let mut file = tokio::fs::File::create(&temp_path).await?;
                        let mut stream = resp.body;
                        let mut downloaded: u64 = 0;

                        while let Some(chunk) = stream.try_next().await? {
                            file.write_all(&chunk).await?;
                            downloaded += chunk.len() as u64;

                            // Calculate download rate
                            let now = std::time::Instant::now();
                            let elapsed = now.duration_since(last_update).as_secs_f64();
                            if elapsed >= 0.5 { // Update rate every 0.5 seconds
                                let bytes_since_last = downloaded - last_bytes;
                                current_rate = bytes_since_last as f64 / elapsed;
                                last_update = now;
                                last_bytes = downloaded;
                            }

                            // Check for user input (like ESC key) during download
                            if crossterm::event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
                                if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap_or(crossterm::event::Event::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Null, crossterm::event::KeyModifiers::NONE))) {
                                    if key.code == crossterm::event::KeyCode::Esc {
                                        debug!("User pressed ESC to cancel download");
                                        return Ok(None);
                                    }
                                }
                            }

                            // Force a redraw to show progress
                            terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                            // Small delay to allow UI updates
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        }
                        self.temp_file = Some(temp_path_str.clone());
                        debug!("Download completed successfully: {}", temp_path_str);
                        return Ok(Some(temp_path_str));
                    } else {
                        debug!("Could not determine file size for snapshot: {}", snapshot.key);
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

    /// Restore a database from a downloaded snapshot file
    pub async fn restore_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, file_path: &str) -> Result<()> {
        // Start restore operation
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
        
        // Update UI to show restore is in progress
        let mut progress = 0.0;
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
        
        let result = match self.restore_target {
            RestoreTarget::Postgres => {
                // Get PostgreSQL connection details
                let host = self.pg_config.host.as_ref().ok_or_else(|| anyhow!("PostgreSQL host not specified"))?.clone();
                let port = self.pg_config.port.ok_or_else(|| anyhow!("PostgreSQL port not specified"))?;
                let username = self.pg_config.username.clone();
                let password = self.pg_config.password.clone();
                let use_ssl = self.pg_config.use_ssl;
                
                // Call the PostgreSQL restore function
                debug!("Restoring to PostgreSQL at {}:{}", host, port);
                crate::postgres::restore_snapshot(
                    &host,
                    port,
                    username,
                    password,
                    use_ssl,
                    file_path,
                ).await.map(|db_name| {
                    info!("Restored to PostgreSQL database: {}", db_name);
                })
            },
            RestoreTarget::Elasticsearch => {
                // Get Elasticsearch connection details
                let host = self.es_config.host.as_ref().ok_or_else(|| anyhow!("Elasticsearch host not specified"))?.clone();
                let index = self.es_config.index.as_ref().ok_or_else(|| anyhow!("Elasticsearch index not specified"))?.clone();
                
                // Call the Elasticsearch restore function
                debug!("Restoring to Elasticsearch at {}, index {}", host, index);
                crate::datastore::restore_to_elasticsearch(&host, &index, file_path).await
            },
            RestoreTarget::Qdrant => {
                // Get Qdrant connection details
                let host = self.qdrant_config.host.as_ref().ok_or_else(|| anyhow!("Qdrant host not specified"))?.clone();
                let collection = self.qdrant_config.collection.as_ref().ok_or_else(|| anyhow!("Qdrant collection not specified"))?.clone();
                let api_key = self.qdrant_config.api_key.clone();
                
                // Call the Qdrant restore function
                debug!("Restoring to Qdrant at {}, collection {}", host, collection);
                crate::datastore::restore_to_qdrant(&host, &collection, api_key.as_deref(), file_path).await
            },
        };
        
        // Update progress and UI
        progress = 1.0;
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
        
        // Handle the result
        match result {
            Ok(_) => {
                debug!("Restore completed successfully");
                // Show final status message
                terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                Ok(())
            },
            Err(e) => {
                debug!("Restore failed: {}", e);
                Err(anyhow!("Restore failed: {}", e))
            }
        }
    }
}

/// Run the TUI application, delegating to RustoredApp
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut crate::ui::rustored::RustoredApp) -> Result<Option<String>> {
    // Initial load of snapshots
    if let Err(e) = app.snapshot_browser.load_snapshots().await {
        debug!("Failed to load snapshots: {}", e);
    }

    loop {
        // Draw UI
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, &mut app.snapshot_browser))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle suspend on Ctrl+Z when in Normal mode
                if app.input_mode == InputMode::Normal && key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('z') {
                    // Exit TUI mode
                    disable_raw_mode()?;
                    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                    // Suspend process
                    unsafe { raise(SIGTSTP); }
                    // On resume, re-enter TUI
                    enable_raw_mode()?;
                    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                    continue;
                }

                // Handle ConfirmRestore popup (cancel/confirm)
                if let PopupState::ConfirmRestore(snapshot_meta) = &app.popup_state {
                    match key.code {
                        KeyCode::Char('n') => { app.popup_state = PopupState::Hidden; }
                        KeyCode::Char('y') => {
                            let snapshot = snapshot_meta.clone();
                            app.popup_state = PopupState::Hidden;
                            // Download snapshot to temp dir
                            let tmp_path = std::env::temp_dir().join(&snapshot.key);
                            if let Some(path_str) = app.snapshot_browser.download_snapshot(&snapshot, terminal, &tmp_path).await? {
                                // Restore downloaded snapshot
                                app.snapshot_browser.restore_snapshot(&snapshot, terminal, &path_str).await?;
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                match app.input_mode {
                    InputMode::Editing => {
                        // Capture edit input
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                debug!("Exited editing mode");
                            }
                            KeyCode::Enter => {
                                // Commit edits based on focus
                                match app.focus {
                                    // S3 Config fields
                                    FocusField::Bucket => app.s3_config.bucket = app.input_buffer.clone(),
                                    FocusField::Region => app.s3_config.region = app.input_buffer.clone(),
                                    FocusField::Prefix => app.s3_config.prefix = app.input_buffer.clone(),
                                    FocusField::EndpointUrl => app.s3_config.endpoint_url = app.input_buffer.clone(),
                                    FocusField::AccessKeyId => app.s3_config.access_key_id = app.input_buffer.clone(),
                                    FocusField::SecretAccessKey => app.s3_config.secret_access_key = app.input_buffer.clone(),
                                    FocusField::PathStyle => app.s3_config.path_style = app.input_buffer.parse().unwrap_or(false),
                                    
                                    // PostgreSQL Config fields
                                    FocusField::PgHost => app.pg_config.host = Some(app.input_buffer.clone()),
                                    FocusField::PgPort => app.pg_config.port = app.input_buffer.parse().ok(),
                                    FocusField::PgUsername => app.pg_config.username = Some(app.input_buffer.clone()),
                                    FocusField::PgPassword => app.pg_config.password = Some(app.input_buffer.clone()),
                                    FocusField::PgSsl => app.pg_config.use_ssl = app.input_buffer.parse().unwrap_or(false),
                                    FocusField::PgDbName => app.pg_config.db_name = Some(app.input_buffer.clone()),
                                    
                                    // Elasticsearch Config fields
                                    FocusField::EsHost => app.es_config.host = Some(app.input_buffer.clone()),
                                    FocusField::EsIndex => app.es_config.index = Some(app.input_buffer.clone()),
                                    
                                    // Qdrant Config fields (reusing ES fields for Qdrant)
                                    // FocusField::EsHost is used for Qdrant host in the QdrantConfig
                                    // FocusField::EsIndex is used for Qdrant collection in the QdrantConfig
                                    FocusField::QdrantApiKey => app.qdrant_config.api_key = Some(app.input_buffer.clone()),
                                    
                                    // Other fields don't need to be handled here
                                    _ => {}
                                }
                                app.input_buffer.clear();
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => { app.input_buffer.push(c); }
                            KeyCode::Backspace => { app.input_buffer.pop(); }
                            _ => {}
                        }
                    },
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => {
                                debug!("User pressed 'q' to quit");
                                return Ok(None);
                            },
                            KeyCode::Tab => {
                                if app.focus == FocusField::RestoreTarget {
                                    // Enter settings for selected restore target using the model's method
                                    app.focus = app.restore_target.first_focus_field();
                                    debug!("Switched focus into {:?} settings", app.focus);
                                } else {
                                    // Cycle through main windows: Restore Target, S3 Settings, Snapshots
                                    let windows = [FocusField::RestoreTarget, FocusField::Bucket, FocusField::SnapshotList];
                                    let idx = windows.iter().position(|w| app.focus == *w).unwrap_or(0);
                                    app.focus = windows[(idx + 1) % windows.len()];
                                    debug!("Switched focus to window: {:?}", app.focus);
                                }
                            },
                            KeyCode::BackTab => {
                                // Reverse cycle
                                let windows = [FocusField::RestoreTarget, FocusField::Bucket, FocusField::SnapshotList];
                                let idx = windows.iter().position(|w| app.focus == *w).unwrap_or(0);
                                app.focus = windows[(idx + windows.len() - 1) % windows.len()];
                                debug!("Switched focus to window: {:?}", app.focus);
                            },
                            KeyCode::Left | KeyCode::Right => {
                                // Tab navigation (left/right) for RestoreTarget tabs
                                if app.focus == FocusField::RestoreTarget {
                                    match key.code {
                                        KeyCode::Right => {
                                            app.restore_target = match app.restore_target {
                                                crate::ui::models::RestoreTarget::Postgres => crate::ui::models::RestoreTarget::Elasticsearch,
                                                crate::ui::models::RestoreTarget::Elasticsearch => crate::ui::models::RestoreTarget::Qdrant,
                                                crate::ui::models::RestoreTarget::Qdrant => crate::ui::models::RestoreTarget::Postgres,
                                            };
                                            debug!("Restore target changed to {:?}", app.restore_target);
                                        },
                                        KeyCode::Left => {
                                            app.restore_target = match app.restore_target {
                                                crate::ui::models::RestoreTarget::Postgres => crate::ui::models::RestoreTarget::Qdrant,
                                                crate::ui::models::RestoreTarget::Elasticsearch => crate::ui::models::RestoreTarget::Postgres,
                                                crate::ui::models::RestoreTarget::Qdrant => crate::ui::models::RestoreTarget::Elasticsearch,
                                            };
                                            debug!("Restore target changed to {:?}", app.restore_target);
                                        },
                                        _ => {}
                                    }
                                }
                            },
                            KeyCode::Enter => {
                                // Enter: either start editing or handle snapshot restore
                                match app.focus {
                                    FocusField::SnapshotList => {
                                        if handle_snapshot_list_navigation(&mut app.snapshot_browser, key.code) {
                                            debug!("Snapshot list navigation: focus={:?} selected_index={:?}", app.focus, app.snapshot_browser.selected_index);
                                        }
                                    }
                                    // Start editing S3 config
                                    field if S3Config::contains_field(field) => {
                                        app.input_mode = InputMode::Editing;
                                        app.input_buffer = app.s3_config.get_field_value(app.focus);
                                    }
                                    // Start editing PostgreSQL settings
                                    field if PostgresConfig::contains_field(field) => {
                                        app.input_mode = InputMode::Editing;
                                        app.input_buffer = app.pg_config.get_field_value(app.focus);
                                    }
                                    // Start editing Elasticsearch or Qdrant settings
                                    field if matches!(field, FocusField::EsHost | FocusField::EsIndex | FocusField::QdrantApiKey) => {
                                        app.input_mode = InputMode::Editing;
                                        app.input_buffer = match app.focus {
                                            FocusField::EsHost => app.es_config.host.clone().unwrap_or_default(),
                                            FocusField::EsIndex => app.es_config.index.clone().unwrap_or_default(),
                                            FocusField::QdrantApiKey => app.qdrant_config.api_key.clone().unwrap_or_default(),
                                            _ => String::new(),
                                        };
                                    }
                                    _ => { }
                                }
                            },
                            KeyCode::Char('t') => {
                                // Test connection on 't'
                                if S3Config::contains_field(app.focus) {
                                    if let Err(e) = app.s3_config.test_connection(|s| app.popup_state = s).await {
                                        debug!("S3 connection test failed: {}", e);
                                    }
                                } else if PostgresConfig::contains_field(app.focus) {
                                    if let Err(e) = app.pg_config.test_connection(|s| app.popup_state = s).await {
                                        debug!("PG connection test failed: {}", e);
                                    }
                                }
                            },
                            KeyCode::Up | KeyCode::Down => {
                                // Delegate navigation within the current window
                                if app.focus == FocusField::SnapshotList {
                                    handle_snapshot_list_navigation(&mut app.snapshot_browser, key.code);
                                } else if handle_s3_settings_navigation(app, key.code) {
                                    debug!("S3 settings navigation: focus={:?}", app.focus);
                                } else if handle_restore_target_navigation(app, key.code) {
                                    debug!("Restore target navigation: focus={:?}", app.focus);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        // Check if we need to show a success message briefly
        if let PopupState::Success(_) = &app.popup_state {
            sleep(Duration::from_secs(1)).await;
            app.popup_state = PopupState::Hidden;
        }
    }
}

fn handle_snapshot_list_navigation(browser: &mut SnapshotBrowser, key: KeyCode) -> bool {
    match key {
        KeyCode::Up => { browser.prev(); true },
        KeyCode::Down => { browser.next(); true },
        KeyCode::Enter => {
            if let Some(snapshot) = browser.selected_snapshot() {
                let key = snapshot.key.clone();
                debug!("Selected snapshot for restore: {}", key);
            }
            true
        },
        _ => false
    }
}

fn handle_s3_settings_navigation(app: &mut crate::ui::rustored::RustoredApp, key: KeyCode) -> bool {
    // Get fields for S3 settings using the model's method
    let fields = S3Config::focus_fields();
    
    if let Some(idx) = fields.iter().position(|f| app.focus == *f) {
        match key {
            KeyCode::Up => {
                app.focus = fields[(idx + fields.len() - 1) % fields.len()];
                true
            }
            KeyCode::Down => {
                app.focus = fields[(idx + 1) % fields.len()];
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

fn handle_restore_target_navigation(app: &mut crate::ui::rustored::RustoredApp, key: KeyCode) -> bool {
    // Get fields for current restore target using the model's method
    let fields = app.restore_target.focus_fields();
    
    if app.focus == FocusField::RestoreTarget {
        match key {
            KeyCode::Down => { app.focus = fields[0]; true },
            KeyCode::Up => { app.focus = fields[fields.len() - 1]; true },
            _ => false,
        }
    } else if let Some(idx) = fields.iter().position(|f| app.focus == *f) {
        match key {
            KeyCode::Up => {
                app.focus = fields[(idx + fields.len() - 1) % fields.len()];
                true
            }
            KeyCode::Down => {
                app.focus = fields[(idx + 1) % fields.len()];
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

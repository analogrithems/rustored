use anyhow::{anyhow, Result};
use aws_sdk_s3::{Client as S3Client, config::Credentials};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use libc::{raise, SIGTSTP};
use log::{debug, info};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::time::Duration;
use std::io::stdout;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use random_word::Lang;
use tokio_postgres::Config as PgConfig;

use crate::postgres;
use crate::ui::models::{S3Config, PostgresConfig, ElasticsearchConfig, QdrantConfig, BackupMetadata, PopupState, InputMode, FocusField, RestoreTarget};

/// Snapshot browser for managing S3 backups
pub struct SnapshotBrowser {
    pub config: S3Config,
    pub pg_config: PostgresConfig,
    pub s3_client: Option<S3Client>,
    pub restore_target: crate::ui::models::RestoreTarget,
    pub es_host: Option<String>,
    pub es_index: Option<String>,
    pub qdrant_api_key: Option<String>,
    pub snapshots: Vec<BackupMetadata>,
    pub selected_idx: Option<usize>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub focus: FocusField,
    pub popup_state: PopupState,
    pub temp_file: Option<String>,
}

impl std::fmt::Debug for SnapshotBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotBrowser")
            .field("config", &self.config)
            .field("pg_config", &self.pg_config)
            .field("s3_client", &"<S3Client>")
            .field("snapshots", &self.snapshots)
            .field("selected_idx", &self.selected_idx)
            .field("input_mode", &self.input_mode)
            .field("input_buffer", &self.input_buffer)
            .field("focus", &self.focus)
            .field("popup_state", &self.popup_state)
            .field("temp_file", &self.temp_file)
            .finish()
    }
}

impl SnapshotBrowser {
    pub async fn test_s3_connection(&mut self) -> Result<()> {
        if self.s3_client.is_none() {
            if let Err(e) = self.init_s3_client().await {
                self.popup_state = PopupState::Error(format!("Failed to initialize S3 client: {}", e));
                return Err(e);
            }
        }

        match self.s3_client.as_ref().unwrap().list_buckets().send().await {
            Ok(resp) => {
                let buckets = resp.buckets();
                let bucket_names: Vec<String> = buckets
                    .iter()
                    .filter_map(|b| b.name().map(|s| s.to_string()))
                    .collect();

                let result = format!("Successfully connected to S3!\nAvailable buckets: {}",
                    if bucket_names.is_empty() { "None".to_string() } else { bucket_names.join(", ") });
                self.popup_state = PopupState::TestS3Result(result);
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to connect to S3: {}", e);
                self.popup_state = PopupState::Error(error_msg.clone());
                Err(anyhow!(error_msg))
            }
        }
    }

    pub async fn test_pg_connection(&mut self) -> Result<Option<tokio_postgres::Client>> {
        // Validate PostgreSQL settings
        if self.pg_config.host.is_none() || self.pg_config.host.as_ref().unwrap().is_empty() {
            self.popup_state = PopupState::Error("PostgreSQL host is required".to_string());
            return Err(anyhow!("PostgreSQL host is required"));
        }

        if self.pg_config.port.is_none() {
            self.popup_state = PopupState::Error("PostgreSQL port is required".to_string());
            return Err(anyhow!("PostgreSQL port is required"));
        }

        if self.pg_config.username.is_none() || self.pg_config.username.as_ref().unwrap().is_empty() {
            self.popup_state = PopupState::Error("PostgreSQL username is required".to_string());
            return Err(anyhow!("PostgreSQL username is required"));
        }

        // Construct connection string
        let mut config = PgConfig::new();
        config.host(self.pg_config.host.as_ref().unwrap());
        config.port(self.pg_config.port.unwrap());
        config.user(self.pg_config.username.as_ref().unwrap());
        config.password(&self.pg_config.password.as_ref().unwrap_or(&String::new()));
        let result = if self.pg_config.use_ssl {
            postgres::connect_ssl(&config, false, None).await
        } else {
            postgres::connect_no_ssl(&config).await
        };
        match result {
            Ok(client) => {
                info!("Successfully connected to PostgreSQL");
                self.popup_state = PopupState::TestPgResult(format!("Successfully connected to PostgreSQL\nConnection string: {:?}", config));
                Ok(Some(client))
            },
            Err(e) => {
                let error_msg = format!("Failed to connect to PostgreSQL: {}", e);
                self.popup_state = PopupState::Error(error_msg.clone());
                Err(anyhow!(error_msg))
            }
        }
    }



pub fn new(config: S3Config, pg_config: PostgresConfig) -> Self {
        Self {
            config,
            pg_config,
            s3_client: None,
            snapshots: Vec::new(),
            selected_idx: None,
            input_mode: InputMode::Normal,
        restore_target: RestoreTarget::Postgres,
        es_host: None,
        es_index: None,
        qdrant_api_key: None,
            input_buffer: String::new(),
            focus: FocusField::SnapshotList,
            popup_state: PopupState::Hidden,
            temp_file: None,
        }
    }

    pub fn verify_s3_settings(&self) -> Result<()> {
        if self.config.bucket.is_empty() {
            return Err(anyhow!("Bucket name is required"));
        }

        if self.config.region.is_empty() {
            return Err(anyhow!("Region is required"));
        }

        if self.config.endpoint_url.is_empty() {
            return Err(anyhow!("Endpoint URL is required"));
        }

        if self.config.access_key_id.is_empty() {
            return Err(anyhow!("Access Key ID is required"));
        }

        if self.config.secret_access_key.is_empty() {
            return Err(anyhow!("Secret Access Key is required"));
        }

        Ok(())
    }

    pub fn set_error(&mut self, message: Option<String>) {
        self.config.error_message = message;
    }

    pub async fn init_s3_client(&mut self) -> Result<()> {
        if let Err(e) = self.verify_s3_settings() {
            self.set_error(Some(e.to_string()));
            return Err(e);
        }

        // Clear any previous error
        self.set_error(None);

        let credentials = Credentials::new(
            &self.config.access_key_id,
            &self.config.secret_access_key,
            None, None, "postgres-manager"
        );

        let mut config_builder = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .region(aws_sdk_s3::config::Region::new(self.config.region.clone()));

        if !self.config.endpoint_url.is_empty() {
            let endpoint_url = if !self.config.endpoint_url.starts_with("http") {
                format!("http://{}", self.config.endpoint_url)
            } else {
                self.config.endpoint_url.clone()
            };

            config_builder = config_builder.endpoint_url(endpoint_url);
        }

        if self.config.path_style {
            config_builder = config_builder.force_path_style(true);
        }

        // Add behavior version which is required by AWS SDK
        config_builder = config_builder.behavior_version(aws_sdk_s3::config::BehaviorVersion::latest());

        let config = config_builder.build();
        self.s3_client = Some(S3Client::from_conf(config));

        Ok(())
    }

    pub async fn load_snapshots(&mut self) -> Result<()> {
        if self.s3_client.is_none() {
            if let Err(e) = self.init_s3_client().await {
                return Err(e);
            }
        }

        let client = self.s3_client.as_ref().unwrap();

        let mut list_objects_builder = client.list_objects_v2()
            .bucket(&self.config.bucket);

        if !self.config.prefix.is_empty() {
            list_objects_builder = list_objects_builder.prefix(&self.config.prefix);
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

                if !self.snapshots.is_empty() && self.selected_idx.is_none() {
                    self.selected_idx = Some(0);
                } else if self.snapshots.is_empty() {
                    self.selected_idx = None;
                } else if let Some(idx) = self.selected_idx {
                    if idx >= self.snapshots.len() {
                        self.selected_idx = Some(self.snapshots.len() - 1);
                    }
                }

                Ok(())
            },
            Err(e) => {
                self.set_error(Some(format!("Failed to list objects: {}", e)));
                Err(anyhow!("Failed to list objects: {}", e))
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(idx) = self.selected_idx {
            if idx + 1 < self.snapshots.len() {
                self.selected_idx = Some(idx + 1);
            }
        } else if !self.snapshots.is_empty() {
            self.selected_idx = Some(0);
        }
    }

    pub fn previous(&mut self) {
        if let Some(idx) = self.selected_idx {
            if idx > 0 {
                self.selected_idx = Some(idx - 1);
            }
        } else if !self.snapshots.is_empty() {
            self.selected_idx = Some(self.snapshots.len() - 1);
        }
    }

    pub fn selected_snapshot(&self) -> Option<&BackupMetadata> {
        self.selected_idx.and_then(|idx| self.snapshots.get(idx))
    }

    pub async fn download_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, temp_path: &std::path::Path) -> Result<Option<String>> {
        // Clone the necessary data to avoid borrowing issues
        let temp_path_str = temp_path.to_string_lossy().to_string();
        let s3_client = self.s3_client.clone();
        let bucket = self.config.bucket.clone();

        // Start download
        self.popup_state = PopupState::Downloading(snapshot.clone(), 0.0, 0.0);
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
                            let progress = downloaded as f32 / total_size as f32;

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
                                        log::debug!("User pressed ESC to cancel download");
                                        self.popup_state = PopupState::ConfirmCancel(snapshot.clone(), progress, current_rate);
                                        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                                        continue;
                                    }
                                }
                            }

                            match &self.popup_state {
                                PopupState::ConfirmCancel(..) => {
                                    // Wait for user confirmation
                                    terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                                    continue;
                                },
                                PopupState::Hidden => {
                                    // Download was cancelled and confirmed
                                    log::debug!("Download cancelled by user");
                                    file.flush().await?;
                                    self.temp_file = None; // Reset temp file
                                    return Ok(None);
                                },
                                _ => {
                                    // Continue downloading
                                    self.popup_state = PopupState::Downloading(snapshot.clone(), progress, current_rate);
                                }
                            }
                            // Force a redraw to show progress
                            terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                            // Small delay to allow UI updates
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        }
                        self.temp_file = Some(temp_path_str.clone());
                        log::info!("Download completed successfully: {}", temp_path_str);
                        self.popup_state = PopupState::Success("Download complete".to_string());
                        // Show success message briefly
                        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        return Ok(Some(temp_path_str));
                    } else {
                        log::warn!("Could not determine file size for snapshot: {}", snapshot.key);
                        self.popup_state = PopupState::Error("Could not determine file size".to_string());
                        return Ok(None);
                    }
                }
                Err(e) => {
                    log::error!("Failed to download snapshot {}: {}", snapshot.key, e);
                    self.popup_state = PopupState::Error(format!("Failed to download backup: {}", e));
                    return Ok(None);
                }
            }
        } else {
            log::warn!("Download attempted but S3 client not initialized");
            self.popup_state = PopupState::Error("S3 client not initialized".to_string());
            return Ok(None);
        }
    }

    /// Restore a database from a downloaded snapshot file
    pub async fn restore_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, file_path: &str) -> Result<()> {
        // Validate PostgreSQL settings
        if self.pg_config.host.is_none() || self.pg_config.host.as_ref().unwrap().is_empty() {
            self.popup_state = PopupState::Error("PostgreSQL host is required".to_string());
            return Err(anyhow!("PostgreSQL host is required"));
        }

        if self.pg_config.port.is_none() {
            self.popup_state = PopupState::Error("PostgreSQL port is required".to_string());
            return Err(anyhow!("PostgreSQL port is required"));
        }

        if self.pg_config.username.is_none() || self.pg_config.username.as_ref().unwrap().is_empty() {
            self.popup_state = PopupState::Error("PostgreSQL username is required".to_string());
            return Err(anyhow!("PostgreSQL username is required"));
        }

        if self.pg_config.db_name.is_none() || self.pg_config.db_name.as_ref().unwrap().is_empty() {
            self.popup_state = PopupState::Error("PostgreSQL database name is required".to_string());
            return Err(anyhow!("PostgreSQL database name is required"));
        }

        // Start restore operation
        self.popup_state = PopupState::Restoring(snapshot.clone(), 0.0);
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;

        // Use a separate thread for the restore operation to avoid blocking the UI
        let host = self.pg_config.host.as_ref().unwrap().clone();
        let port = self.pg_config.port.unwrap();
        let username = self.pg_config.username.clone();
        let password = self.pg_config.password.clone();

        let use_ssl = self.pg_config.use_ssl;
        let file_path_owned = file_path.to_string();

        let pgclient = self.test_pg_connection().await?;

        let new_dbname = format!("{}-restored", random_word::get(Lang::En));
        let create_restore_db = crate::postgres::create_database(&pgclient.unwrap(), &new_dbname).await;

        match create_restore_db {
            Ok(_) => {
                log::info!("Successfully created restore database: {}", new_dbname);
            },
            Err(e) => {
                let error_msg = format!("Failed to create restore database: {}", e);
                self.popup_state = PopupState::Error(error_msg.clone());
                return Err(anyhow!(error_msg));
            }
        }
        // Spawn a blocking task to handle the restore operation
        let restore_handle = tokio::task::spawn_blocking(move || {
            // Call the restore_database function from the backup module
            let result = crate::backup::restore_database(
                &new_dbname,
                &file_path_owned,
                &host,
                port,
                username.as_deref(),
                password.as_deref(),
                use_ssl,
            );
            result
        });

        // Send completion signal (100% progress) in the main async context after restore completes
        // Update the UI with progress while the restore is running
        let mut progress = 0.0;
        // Only update progress bar at the end (no fine-grained progress for pg_restore)
        while progress < 1.0 {
            // Check for user input (like ESC key) during restore
            if crossterm::event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
                if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap_or(crossterm::event::Event::Key(crossterm::event::KeyEvent::new(crossterm::event::KeyCode::Null, crossterm::event::KeyModifiers::NONE))) {
                    if key.code == crossterm::event::KeyCode::Esc {
                        log::debug!("User pressed ESC during restore, but restore cannot be cancelled");
                        // We don't allow cancelling restore operations as they can leave the database in an inconsistent state
                    }
                }
            }
            // Update the UI
            self.popup_state = PopupState::Restoring(snapshot.clone(), progress);
            terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            // For now, break after a short wait and update progress to 1.0 when done
            break;
        }
        // Wait for the restore operation to complete
        match restore_handle.await {
            Ok(inner_result) => {
                progress = 1.0;
                self.popup_state = PopupState::Restoring(snapshot.clone(), progress);
                terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                match inner_result {
                    Ok(_) => {
                        log::info!("pg_restore completed successfully");
                        self.popup_state = PopupState::Success("pg_restore completed successfully".to_string());
                    },
                    Err(e) => {
                        log::error!("pg_restore failed: {}", e);
                        self.popup_state = PopupState::Error(format!("pg_restore failed: {}", e));
                        return Err(anyhow!("pg_restore task failed: {}", e));
                    }
                }
            },
            Err(e) => {
                log::error!("pg_restore task panicked: {}", e);
                self.popup_state = PopupState::Error(format!("pg_restore task failed: {}", e));
                return Err(anyhow!("pg_restore_handler task issues: {}", e));
            }
        }

        // Show final status message
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

/// Run the TUI application
pub async fn run_tui(
    bucket: Option<String>,
    region: Option<String>,
    prefix: Option<String>,
    endpoint_url: Option<String>,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    path_style: bool,
) -> Result<Option<String>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load configuration from environment variables
    let env_s3_config = crate::config::load_s3_config();
    let env_pg_config = crate::config::load_postgres_config();

    // Create app state with CLI args taking precedence over env vars
    let config = S3Config {
        bucket: bucket.as_ref().map_or(env_s3_config.bucket, |b| b.clone()),
        region: region.as_ref().map_or(env_s3_config.region, |r| r.clone()),
        prefix: prefix.as_ref().map_or(env_s3_config.prefix, |p| p.clone()),
        endpoint_url: endpoint_url.as_ref().map_or(env_s3_config.endpoint_url, |e| e.clone()),
        access_key_id: access_key_id.as_ref().map_or(env_s3_config.access_key_id, |a| a.clone()),
        secret_access_key: secret_access_key.as_ref().map_or(env_s3_config.secret_access_key, |s| s.clone()),
        path_style: if bucket.as_ref().is_some() { path_style } else { env_s3_config.path_style },
        error_message: None,
    };

    let pg_config = env_pg_config;
    let browser = SnapshotBrowser::new(config, pg_config);

    // Run app
    let res = run_app(&mut terminal, browser).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    res
}

/// Run the application
#[allow(unreachable_code)]
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut browser: SnapshotBrowser) -> Result<Option<String>> {
    // Initial load of snapshots
    if let Err(e) = browser.load_snapshots().await {
        debug!("Failed to load snapshots: {}", e);
    }

    loop {
        // Draw UI
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, &mut browser))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle suspend on Ctrl+Z when in Normal mode
                if browser.input_mode == InputMode::Normal && key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('z') {
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
                if let PopupState::ConfirmRestore(snapshot_meta) = &browser.popup_state {
                    match key.code {
                        KeyCode::Char('n') => { browser.popup_state = PopupState::Hidden; }
                        KeyCode::Char('y') => {
                            let snapshot = snapshot_meta.clone();
                            browser.popup_state = PopupState::Hidden;
                            // Download snapshot to temp dir
                            let tmp_path = std::env::temp_dir().join(&snapshot.key);
                            if let Some(path_str) = browser.download_snapshot(&snapshot, terminal, &tmp_path).await? {
                                // Restore downloaded snapshot
                                browser.restore_snapshot(&snapshot, terminal, &path_str).await?;
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                match browser.input_mode {
                    InputMode::Editing => {
                        // Capture edit input
                        match key.code {
                            KeyCode::Esc => {
                                browser.input_mode = InputMode::Normal;
                                debug!("Exited editing mode");
                            }
                            KeyCode::Enter => {
                                // Commit edits based on focus using the model's methods
                                if S3Config::contains_field(browser.focus) {
                                    browser.config.set_field_value(browser.focus, browser.input_buffer.clone());
                                } else if PostgresConfig::contains_field(browser.focus) {
                                    browser.pg_config.set_field_value(browser.focus, browser.input_buffer.clone());
                                } else if ElasticsearchConfig::contains_field(browser.focus) {
                                    // For Elasticsearch and Qdrant, we need to handle the shared fields
                                    match browser.focus {
                                        FocusField::EsHost => browser.es_host = Some(browser.input_buffer.clone()),
                                        FocusField::EsIndex => browser.es_index = Some(browser.input_buffer.clone()),
                                        FocusField::QdrantApiKey => browser.qdrant_api_key = Some(browser.input_buffer.clone()),
                                        _ => {}
                                    }
                                }
                                browser.input_buffer.clear();
                                browser.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => { browser.input_buffer.push(c); }
                            KeyCode::Backspace => { browser.input_buffer.pop(); }
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
                                if browser.focus == FocusField::RestoreTarget {
                                    // Enter settings for selected restore target using the model's method
                                    browser.focus = browser.restore_target.first_focus_field();
                                    debug!("Switched focus into {:?} settings", browser.focus);
                                } else {
                                    // Cycle through main windows: Restore Target, S3 Settings, Snapshots
                                    let windows = [FocusField::RestoreTarget, FocusField::Bucket, FocusField::SnapshotList];
                                    let idx = windows.iter().position(|w| browser.focus == *w).unwrap_or(0);
                                    browser.focus = windows[(idx + 1) % windows.len()];
                                    debug!("Switched focus to window: {:?}", browser.focus);
                                }
                            },
                            KeyCode::BackTab => {
                                // Reverse cycle
                                let windows = [FocusField::RestoreTarget, FocusField::Bucket, FocusField::SnapshotList];
                                let idx = windows.iter().position(|w| browser.focus == *w).unwrap_or(0);
                                browser.focus = windows[(idx + windows.len() - 1) % windows.len()];
                                debug!("Switched focus to window: {:?}", browser.focus);
                            },
                            KeyCode::Left | KeyCode::Right => {
                                // Tab navigation (left/right) for RestoreTarget tabs
                                if browser.focus == FocusField::RestoreTarget {
                                    let _prev = browser.restore_target.clone();
                                    browser.restore_target = match (browser.restore_target.clone(), key.code) {
                                        (RestoreTarget::Postgres, KeyCode::Right) => RestoreTarget::Elasticsearch,
                                        (RestoreTarget::Elasticsearch, KeyCode::Right) => RestoreTarget::Qdrant,
                                        (RestoreTarget::Qdrant, KeyCode::Right) => RestoreTarget::Postgres,
                                        (RestoreTarget::Postgres, KeyCode::Left) => RestoreTarget::Qdrant,
                                        (RestoreTarget::Elasticsearch, KeyCode::Left) => RestoreTarget::Postgres,
                                        (RestoreTarget::Qdrant, KeyCode::Left) => RestoreTarget::Elasticsearch,
                                        (other, _) => other,
                                    };
                                    debug!("Switched restore target tab: {:?}", browser.restore_target);
                                }
                            },
                            KeyCode::Enter => {
                                // Enter: either start editing or handle snapshot restore
                                match browser.focus {
                                    FocusField::SnapshotList => {
                                        if handle_snapshot_list_navigation(&mut browser, key.code) {
                                            debug!("Snapshot list navigation: focus={:?} selected_idx={:?}", browser.focus, browser.selected_idx);
                                        }
                                    }
                                    // Start editing S3 config
                                    field if S3Config::contains_field(field) => {
                                        browser.input_mode = InputMode::Editing;
                                        browser.input_buffer = browser.config.get_field_value(browser.focus);
                                    }
                                    // Start editing PostgreSQL settings
                                    field if PostgresConfig::contains_field(field) => {
                                        browser.input_mode = InputMode::Editing;
                                        browser.input_buffer = browser.pg_config.get_field_value(browser.focus);
                                    }
                                    // Start editing Elasticsearch or Qdrant settings
                                    field if ElasticsearchConfig::contains_field(field) || QdrantConfig::contains_field(field) => {
                                        browser.input_mode = InputMode::Editing;
                                        browser.input_buffer = match browser.focus {
                                            FocusField::EsHost => browser.es_host.clone().unwrap_or_default(),
                                            FocusField::EsIndex => browser.es_index.clone().unwrap_or_default(),
                                            FocusField::QdrantApiKey => browser.qdrant_api_key.clone().unwrap_or_default(),
                                            _ => String::new(),
                                        };
                                    }
                                    _ => { }
                                }
                            },
                            KeyCode::Up | KeyCode::Down => {
                                // Delegate navigation within the current window
                                if browser.focus == FocusField::SnapshotList {
                                    handle_snapshot_list_navigation(&mut browser, key.code);
                                } else if handle_s3_settings_navigation(&mut browser, key.code) {
                                    debug!("S3 settings navigation: focus={:?}", browser.focus);
                                } else if handle_restore_target_navigation(&mut browser, key.code) {
                                    debug!("Restore target navigation: focus={:?}", browser.focus);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        // Check if we need to show a success message briefly
        if let PopupState::Success(_) = &browser.popup_state {
            sleep(Duration::from_secs(1)).await;
            browser.popup_state = PopupState::Hidden;
        }
    }
    Ok(None)
}

fn handle_snapshot_list_navigation(browser: &mut SnapshotBrowser, key: KeyCode) -> bool {
    match key {
        KeyCode::Up => { browser.previous(); true },
        KeyCode::Down => { browser.next(); true },
        KeyCode::Enter => {
            if let Some(snapshot) = browser.selected_snapshot() {
                let key = snapshot.key.clone();
                browser.popup_state = PopupState::ConfirmRestore(snapshot.to_owned());
                debug!("Selected snapshot for restore: {}", key);
            }
            true
        },
        _ => false
    }
}

fn handle_s3_settings_navigation(browser: &mut SnapshotBrowser, key: KeyCode) -> bool {
    // Get fields for S3 settings using the model's method
    let fields = S3Config::focus_fields();
    
    if let Some(idx) = fields.iter().position(|f| browser.focus == *f) {
        match key {
            KeyCode::Up => {
                browser.focus = fields[(idx + fields.len() - 1) % fields.len()];
                true
            }
            KeyCode::Down => {
                browser.focus = fields[(idx + 1) % fields.len()];
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

fn handle_restore_target_navigation(browser: &mut SnapshotBrowser, key: KeyCode) -> bool {
    // Get fields for current restore target using the model's method
    let fields = browser.restore_target.focus_fields();
    
    if browser.focus == FocusField::RestoreTarget {
        match key {
            KeyCode::Down => { browser.focus = fields[0]; true },
            KeyCode::Up => { browser.focus = fields[fields.len() - 1]; true },
            _ => false,
        }
    } else if let Some(idx) = fields.iter().position(|f| browser.focus == *f) {
        match key {
            KeyCode::Up => {
                browser.focus = fields[(idx + fields.len() - 1) % fields.len()];
                true
            }
            KeyCode::Down => {
                browser.focus = fields[(idx + 1) % fields.len()];
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

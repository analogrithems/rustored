use anyhow::{anyhow, Result};
use aws_sdk_s3::Client as S3Client;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, DisableMouseCapture, EnableMouseCapture};
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use libc::{raise, SIGTSTP};
use log::debug;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;
use tokio::time::sleep;
use tokio::io::AsyncWriteExt;
// Import removed: use random_word::Lang;

use crate::ui::models::{S3Config, PostgresConfig, ElasticsearchConfig, QdrantConfig, BackupMetadata, PopupState, InputMode, FocusField, RestoreTarget};

/// Snapshot browser for managing S3 backups
pub struct SnapshotBrowser {
    pub config: S3Config,
    pub s3_client: Option<S3Client>,
    pub restore_target: crate::ui::models::RestoreTarget,

    // Config objects for different restore targets
    pub postgres_config: Option<PostgresConfig>,
    pub elasticsearch_config: Option<ElasticsearchConfig>,
    pub qdrant_config: Option<QdrantConfig>,

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
            .field("postgres_config", &self.postgres_config)
            .field("elasticsearch_config", &self.elasticsearch_config)
            .field("qdrant_config", &self.qdrant_config)
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
        // Use the test_connection method from S3Config
        self.config.test_connection(|state| self.popup_state = state).await
    }

    pub async fn test_pg_connection(&mut self) -> Result<Option<tokio_postgres::Client>> {
        // Create PostgresConfig if it doesn't exist
        if self.postgres_config.is_none() {
            self.postgres_config = Some(PostgresConfig::default());
        }

        // Use the test_connection method from PostgresConfig
        if let Some(pg_config) = &self.postgres_config {
            pg_config.test_connection(|state| self.popup_state = state).await
        } else {
            Err(anyhow!("PostgreSQL configuration is not available"))
        }
    }



pub fn new(config: S3Config) -> Self {
        Self {
            config,
            s3_client: None,
            snapshots: Vec::new(),
            selected_idx: None,
            input_mode: InputMode::Normal,
            restore_target: RestoreTarget::Postgres,
            postgres_config: Some(PostgresConfig::default()),
            elasticsearch_config: None,
            qdrant_config: None,
            input_buffer: String::new(),
            focus: FocusField::SnapshotList,
            popup_state: PopupState::Hidden,
            temp_file: None,
        }
    }

    pub async fn verify_s3_settings(&mut self) -> Result<()> {
        // Use the verify_settings method from S3Config
        self.config.verify_settings()
    }

    pub fn set_error(&mut self, message: Option<String>) {
        self.config.error_message = message;
    }

    pub async fn init_s3_client(&mut self) -> Result<()> {
        // Use the create_client method from S3Config
        match self.config.create_client() {
            Ok(client) => {
                // Clear any previous error
                self.set_error(None);
                self.s3_client = Some(client);
                Ok(())
            },
            Err(e) => {
                self.set_error(Some(e.to_string()));
                Err(e)
            }
        }
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

    /// Restore a database from a downloaded snapshot file using the RestoreBrowser
    pub async fn restore_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, file_path: &str) -> Result<()> {
        // Create a RestoreBrowser with the current configuration
        let mut restore_browser = crate::ui::restore_browser::RestoreBrowser::new(
            self.restore_target.clone(),
            self.postgres_config.clone(),
            self.elasticsearch_config.clone(),
            self.qdrant_config.clone(),
        );

        // Delegate the restore operation to the RestoreBrowser
        let result = restore_browser.restore_snapshot(snapshot, terminal, file_path).await;

        // Update our popup state with the result from the RestoreBrowser
        self.popup_state = restore_browser.popup_state.clone();

        // Return the result
        result
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
) -> Result<Option<bool>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load configuration from environment variables
    let env_s3_config = crate::config::load_s3_config();

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

    // Create browser with S3 config only
    let browser = SnapshotBrowser::new(config);

    // Run app
    let res = run_app(&mut terminal, browser).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    res
}

/// Run the application
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut browser: SnapshotBrowser) -> Result<Option<bool>> {
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
                                    // Create PostgresConfig if it doesn't exist
                                    if browser.postgres_config.is_none() {
                                        browser.postgres_config = Some(PostgresConfig::default());
                                    }
                                    if let Some(pg_config) = &mut browser.postgres_config {
                                        pg_config.set_field_value(browser.focus, browser.input_buffer.clone());
                                    }
                                } else if ElasticsearchConfig::contains_field(browser.focus) {
                                    // Create ElasticsearchConfig if it doesn't exist
                                    if browser.elasticsearch_config.is_none() {
                                        browser.elasticsearch_config = Some(ElasticsearchConfig::default());
                                    }
                                    if let Some(es_config) = &mut browser.elasticsearch_config {
                                        es_config.set_field_value(browser.focus, browser.input_buffer.clone());
                                    }
                                } else if QdrantConfig::contains_field(browser.focus) {
                                    // Create QdrantConfig if it doesn't exist
                                    if browser.qdrant_config.is_none() {
                                        browser.qdrant_config = Some(QdrantConfig::default());
                                    }
                                    if let Some(qdrant_config) = &mut browser.qdrant_config {
                                        qdrant_config.set_field_value(browser.focus, browser.input_buffer.clone());
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
                                        if browser.postgres_config.is_none() {
                                            browser.postgres_config = Some(PostgresConfig::default());
                                        }
                                        browser.input_buffer = browser.postgres_config.as_ref()
                                            .map(|pg| pg.get_field_value(browser.focus))
                                            .unwrap_or_default();
                                    }
                                    // Start editing Elasticsearch settings
                                    field if ElasticsearchConfig::contains_field(field) => {
                                        browser.input_mode = InputMode::Editing;
                                        if browser.elasticsearch_config.is_none() {
                                            browser.elasticsearch_config = Some(ElasticsearchConfig::default());
                                        }
                                        browser.input_buffer = browser.elasticsearch_config.as_ref()
                                            .map(|es| es.get_field_value(browser.focus))
                                            .unwrap_or_default();
                                    }
                                    // Start editing Qdrant settings
                                    field if QdrantConfig::contains_field(field) => {
                                        browser.input_mode = InputMode::Editing;
                                        if browser.qdrant_config.is_none() {
                                            browser.qdrant_config = Some(QdrantConfig::default());
                                        }
                                        browser.input_buffer = browser.qdrant_config.as_ref()
                                            .map(|q| q.get_field_value(browser.focus))
                                            .unwrap_or_default();
                                    }
                                    _ => { }
                                }
                            },
                            KeyCode::Char('t') => {
                                // Test connection based on current focus
                                if S3Config::contains_field(browser.focus) {
                                    // Test S3 connection when focus is on any S3 field
                                    debug!("Testing S3 connection...");
                                    if let Err(e) = browser.test_s3_connection().await {
                                        debug!("S3 connection test failed: {}", e);
                                    }
                                } else if PostgresConfig::contains_field(browser.focus) {
                                    // Test PostgreSQL connection when focus is on any PostgreSQL field
                                    debug!("Testing PostgreSQL connection...");
                                    if let Err(e) = browser.test_pg_connection().await {
                                        debug!("PostgreSQL connection test failed: {}", e);
                                    }
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

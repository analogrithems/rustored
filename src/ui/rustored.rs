use crate::ui::models::{S3Config, PostgresConfig, ElasticsearchConfig, QdrantConfig, PopupState, InputMode, FocusField, RestoreTarget, BackupMetadata};
use crate::ui::browser::SnapshotBrowser;
use ratatui::backend::Backend;
use ratatui::Terminal;
use anyhow::{Result, anyhow};
use log::debug;

pub struct RustoredApp {
    pub snapshot_browser: SnapshotBrowser,
    pub s3_config: S3Config,
    pub pg_config: PostgresConfig,
    pub es_config: ElasticsearchConfig,
    pub qdrant_config: QdrantConfig,
    pub restore_target: RestoreTarget,
    pub popup_state: PopupState,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub focus: FocusField,
}

impl RustoredApp {
    /// Create a new RustoredApp from individual CLI arguments
    pub fn new(
        bucket: &Option<String>,
        region: &Option<String>,
        prefix: &Option<String>,
        endpoint_url: &Option<String>,
        access_key_id: &Option<String>,
        secret_access_key: &Option<String>,
        path_style: bool,
        host: &Option<String>,
        port: &Option<u16>,
        username: &Option<String>,
        password: &Option<String>,
        use_ssl: bool,
        db_name: &Option<String>,
        es_host: &Option<String>,
        es_index: &Option<String>,
        qdrant_api_key: &Option<String>,
    ) -> Self {
        let s3_config = S3Config {
            bucket: bucket.clone().unwrap_or_default(),
            region: region.clone().unwrap_or_default(),
            prefix: prefix.clone().unwrap_or_default(),
            endpoint_url: endpoint_url.clone().unwrap_or_default(),
            access_key_id: access_key_id.clone().unwrap_or_default(),
            secret_access_key: secret_access_key.clone().unwrap_or_default(),
            path_style,
            error_message: None,
            test_s3_button: false,
        };
        let pg_config = PostgresConfig {
            host: host.clone(),
            port: *port,
            username: username.clone(),
            password: password.clone(),
            use_ssl,
            db_name: db_name.clone(),
            ..Default::default()
        };
        let es_config = ElasticsearchConfig {
            host: es_host.clone(),
            index: es_index.clone(),
        };
        let qdrant_config = QdrantConfig {
            host: es_host.clone(),
            collection: es_index.clone(),
            api_key: qdrant_api_key.clone(),
        };
        let snapshot_browser = SnapshotBrowser::new(s3_config.clone());
        RustoredApp {
            snapshot_browser,
            s3_config,
            pg_config,
            es_config,
            qdrant_config,
            restore_target: RestoreTarget::Postgres,
            popup_state: PopupState::Hidden,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            focus: FocusField::SnapshotList,
        }
    }

    /// Run the application loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<Option<String>> {
        // Delegate to existing run_app with full app state
        crate::ui::app::run_app(terminal, self).await
    }

    /// Handle key events and return a snapshot path if one is downloaded
    pub async fn handle_key_event<B: Backend>(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<String>> {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Handle popup states first
        if self.popup_state != PopupState::Hidden {
            match &self.popup_state {
                PopupState::ConfirmRestore(snapshot) => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // Download the snapshot
                            let tmp_path = std::env::temp_dir().join(format!("rustored_snapshot_{}", snapshot.key.replace("/", "_")));
                            return self.snapshot_browser.download_snapshot(snapshot, &tmp_path).await;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            self.popup_state = PopupState::Hidden;
                        }
                        _ => {}
                    }
                    return Ok(None);
                }
                PopupState::ConfirmCancel(_, _, _) => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            self.popup_state = PopupState::Error("Download cancelled".to_string());
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // Resume downloading
                            if let PopupState::ConfirmCancel(snapshot, progress, rate) = &self.popup_state {
                                self.popup_state = PopupState::Downloading(snapshot.clone(), *progress, *rate);
                            }
                        }
                        _ => {}
                    }
                    return Ok(None);
                }
                PopupState::Downloading(_, _, _) => {
                    if key.code == KeyCode::Esc {
                        // Ask for confirmation
                        if let PopupState::Downloading(snapshot, progress, rate) = &self.popup_state {
                            self.popup_state = PopupState::ConfirmCancel(snapshot.clone(), *progress, *rate);
                        }
                    }
                    return Ok(None);
                }
                PopupState::Error(_) | PopupState::Success(_) => {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                        self.popup_state = PopupState::Hidden;
                    }
                    return Ok(None);
                }
                _ => {}
            }
        }

        // Handle Ctrl+Z to suspend the application
        if key.code == KeyCode::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) {
            // Use the nix crate to send a SIGTSTP signal to the current process
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                let _ = kill(Pid::this(), Signal::SIGTSTP);
            }
            return Ok(None);
        }

        // Handle input mode
        if self.input_mode == InputMode::Editing {
            match key.code {
                KeyCode::Enter => {
                    // Apply the edited value
                    match self.focus {
                        FocusField::Bucket => self.s3_config.bucket = self.input_buffer.clone(),
                        FocusField::Region => self.s3_config.region = self.input_buffer.clone(),
                        FocusField::Prefix => self.s3_config.prefix = self.input_buffer.clone(),
                        FocusField::EndpointUrl => self.s3_config.endpoint_url = self.input_buffer.clone(),
                        FocusField::AccessKeyId => self.s3_config.access_key_id = self.input_buffer.clone(),
                        FocusField::SecretAccessKey => self.s3_config.secret_access_key = self.input_buffer.clone(),
                        FocusField::PathStyle => {
                            self.s3_config.path_style = self.input_buffer.to_lowercase() == "true";
                        }
                        FocusField::PgHost => {
                            if let Some(host) = &mut self.pg_config.host {
                                *host = self.input_buffer.clone();
                            } else {
                                self.pg_config.host = Some(self.input_buffer.clone());
                            }
                        }
                        FocusField::PgPort => {
                            if let Ok(port) = self.input_buffer.parse::<u16>() {
                                self.pg_config.port = Some(port);
                            }
                        }
                        FocusField::PgUsername => {
                            if let Some(username) = &mut self.pg_config.username {
                                *username = self.input_buffer.clone();
                            } else {
                                self.pg_config.username = Some(self.input_buffer.clone());
                            }
                        }
                        FocusField::PgPassword => {
                            if let Some(password) = &mut self.pg_config.password {
                                *password = self.input_buffer.clone();
                            } else {
                                self.pg_config.password = Some(self.input_buffer.clone());
                            }
                        }
                        FocusField::PgSsl => {
                            self.pg_config.use_ssl = self.input_buffer.to_lowercase() == "true";
                        }
                        FocusField::PgDbName => {
                            if let Some(db_name) = &mut self.pg_config.db_name {
                                *db_name = self.input_buffer.clone();
                            } else {
                                self.pg_config.db_name = Some(self.input_buffer.clone());
                            }
                        }
                        _ => {}
                    }
                    self.input_mode = InputMode::Normal;
                    
                    // Update S3 client with new settings if S3 settings were changed
                    if matches!(self.focus, 
                        FocusField::Bucket | 
                        FocusField::Region | 
                        FocusField::Prefix | 
                        FocusField::EndpointUrl | 
                        FocusField::AccessKeyId | 
                        FocusField::SecretAccessKey | 
                        FocusField::PathStyle
                    ) {
                        self.snapshot_browser.s3_config = self.s3_config.clone();
                        let _ = self.snapshot_browser.init_client().await;
                        
                        // Reload snapshots with new settings
                        if let Err(e) = self.snapshot_browser.load_snapshots().await {
                            debug!("Failed to load snapshots: {}", e);
                        }
                    }
                }
                KeyCode::Esc => {
                    if self.input_mode == InputMode::Editing {
                        // Cancel editing
                        self.input_mode = InputMode::Normal;
                        self.input_buffer.clear();
                    } else if self.popup_state != PopupState::Hidden {
                        // Dismiss any popup
                        self.popup_state = PopupState::Hidden;
                    }
                }
                KeyCode::Backspace => {
                    // Remove character
                    self.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    // Add character
                    self.input_buffer.push(c);
                }
                _ => {}
            }
            return Ok(None);
        }

        // Normal mode
        match key.code {
            KeyCode::Char('q') => {
                // Quit
                return Ok(Some("quit".to_string()));
            }
            KeyCode::Char('r') => {
                // Reload snapshots
                if let Err(e) = self.snapshot_browser.load_snapshots().await {
                    debug!("Failed to load snapshots: {}", e);
                }
            }
            KeyCode::Char('t') => {
                // Test S3 connection when focus is on S3 settings window
                if matches!(self.focus, 
                    FocusField::Bucket | 
                    FocusField::Region | 
                    FocusField::Prefix | 
                    FocusField::EndpointUrl | 
                    FocusField::AccessKeyId | 
                    FocusField::SecretAccessKey | 
                    FocusField::PathStyle
                ) {
                    // Show testing popup
                    self.popup_state = PopupState::TestingS3;
                    
                    // Test connection and update popup state with result
                    if let Err(e) = self.s3_config.test_connection(|state| self.popup_state = state).await {
                        debug!("S3 connection test failed: {}", e);
                    }
                }
            },
            KeyCode::Char('p') => {
                // Test PostgreSQL connection when focus is on PostgreSQL settings window
                if matches!(self.focus, 
                    FocusField::PgHost | 
                    FocusField::PgPort | 
                    FocusField::PgUsername | 
                    FocusField::PgPassword | 
                    FocusField::PgSsl | 
                    FocusField::PgDbName
                ) {
                    // Only test if required fields are set
                    if self.pg_config.host.is_some() && 
                       self.pg_config.port.is_some() && 
                       self.pg_config.db_name.is_some() {
                        // Show testing popup
                        self.popup_state = PopupState::TestingPg;
                        
                        // Test connection and update popup state with result
                        if let Err(e) = self.pg_config.test_connection(|state| self.popup_state = state).await {
                            debug!("PostgreSQL connection test failed: {}", e);
                        }
                    }
                }
            }
            KeyCode::Tab => {
                // Cycle between main window sections only
                self.focus = match self.focus {
                    // S3 Settings fields - move to Restore Target settings
                    FocusField::Bucket | 
                    FocusField::Region | 
                    FocusField::Prefix | 
                    FocusField::EndpointUrl | 
                    FocusField::AccessKeyId | 
                    FocusField::SecretAccessKey | 
                    FocusField::PathStyle => {
                        // Move to restore target settings
                        match self.restore_target {
                            RestoreTarget::Postgres => FocusField::PgHost,
                            RestoreTarget::Elasticsearch => FocusField::EsHost,
                            RestoreTarget::Qdrant => FocusField::EsHost, // This should be updated to QdrantHost when available
                        }
                    }
                    // Restore Target settings - move to Snapshot List
                    FocusField::PgHost | 
                    FocusField::PgPort | 
                    FocusField::PgUsername | 
                    FocusField::PgPassword | 
                    FocusField::PgSsl | 
                    FocusField::PgDbName |
                    FocusField::EsHost |
                    FocusField::EsIndex |
                    FocusField::QdrantApiKey => FocusField::SnapshotList,
                    // Snapshot list - move back to S3 Settings
                    FocusField::SnapshotList => FocusField::Bucket,
                    // Default case
                    _ => FocusField::Bucket,
                };
            }
            // The 'e' key is no longer needed for editing as Enter now handles this
            KeyCode::Char('e') => {
                // Keeping this for backward compatibility, but functionality moved to Enter key
            }
            // Handle navigation within windows using up/down arrows
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focus == FocusField::SnapshotList && !self.snapshot_browser.snapshots.is_empty() {
                    // Navigate snapshot list
                    self.snapshot_browser.selected_index = 
                        (self.snapshot_browser.selected_index + 1) % self.snapshot_browser.snapshots.len();
                } else {
                    // Navigate within settings windows
                    self.focus = match self.focus {
                        // S3 Settings navigation
                        FocusField::Bucket => FocusField::Region,
                        FocusField::Region => FocusField::Prefix,
                        FocusField::Prefix => FocusField::EndpointUrl,
                        FocusField::EndpointUrl => FocusField::AccessKeyId,
                        FocusField::AccessKeyId => FocusField::SecretAccessKey,
                        FocusField::SecretAccessKey => FocusField::PathStyle,
                        FocusField::PathStyle => FocusField::Bucket, // Wrap around to first field
                        
                        // PostgreSQL Settings navigation
                        FocusField::PgHost => FocusField::PgPort,
                        FocusField::PgPort => FocusField::PgUsername,
                        FocusField::PgUsername => FocusField::PgPassword,
                        FocusField::PgPassword => FocusField::PgSsl,
                        FocusField::PgSsl => FocusField::PgDbName,
                        FocusField::PgDbName => FocusField::PgHost, // Wrap around to first field
                        
                        // Elasticsearch Settings navigation
                        FocusField::EsHost => FocusField::EsIndex,
                        FocusField::EsIndex => FocusField::EsHost, // Wrap around to first field
                        
                        // Qdrant Settings navigation
                        FocusField::QdrantApiKey => FocusField::QdrantApiKey, // Only one field for now
                        
                        // Default case - don't change focus
                        _ => self.focus,
                    };
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus == FocusField::SnapshotList && !self.snapshot_browser.snapshots.is_empty() {
                    // Navigate snapshot list
                    self.snapshot_browser.selected_index = if self.snapshot_browser.selected_index == 0 {
                        self.snapshot_browser.snapshots.len() - 1
                    } else {
                        self.snapshot_browser.selected_index - 1
                    };
                } else {
                    // Navigate within settings windows
                    self.focus = match self.focus {
                        // S3 Settings navigation (reverse)
                        FocusField::Bucket => FocusField::PathStyle,
                        FocusField::Region => FocusField::Bucket,
                        FocusField::Prefix => FocusField::Region,
                        FocusField::EndpointUrl => FocusField::Prefix,
                        FocusField::AccessKeyId => FocusField::EndpointUrl,
                        FocusField::SecretAccessKey => FocusField::AccessKeyId,
                        FocusField::PathStyle => FocusField::SecretAccessKey,
                        
                        // PostgreSQL Settings navigation (reverse)
                        FocusField::PgHost => FocusField::PgDbName,
                        FocusField::PgPort => FocusField::PgHost,
                        FocusField::PgUsername => FocusField::PgPort,
                        FocusField::PgPassword => FocusField::PgUsername,
                        FocusField::PgSsl => FocusField::PgPassword,
                        FocusField::PgDbName => FocusField::PgSsl,
                        
                        // Elasticsearch Settings navigation (reverse)
                        FocusField::EsHost => FocusField::EsIndex,
                        FocusField::EsIndex => FocusField::EsHost,
                        
                        // Qdrant Settings navigation (reverse)
                        FocusField::QdrantApiKey => FocusField::QdrantApiKey, // Only one field for now
                        
                        // Default case - don't change focus
                        _ => self.focus,
                    };
                }
            }
            KeyCode::Enter => {
                // Handle different actions based on focus
                if self.focus == FocusField::SnapshotList && !self.snapshot_browser.snapshots.is_empty() {
                    // Select snapshot for restore
                    let snapshot = &self.snapshot_browser.snapshots[self.snapshot_browser.selected_index];
                    self.popup_state = PopupState::ConfirmRestore(snapshot.clone());
                } else if false { // TestS3Button removed
                    // Test S3 connection when Enter is pressed on the test button
                    self.popup_state = PopupState::TestingS3;
                    
                    // Test connection and update popup state with result
                    if let Err(e) = self.s3_config.test_connection(|state| self.popup_state = state).await {
                        debug!("S3 connection test failed: {}", e);
                    }
                } else if self.focus != FocusField::SnapshotList {
                    // For any field that's not the snapshot list, enter edit mode
                    self.input_mode = InputMode::Editing;
                    // Set input buffer to current value
                    self.input_buffer = match self.focus {
                        FocusField::Bucket => self.s3_config.bucket.clone(),
                        FocusField::Region => self.s3_config.region.clone(),
                        FocusField::Prefix => self.s3_config.prefix.clone(),
                        FocusField::EndpointUrl => self.s3_config.endpoint_url.clone(),
                        FocusField::AccessKeyId => self.s3_config.access_key_id.clone(),
                        FocusField::SecretAccessKey => self.s3_config.secret_access_key.clone(),
                        FocusField::PathStyle => self.s3_config.path_style.to_string(),
                        FocusField::PgHost => self.pg_config.host.clone().unwrap_or_default(),
                        FocusField::PgPort => self.pg_config.port.map(|p| p.to_string()).unwrap_or_default(),
                        FocusField::PgUsername => self.pg_config.username.clone().unwrap_or_default(),
                        FocusField::PgPassword => self.pg_config.password.clone().unwrap_or_default(),
                        FocusField::PgSsl => self.pg_config.use_ssl.to_string(),
                        FocusField::PgDbName => self.pg_config.db_name.clone().unwrap_or_default(),
                        FocusField::EsHost => self.es_config.host.clone().unwrap_or_default(),
                        FocusField::EsIndex => self.es_config.index.clone().unwrap_or_default(),
                        FocusField::QdrantApiKey => self.qdrant_config.api_key.clone().unwrap_or_default(),
                        _ => String::new(),
                    };
                }
            }
            // Add restore target selection with 1, 2, 3 keys
            KeyCode::Char('1') => {
                self.restore_target = RestoreTarget::Postgres;
                // Set focus to first PostgreSQL field if not already on a PostgreSQL field
                if !matches!(self.focus, 
                    FocusField::PgHost | 
                    FocusField::PgPort | 
                    FocusField::PgUsername | 
                    FocusField::PgPassword | 
                    FocusField::PgSsl | 
                    FocusField::PgDbName
                ) {
                    self.focus = FocusField::PgHost;
                }
            }
            KeyCode::Char('2') => {
                self.restore_target = RestoreTarget::Elasticsearch;
                // Set focus to first Elasticsearch field if not already on an Elasticsearch field
                if !matches!(self.focus, 
                    FocusField::EsHost | 
                    FocusField::EsIndex
                ) {
                    self.focus = FocusField::EsHost;
                }
            }
            KeyCode::Char('3') => {
                self.restore_target = RestoreTarget::Qdrant;
                // Set focus to first Qdrant field if not already on a Qdrant field
                if !matches!(self.focus, 
                    FocusField::EsHost | 
                    FocusField::EsIndex | 
                    FocusField::QdrantApiKey
                ) {
                    self.focus = FocusField::EsHost;
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Restore a database from a downloaded snapshot file
    /// Get the current restore target based on the selected target type
    fn get_current_restore_target(&self) -> Box<dyn crate::restore::RestoreTarget + Send + Sync> {
        match self.restore_target {
            RestoreTarget::Postgres => Box::new(crate::targets::PostgresRestoreTarget {
                config: self.pg_config.clone(),
            }),
            RestoreTarget::Elasticsearch => Box::new(crate::targets::ElasticsearchRestoreTarget {
                config: self.es_config.clone(),
            }),
            RestoreTarget::Qdrant => Box::new(crate::targets::QdrantRestoreTarget {
                config: self.qdrant_config.clone(),
            }),
        }
    }

    pub async fn restore_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, file_path: &str) -> Result<()> {
        use std::path::Path;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use tokio::time::sleep;
        use std::time::Duration;
        
        // Create the appropriate restore target based on the selected target type
        let restore_target = self.get_current_restore_target();
        
        // Check if the target is properly configured
        if !restore_target.is_configured() {
            let required = restore_target.required_fields().join(", ");
            return Err(anyhow!("Restore target not properly configured. Required fields: {}", required));
        }
        
        // Update UI to show initial progress
        self.popup_state = PopupState::Restoring(snapshot.clone(), 0.0);
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
        
        // Use a separate thread-safe mechanism to track progress
        let progress = Arc::new(std::sync::Mutex::new(0.0f32));
        let is_done = Arc::new(AtomicBool::new(false));
        
        // Clone for the progress callback
        let progress_for_callback = Arc::clone(&progress);
        let is_done_for_callback = Arc::clone(&is_done);
        
        // Create a thread-safe progress callback
        let progress_callback = Box::new(move |prog: f32| {
            // Update the shared progress value
            if let Ok(mut p) = progress_for_callback.lock() {
                *p = prog;
                
                // If we're at 100%, mark as done
                if prog >= 1.0 {
                    is_done_for_callback.store(true, Ordering::SeqCst);
                }
            }
        });
        
        // Start the restore operation in the background
        let file_path_owned = file_path.to_string();
        let restore_handle = tokio::spawn(async move {
            let path = Path::new(&file_path_owned);
            restore_target.restore_snapshot(path, Some(progress_callback)).await
        });
        
        // Update the UI periodically while the restore is in progress
        let snapshot_clone = snapshot.clone();
        while !is_done.load(Ordering::SeqCst) {
            // Get the current progress
            let current_progress = if let Ok(p) = progress.lock() {
                *p
            } else {
                0.0
            };
            
            // Update the UI
            self.popup_state = PopupState::Restoring(snapshot_clone.clone(), current_progress);
            terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
            
            // Check if the restore operation is done
            if restore_handle.is_finished() {
                break;
            }
            
            // Wait a short time before updating again
            sleep(Duration::from_millis(100)).await;
        }
        
        // Get the result of the restore operation
        let result = match restore_handle.await {
            Ok(res) => res,
            Err(e) => Err(anyhow!("Restore task failed: {}", e)),
        };
        
        // Handle the result
        match result {
            Ok(message) => {
                debug!("{}", message);
                // Show success message
                self.popup_state = PopupState::Success(message);
                terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                Ok(())
            },
            Err(e) => {
                debug!("Restore failed: {}", e);
                // Show error message
                self.popup_state = PopupState::Error(format!("Restore failed: {}", e));
                terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                Err(anyhow!("Restore failed: {}", e))
            }
        }
    }
}
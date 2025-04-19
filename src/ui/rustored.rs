use crate::ui::models::{S3Config, PostgresConfig, ElasticsearchConfig, QdrantConfig, PopupState, InputMode, FocusField, RestoreTarget, BackupMetadata};
use crate::ui::browser::SnapshotBrowser;
use crate::ui::key_handler;
use ratatui::backend::Backend;
use ratatui::Terminal;
use anyhow::{Result, anyhow};
use log::debug;

/// Main application state struct
/// 
/// This struct holds all the state for the Rustored application, including
/// configuration for various restore targets, UI state, and the snapshot browser.
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
    /// 
    /// # Arguments
    /// 
    /// * Various configuration parameters for S3, PostgreSQL, Elasticsearch, and Qdrant
    /// 
    /// # Returns
    /// 
    /// A new RustoredApp instance with the provided configuration
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
        debug!("Creating new RustoredApp instance");
        debug!("S3 settings: bucket: {:?}, region: {:?}, prefix: {:?}, endpoint: {:?}, path_style: {}", 
               bucket, region, prefix, endpoint_url, path_style);
        debug!("PostgreSQL settings: host: {:?}, port: {:?}, username: {:?}, use_ssl: {}, db_name: {:?}", 
               host, port, username, use_ssl, db_name);
        debug!("Elasticsearch settings: host: {:?}, index: {:?}", es_host, es_index);
        debug!("Qdrant settings: host: {:?}, collection: {:?}", es_host, es_index);
        
        // Create S3 configuration
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
        
        // Create PostgreSQL configuration
        let pg_config = PostgresConfig {
            host: host.clone(),
            port: *port,
            username: username.clone(),
            password: password.clone(),
            use_ssl,
            db_name: db_name.clone(),
            ..Default::default()
        };
        
        // Create Elasticsearch configuration
        let es_config = ElasticsearchConfig {
            host: es_host.clone(),
            index: es_index.clone(),
        };
        
        // Create Qdrant configuration
        let qdrant_config = QdrantConfig {
            host: es_host.clone(),
            collection: es_index.clone(),
            api_key: qdrant_api_key.clone(),
        };
        
        // Create snapshot browser with S3 configuration
        let snapshot_browser = SnapshotBrowser::new(s3_config.clone());
        
        // Create and return the RustoredApp instance
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
    /// 
    /// # Arguments
    /// 
    /// * `terminal` - A mutable reference to the terminal
    /// 
    /// # Returns
    /// 
    /// A Result containing an Option<String> which is Some if a snapshot path is returned
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<Option<String>> {
        debug!("Starting RustoredApp main loop");
        // Delegate to existing run_app with full app state
        crate::ui::app::run_app(terminal, self).await
    }

    /// Handle key events and return a snapshot path if one is downloaded
    /// 
    /// This function delegates to the key_handler module for specific key handling logic
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key event to process
    /// 
    /// # Returns
    /// 
    /// A Result containing an Option<String> which is Some if a snapshot path is returned
    pub async fn handle_key_event<B: Backend>(&mut self, key: crossterm::event::KeyEvent) -> Result<Option<String>> {
        debug!("Handling key event: {:?}", key);
        debug!("Current focus: {:?}, input mode: {:?}", self.focus, self.input_mode);
        use crossterm::event::{KeyCode, KeyModifiers};

        // Handle popup states first
        if self.popup_state != PopupState::Hidden {
            return key_handler::handle_popup_events(self, key).await;
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
            return key_handler::handle_editing_mode(self, key).await;
        }

        // Normal mode
        key_handler::handle_normal_mode(self, key).await
    }

    /// Get the current restore target based on the selected target type
    /// 
    /// # Returns
    /// 
    /// A boxed trait object implementing the RestoreTarget trait
    pub fn get_current_restore_target(&self) -> Box<dyn crate::restore::RestoreTarget + Send + Sync> {
        debug!("Getting current restore target for type: {:?}", self.restore_target);
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

    /// Restore a snapshot from a downloaded file
    /// 
    /// # Arguments
    /// 
    /// * `snapshot` - The snapshot metadata
    /// * `terminal` - A mutable reference to the terminal
    /// * `file_path` - The path to the downloaded snapshot file
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or an error
    pub async fn restore_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, file_path: &str) -> Result<()> {
        debug!("Starting restore of snapshot: {:?} from file: {}", snapshot, file_path);
        debug!("Using restore target: {:?}", self.restore_target);
        use std::path::{Path, PathBuf};
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
        
        // Create a cancellation flag that can be shared between threads
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();
        
        // Spawn a task to update the progress UI
        let progress_handle = tokio::spawn(async move {
            let mut progress = 0.0;
            while progress < 100.0 && !cancelled.load(Ordering::SeqCst) {
                progress += 1.0;
                if progress > 100.0 {
                    progress = 100.0;
                }
                sleep(Duration::from_millis(50)).await;
            }
            progress
        });
        
        // Perform the actual restore operation
        debug!("Calling restore_snapshot on target");
        let file_path = Path::new(file_path);
        let restore_result = restore_target.restore_snapshot(file_path, None).await;
        
        // Cancel the progress task
        cancelled_clone.store(true, Ordering::SeqCst);
        
        // Wait for the progress task to complete
        let _ = progress_handle.await;
        
        // Update UI based on restore result
        match restore_result {
            Ok(result) => {
                debug!("Restore completed successfully: {}", result);
                self.popup_state = PopupState::Success(format!("Restored to {}", result));
            }
            Err(e) => {
                debug!("Restore failed: {}", e);
                self.popup_state = PopupState::Error(format!("Restore failed: {}", e));
            }
        }
        
        // Draw the final UI state
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, self))?;
        
        Ok(())
    }
}

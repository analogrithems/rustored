use crate::ui::models::{S3Config, PostgresConfig, ElasticsearchConfig, QdrantConfig, PopupState, InputMode, FocusField, RestoreTarget, BackupMetadata};
use crate::ui::browser::SnapshotBrowser;
use ratatui::backend::Backend;
use ratatui::Terminal;
use anyhow::{Result, anyhow};
use log::{debug, info};

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
        // Delegate to snapshot browser's key handling
        self.snapshot_browser.handle_key_event::<B>(key, &mut self.popup_state, &mut self.input_mode, &mut self.input_buffer, &mut self.focus, &mut self.s3_config).await
    }

    /// Restore a database from a downloaded snapshot file
    pub async fn restore_snapshot<B: Backend>(&mut self, snapshot: &BackupMetadata, terminal: &mut Terminal<B>, file_path: &str) -> Result<()> {
        let mut progress = 0.0;

        // Update UI to show progress
        self.popup_state = PopupState::Restoring(snapshot.clone(), progress);
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
        self.popup_state = PopupState::Restoring(snapshot.clone(), progress);
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
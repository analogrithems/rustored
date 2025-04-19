use crate::ui::models::{S3Config, PostgresConfig, ElasticsearchConfig, QdrantConfig, PopupState, InputMode, FocusField, RestoreTarget};
use crate::ui::browser::SnapshotBrowser;
use ratatui::backend::Backend;
use ratatui::Terminal;
use anyhow::Result;

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
        bucket: Option<String>,
        region: Option<String>,
        prefix: Option<String>,
        endpoint_url: Option<String>,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
        path_style: bool,
        host: Option<String>,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
        use_ssl: bool,
        db_name: Option<String>,
        es_host: Option<String>,
        es_index: Option<String>,
        qdrant_api_key: Option<String>,
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
            port,
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
        let mut snapshot_browser = SnapshotBrowser::new(s3_config.clone(), pg_config.clone());
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
        crate::ui::browser::run_app(terminal, self).await
    }
}
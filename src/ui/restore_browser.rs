use anyhow::{anyhow, Result};
// Import removed: use crossterm::event::KeyCode;
use ratatui::backend::Backend;
use ratatui::Terminal;
use random_word::Lang;

use crate::ui::models::{
    BackupMetadata, ElasticsearchConfig, PopupState, PostgresConfig, QdrantConfig, RestoreTarget,
};

/// Browser for restoring snapshots to different targets
pub struct RestoreBrowser {
    pub restore_target: RestoreTarget,
    pub postgres_config: Option<PostgresConfig>,
    pub elasticsearch_config: Option<ElasticsearchConfig>,
    pub qdrant_config: Option<QdrantConfig>,
    pub popup_state: PopupState,
}

impl RestoreBrowser {
    /// Create a new RestoreBrowser
    pub fn new(
        restore_target: RestoreTarget,
        postgres_config: Option<PostgresConfig>,
        elasticsearch_config: Option<ElasticsearchConfig>,
        qdrant_config: Option<QdrantConfig>,
    ) -> Self {
        Self {
            restore_target,
            postgres_config,
            elasticsearch_config,
            qdrant_config,
            popup_state: PopupState::Hidden,
        }
    }

    /// Test PostgreSQL connection
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

    /// Restore a database from a downloaded snapshot file
    pub async fn restore_snapshot<B: Backend>(
        &mut self, 
        snapshot: &BackupMetadata, 
        terminal: &mut Terminal<B>, 
        file_path: &str
    ) -> Result<()> {
        // Start restore operation
        self.popup_state = PopupState::Restoring(snapshot.clone(), 0.0);
        terminal.draw(|f| crate::ui::renderer::restore_ui::<B>(f, self))?;
        
        // Handle different restore targets
        match self.restore_target {
            RestoreTarget::Postgres => {
                // Ensure PostgreSQL config exists
                if self.postgres_config.is_none() {
                    self.postgres_config = Some(PostgresConfig::default());
                }
                
                let pg_config = self.postgres_config.as_ref().unwrap();
                
                // Validate PostgreSQL settings
                if pg_config.host.is_none() || pg_config.host.as_ref().unwrap().is_empty() {
                    self.popup_state = PopupState::Error("PostgreSQL host is required".to_string());
                    return Err(anyhow!("PostgreSQL host is required"));
                }

                if pg_config.port.is_none() {
                    self.popup_state = PopupState::Error("PostgreSQL port is required".to_string());
                    return Err(anyhow!("PostgreSQL port is required"));
                }

                if pg_config.username.is_none() || pg_config.username.as_ref().unwrap().is_empty() {
                    self.popup_state = PopupState::Error("PostgreSQL username is required".to_string());
                    return Err(anyhow!("PostgreSQL username is required"));
                }

                if pg_config.db_name.is_none() || pg_config.db_name.as_ref().unwrap().is_empty() {
                    self.popup_state = PopupState::Error("PostgreSQL database name is required".to_string());
                    return Err(anyhow!("PostgreSQL database name is required"));
                }

                // Use a separate thread for the restore operation to avoid blocking the UI
                let host = pg_config.host.as_ref().unwrap().clone();
                let port = pg_config.port.unwrap();
                let username = pg_config.username.clone();
                let password = pg_config.password.clone();
                let use_ssl = pg_config.use_ssl;
                let file_path_owned = file_path.to_string();

                // Test connection and get client
                let pgclient = self.test_pg_connection().await?;

                // Create a new database for the restore
                let new_dbname = format!("{}-restored", random_word::get(Lang::En));
                let create_restore_db = crate::postgres::create_database(&pgclient.unwrap(), &new_dbname).await;
                match create_restore_db {
                    Ok(_) => {
                        log::info!("Successfully created restore database: {}", new_dbname);
                    },
                    Err(e) => {
                        self.popup_state = PopupState::Error(format!("Failed to create restore database: {}", e));
                        return Err(anyhow!("Failed to create restore database: {}", e));
                    }
                }

                // Restore the database
                let restore_result = crate::backup::restore_database(
                    &new_dbname,
                    &file_path_owned,
                    &host,
                    port,
                    username.as_ref().map(|s| s.as_str()),
                    password.as_ref().map(|s| s.as_str()),
                    use_ssl,
                );

                match restore_result {
                    Ok(_) => {
                        self.popup_state = PopupState::Success(format!("Successfully restored to database: {}", new_dbname));
                        Ok(())
                    },
                    Err(e) => {
                        self.popup_state = PopupState::Error(format!("Failed to restore database: {}", e));
                        Err(anyhow!("Failed to restore database: {}", e))
                    }
                }
            },
            RestoreTarget::Elasticsearch => {
                // Ensure Elasticsearch config exists
                if self.elasticsearch_config.is_none() {
                    self.elasticsearch_config = Some(ElasticsearchConfig::default());
                }
                
                // Implement Elasticsearch restore logic here
                self.popup_state = PopupState::Error("Elasticsearch restore not yet implemented".to_string());
                Err(anyhow!("Elasticsearch restore not yet implemented"))
            },
            RestoreTarget::Qdrant => {
                // Ensure Qdrant config exists
                if self.qdrant_config.is_none() {
                    self.qdrant_config = Some(QdrantConfig::default());
                }
                
                // Implement Qdrant restore logic here
                self.popup_state = PopupState::Error("Qdrant restore not yet implemented".to_string());
                Err(anyhow!("Qdrant restore not yet implemented"))
            },
        }
    }
}

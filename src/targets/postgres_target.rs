use crate::restore::RestoreTarget;
use crate::ui::models::postgres_config::PostgresConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};
use std::path::Path;

/// PostgreSQL restore target implementation
pub struct PostgresRestoreTarget {
    pub config: PostgresConfig,
}

#[async_trait]
impl RestoreTarget for PostgresRestoreTarget {
    fn name(&self) -> &'static str {
        "PostgreSQL"
    }

    fn is_configured(&self) -> bool {
        self.config.host.is_some() && 
        self.config.port.is_some() && 
        self.config.db_name.is_some()
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["host", "port", "database"]
    }

    async fn restore_snapshot(
        &self,
        snapshot_path: &Path,
        progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
    ) -> Result<String> {
        // Get PostgreSQL connection details
        let host = self.config.host.as_ref().ok_or_else(|| anyhow!("PostgreSQL host not specified"))?.clone();
        let port = self.config.port.ok_or_else(|| anyhow!("PostgreSQL port not specified"))?;
        let username = self.config.username.clone();
        let password = self.config.password.clone();
        let use_ssl = self.config.use_ssl;

        // Call the PostgreSQL restore function
        debug!("Restoring to PostgreSQL at {}:{}", host, port);
        
        // Report initial progress
        if let Some(ref callback) = progress_callback {
            callback(0.0);
        }

        let result = crate::postgres::restore_snapshot(
            &host,
            port,
            username,
            password,
            use_ssl,
            snapshot_path.to_str().ok_or_else(|| anyhow!("Invalid snapshot path"))?,
        ).await;

        // Report completion progress
        if let Some(ref callback) = progress_callback {
            callback(1.0);
        }

        match result {
            Ok(db_name) => {
                info!("Restored to PostgreSQL database: {}", db_name);
                Ok(format!("Successfully restored to database: {}", db_name))
            }
            Err(e) => Err(anyhow!("Failed to restore to PostgreSQL: {}", e)),
        }
    }

    async fn test_connection(&self) -> Result<String> {
        // Get PostgreSQL connection details
        let host = self.config.host.as_ref().ok_or_else(|| anyhow!("PostgreSQL host not specified"))?.clone();
        let port = self.config.port.ok_or_else(|| anyhow!("PostgreSQL port not specified"))?;
        let username = self.config.username.clone();
        let password = self.config.password.clone();
        let use_ssl = self.config.use_ssl;

        // Create a PgConfig for connection
        let mut config = tokio_postgres::config::Config::new();
        config.host(&host);
        config.port(port);
        
        if let Some(user) = &username {
            config.user(user);
        }
        
        if let Some(pass) = &password {
            config.password(pass);
        }
        
        // Try to connect to PostgreSQL
        let connect_result = if use_ssl {
            crate::postgres::connect_ssl(&config, false, None).await
        } else {
            crate::postgres::connect_no_ssl(&config).await
        };

        match connect_result {
            Ok(_) => Ok(format!("Successfully connected to PostgreSQL at {}:{}", host, port)),
            Err(e) => Err(anyhow!("Failed to connect to PostgreSQL: {}", e)),
        }
    }
}

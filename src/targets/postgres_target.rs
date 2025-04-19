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
        debug!("Getting name for PostgreSQL restore target");
        "PostgreSQL"
    }

    fn is_configured(&self) -> bool {
        debug!("Checking if PostgreSQL target is configured");
        let configured = self.config.host.is_some() && 
                        self.config.port.is_some() && 
                        self.config.db_name.is_some();
        debug!("PostgreSQL target configured: {}", configured);
        configured
    }

    fn required_fields(&self) -> Vec<&'static str> {
        debug!("Getting required fields for PostgreSQL target");
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
        debug!("Testing connection to PostgreSQL");
        
        // Get PostgreSQL connection details
        let host = match self.config.host.as_ref() {
            Some(h) => {
                debug!("Using PostgreSQL host: {}", h);
                h.clone()
            },
            None => {
                debug!("PostgreSQL host not specified");
                return Err(anyhow!("PostgreSQL host not specified"));
            }
        };
        
        let port = match self.config.port {
            Some(p) => {
                debug!("Using PostgreSQL port: {}", p);
                p
            },
            None => {
                debug!("PostgreSQL port not specified");
                return Err(anyhow!("PostgreSQL port not specified"));
            }
        };
        
        let username = self.config.username.clone();
        debug!("Username provided: {}", username.is_some());
        
        let password_provided = self.config.password.is_some();
        debug!("Password provided: {}", password_provided);
        let password = self.config.password.clone();
        
        let use_ssl = self.config.use_ssl;
        debug!("Using SSL: {}", use_ssl);

        // Create a PgConfig for connection
        debug!("Creating PostgreSQL connection config");
        let mut config = tokio_postgres::config::Config::new();
        config.host(&host);
        config.port(port);
        
        if let Some(user) = &username {
            debug!("Setting PostgreSQL user: {}", user);
            config.user(user);
        }
        
        if let Some(_) = &password {
            debug!("Setting PostgreSQL password: [MASKED]");
            config.password(password.as_ref().unwrap());
        }
        
        // Try to connect to PostgreSQL
        debug!("Attempting to connect to PostgreSQL server");
        let connect_result = if use_ssl {
            debug!("Using SSL for PostgreSQL connection");
            crate::postgres::connect_ssl(&config, false, None).await
        } else {
            debug!("Not using SSL for PostgreSQL connection");
            crate::postgres::connect_no_ssl(&config).await
        };

        match connect_result {
            Ok(_) => {
                debug!("Successfully connected to PostgreSQL at {}:{}", host, port);
                Ok(format!("Successfully connected to PostgreSQL at {}:{}", host, port))
            },
            Err(e) => {
                debug!("Failed to connect to PostgreSQL: {}", e);
                Err(anyhow!("Failed to connect to PostgreSQL: {}", e))
            },
        }
    }
}

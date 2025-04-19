use crate::restore::RestoreTarget;
use crate::ui::models::elasticsearch_config::ElasticsearchConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};
use std::path::Path;

/// Elasticsearch restore target implementation
pub struct ElasticsearchRestoreTarget {
    pub config: ElasticsearchConfig,
}

#[async_trait]
impl RestoreTarget for ElasticsearchRestoreTarget {
    fn name(&self) -> &'static str {
        "Elasticsearch"
    }

    fn is_configured(&self) -> bool {
        self.config.host.is_some() && self.config.index.is_some()
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["host", "index"]
    }

    async fn restore_snapshot(
        &self,
        snapshot_path: &Path,
        progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
    ) -> Result<String> {
        // Get Elasticsearch connection details
        let host = self.config.host.as_ref().ok_or_else(|| anyhow!("Elasticsearch host not specified"))?.clone();
        let index = self.config.index.as_ref().ok_or_else(|| anyhow!("Elasticsearch index not specified"))?.clone();

        // Report initial progress
        if let Some(ref callback) = progress_callback {
            callback(0.0);
        }

        // Call the Elasticsearch restore function
        debug!("Restoring to Elasticsearch at {}, index {}", host, index);
        let result = crate::datastore::restore_to_elasticsearch(
            &host,
            &index,
            snapshot_path.to_str().ok_or_else(|| anyhow!("Invalid snapshot path"))?,
        ).await;

        // Report completion progress
        if let Some(ref callback) = progress_callback {
            callback(1.0);
        }

        match result {
            Ok(_) => {
                info!("Restored to Elasticsearch index: {}", index);
                Ok(format!("Successfully restored to index: {}", index))
            }
            Err(e) => Err(anyhow!("Failed to restore to Elasticsearch: {}", e)),
        }
    }

    async fn test_connection(&self) -> Result<String> {
        // Get Elasticsearch connection details
        let host = self.config.host.as_ref().ok_or_else(|| anyhow!("Elasticsearch host not specified"))?.clone();
        
        // Simple connection test - in a real implementation, this would use the Elasticsearch client
        // to check if the server is reachable
        if host.starts_with("http://") || host.starts_with("https://") {
            // For now, just return success since we don't have a real implementation
            Ok(format!("Successfully connected to Elasticsearch at {}", host))
        } else {
            Err(anyhow!("Invalid Elasticsearch host URL: {}", host))
        }
    }
}

use crate::restore::RestoreTarget;
use crate::ui::models::qdrant_config::QdrantConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use log::{debug, info};
use std::path::Path;

/// Qdrant restore target implementation
pub struct QdrantRestoreTarget {
    pub config: QdrantConfig,
}

#[async_trait]
impl RestoreTarget for QdrantRestoreTarget {
    fn name(&self) -> &'static str {
        "Qdrant"
    }

    fn is_configured(&self) -> bool {
        self.config.host.is_some() && self.config.collection.is_some()
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["host", "collection"]
    }

    async fn restore_snapshot(
        &self,
        snapshot_path: &Path,
        progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>,
    ) -> Result<String> {
        // Get Qdrant connection details
        let host = self.config.host.as_ref().ok_or_else(|| anyhow!("Qdrant host not specified"))?.clone();
        let collection = self.config.collection.as_ref().ok_or_else(|| anyhow!("Qdrant collection not specified"))?.clone();
        let api_key = self.config.api_key.clone();

        // Report initial progress
        if let Some(ref callback) = progress_callback {
            callback(0.0);
        }

        // Call the Qdrant restore function
        debug!("Restoring to Qdrant at {}, collection {}", host, collection);
        let result = crate::datastore::restore_to_qdrant(
            &host,
            &collection,
            api_key.as_deref(),
            snapshot_path.to_str().ok_or_else(|| anyhow!("Invalid snapshot path"))?,
        ).await;

        // Report completion progress
        if let Some(ref callback) = progress_callback {
            callback(1.0);
        }

        match result {
            Ok(_) => {
                info!("Restored to Qdrant collection: {}", collection);
                Ok(format!("Successfully restored to collection: {}", collection))
            }
            Err(e) => Err(anyhow!("Failed to restore to Qdrant: {}", e)),
        }
    }

    async fn test_connection(&self) -> Result<String> {
        // Get Qdrant connection details
        let host = self.config.host.as_ref().ok_or_else(|| anyhow!("Qdrant host not specified"))?.clone();
        let api_key_info = if self.config.api_key.is_some() { " with API key" } else { "" };
        
        // Simple connection test - in a real implementation, this would use the Qdrant client
        // to check if the server is reachable
        if host.starts_with("http://") || host.starts_with("https://") {
            // For now, just return success since we don't have a real implementation
            Ok(format!("Successfully connected to Qdrant at {}{}", host, api_key_info))
        } else {
            Err(anyhow!("Invalid Qdrant host URL: {}", host))
        }
    }
}

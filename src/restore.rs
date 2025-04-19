use anyhow::Result;
use std::path::Path;
use async_trait::async_trait;

/// Trait for restore targets
/// 
/// This trait defines the interface for restoring snapshots to different targets.
/// Each restore target type (PostgreSQL, Elasticsearch, Qdrant) should implement this trait.
#[async_trait]
pub trait RestoreTarget {
    /// Get the name of this restore target type
    fn name(&self) -> &'static str;
    
    /// Check if the target is properly configured
    fn is_configured(&self) -> bool;
    
    /// Get a list of required configuration fields
    fn required_fields(&self) -> Vec<&'static str>;
    
    /// Restore a snapshot to this target
    /// 
    /// # Arguments
    /// * `snapshot_path` - Path to the snapshot file
    /// * `progress_callback` - Optional callback for reporting progress (0.0 to 1.0)
    async fn restore_snapshot(
        &self, 
        snapshot_path: &Path, 
        progress_callback: Option<Box<dyn Fn(f32) + Send + Sync>>
    ) -> Result<String>;
    
    /// Test the connection to this target
    async fn test_connection(&self) -> Result<String>;
}

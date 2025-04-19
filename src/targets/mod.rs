mod postgres_target;
mod elasticsearch_target;
mod qdrant_target;

pub use postgres_target::PostgresRestoreTarget;
pub use elasticsearch_target::ElasticsearchRestoreTarget;
pub use qdrant_target::QdrantRestoreTarget;

use crate::restore::RestoreTarget;
use crate::datastore::RestoreTarget as RestoreTargetEnum;

/// Factory function to create a restore target based on the target type
pub fn create_restore_target(
    target_type: RestoreTargetEnum,
    pg_config: crate::ui::models::postgres_config::PostgresConfig,
    es_config: crate::ui::models::elasticsearch_config::ElasticsearchConfig,
    qdrant_config: crate::ui::models::qdrant_config::QdrantConfig,
) -> Box<dyn RestoreTarget + Send + Sync> {
    match target_type {
        RestoreTargetEnum::Postgres => Box::new(PostgresRestoreTarget { config: pg_config }),
        RestoreTargetEnum::Elasticsearch => Box::new(ElasticsearchRestoreTarget { config: es_config }),
        RestoreTargetEnum::Qdrant => Box::new(QdrantRestoreTarget { config: qdrant_config }),
    }
}

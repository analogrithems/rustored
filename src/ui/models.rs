use aws_sdk_s3::primitives::DateTime as AwsDateTime;
use std::fmt;
use std::fmt::Debug;

// Re-export separated config modules
pub mod s3_config;
pub use s3_config::S3Config;

/// Restore target options
#[derive(Clone, Debug, PartialEq, Default)]
pub enum RestoreTarget {
    #[default]
    Postgres,
    Elasticsearch,
    Qdrant,
}

pub mod postgres_config;
pub use postgres_config::PostgresConfig;
pub mod elasticsearch_config;
pub use elasticsearch_config::ElasticsearchConfig;
pub mod qdrant_config;
pub use qdrant_config::QdrantConfig;

/// Input mode for the UI
#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Metadata for a backup
#[derive(Clone, Debug, PartialEq)]
pub struct BackupMetadata {
    pub key: String,
    pub size: i64,
    pub last_modified: AwsDateTime,
}

/// State of the popup
#[derive(Debug, PartialEq)]
pub enum PopupState {
    Hidden,
    ConfirmRestore(BackupMetadata),
    Downloading(BackupMetadata, f32, f64),
    ConfirmCancel(BackupMetadata, f32, f64),
    Restoring(BackupMetadata, f32),  // Snapshot being restored, progress percentage
    TestS3Result(String),
    TestPgResult(String),
    Error(String),
    Success(String),
}

/// Focus field for the UI
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FocusField {
    // S3 Settings (10-19)
    Bucket,          // Alt+1
    Region,          // Alt+2
    Prefix,          // Alt+3
    EndpointUrl,     // Alt+4
    AccessKeyId,     // Alt+5
    SecretAccessKey, // Alt+6
    PathStyle,       // Alt+7

    // PostgreSQL Settings (20-29)
    PgHost,          // Alt+q
    PgPort,          // Alt+w
    PgUsername,      // Alt+e
    PgPassword,      // Alt+r
    PgSsl,          // Alt+t
    PgDbName,        // Alt+y
    SnapshotList,
    RestoreTarget,
    EsHost,
    EsIndex,
    QdrantApiKey,
}

impl fmt::Display for FocusField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FocusField::Bucket => write!(f, "Bucket"),
            FocusField::Region => write!(f, "Region"),
            FocusField::Prefix => write!(f, "Prefix"),
            FocusField::EndpointUrl => write!(f, "Endpoint URL"),
            FocusField::AccessKeyId => write!(f, "Access Key ID"),
            FocusField::SecretAccessKey => write!(f, "Secret Access Key"),
            FocusField::PathStyle => write!(f, "Path Style"),
            FocusField::SnapshotList => write!(f, "Snapshot List"),

            // Datastore Settings
            FocusField::RestoreTarget => write!(f, "Restore Target"),
            // PostgreSQL Settings (20-29)
            FocusField::PgHost => write!(f, "PostgreSQL Host"),
            FocusField::PgPort => write!(f, "PostgreSQL Port"),
            FocusField::PgUsername => write!(f, "PostgreSQL Username"),
            FocusField::PgPassword => write!(f, "PostgreSQL Password"),
            FocusField::PgSsl => write!(f, "PostgreSQL SSL"),
            FocusField::PgDbName => write!(f, "PostgreSQL Database"),
            // Elasticsearch Settings (30-39)
            FocusField::EsHost => write!(f, "Elasticsearch/Qdrant Host"),
            FocusField::EsIndex => write!(f, "Index/Collection"),
            // Qdrant Settings (40-49)
            FocusField::QdrantApiKey => write!(f, "Qdrant API Key"),
        }
    }
}

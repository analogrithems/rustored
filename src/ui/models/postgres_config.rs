/// Configuration for PostgreSQL connection
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PostgresConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_ssl: bool,
    pub db_name: Option<String>,
}


pub struct PostgresRestoreSettings {
    // Restore Checkboxes
    pub no_data: bool,
    pub no_schema: bool,
    pub no_owner: bool,
    pub no_comments: bool,
}
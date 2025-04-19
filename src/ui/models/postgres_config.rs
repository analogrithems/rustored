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

impl PostgresConfig {
    /// Get all focus fields for PostgreSQL settings
    pub fn focus_fields() -> &'static [super::FocusField] {
        use super::FocusField;
        &[
            FocusField::PgHost,
            FocusField::PgPort,
            FocusField::PgUsername,
            FocusField::PgPassword,
            FocusField::PgSsl,
            FocusField::PgDbName,
        ]
    }

    /// Get the field value for a given focus field
    pub fn get_field_value(&self, field: super::FocusField) -> String {
        use super::FocusField;
        match field {
            FocusField::PgHost => self.host.clone().unwrap_or_default(),
            FocusField::PgPort => self.port.map(|p| p.to_string()).unwrap_or_default(),
            FocusField::PgUsername => self.username.clone().unwrap_or_default(),
            FocusField::PgPassword => self.password.clone().unwrap_or_default(),
            FocusField::PgSsl => self.use_ssl.to_string(),
            FocusField::PgDbName => self.db_name.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    /// Set a field value from a string
    pub fn set_field_value(&mut self, field: super::FocusField, value: String) {
        use super::FocusField;
        match field {
            FocusField::PgHost => self.host = Some(value),
            FocusField::PgPort => self.port = value.parse().ok(),
            FocusField::PgUsername => self.username = Some(value),
            FocusField::PgPassword => self.password = Some(value),
            FocusField::PgSsl => self.use_ssl = matches!(value.as_str(), "true" | "1"),
            FocusField::PgDbName => self.db_name = Some(value),
            _ => {},
        }
    }

    /// Check if a focus field belongs to this config
    pub fn contains_field(field: super::FocusField) -> bool {
        use super::FocusField;
        matches!(field, 
            FocusField::PgHost | 
            FocusField::PgPort | 
            FocusField::PgUsername |
            FocusField::PgPassword | 
            FocusField::PgSsl |
            FocusField::PgDbName
        )
    }
}


pub struct PostgresRestoreSettings {
    // Restore Checkboxes
    pub no_data: bool,
    pub no_schema: bool,
    pub no_owner: bool,
    pub no_comments: bool,
}
/// Configuration for PostgreSQL connection
use anyhow::{anyhow, Result};
use log::info;
use tokio_postgres::Config as PgConfig;
use crate::postgres;
use crate::ui::models::PopupState;

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
    // Initialize with default values
    pub fn new() -> Self {
        // Load environment variables
        let env_pg_config = crate::config::load_postgres_config();

        // Set values from environment variables
        Self {
            host: env_pg_config.host,
            port: env_pg_config.port,
            username: env_pg_config.username,
            password: env_pg_config.password,
            use_ssl: env_pg_config.use_ssl,
            db_name: env_pg_config.db_name,
        }
    }

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

    /// Test PostgreSQL connection and return a client if successful
    pub async fn test_connection(&self, popup_state_setter: impl FnOnce(PopupState)) -> Result<Option<tokio_postgres::Client>> {
        // Validate PostgreSQL settings
        if self.host.is_none() || self.host.as_ref().unwrap().is_empty() {
            let error = "PostgreSQL host is required".to_string();
            popup_state_setter(PopupState::Error(error.clone()));
            return Err(anyhow!(error));
        }

        if self.port.is_none() {
            let error = "PostgreSQL port is required".to_string();
            popup_state_setter(PopupState::Error(error.clone()));
            return Err(anyhow!(error));
        }

        if self.username.is_none() || self.username.as_ref().unwrap().is_empty() {
            let error = "PostgreSQL username is required".to_string();
            popup_state_setter(PopupState::Error(error.clone()));
            return Err(anyhow!(error));
        }

        // Construct connection string
        let mut config = PgConfig::new();
        config.host(self.host.as_ref().unwrap());
        config.port(self.port.unwrap());
        config.user(self.username.as_ref().unwrap());
        config.password(&self.password.as_ref().unwrap_or(&String::new()));

        let result = if self.use_ssl {
            postgres::connect_ssl(&config, false, None).await
        } else {
            postgres::connect_no_ssl(&config).await
        };

        match result {
            Ok(client) => {
                info!("Successfully connected to PostgreSQL");
                popup_state_setter(PopupState::TestPgResult(format!("Successfully connected to PostgreSQL\nConnection string: {:?}", config)));
                Ok(Some(client))
            },
            Err(e) => {
                let error_msg = format!("Failed to connect to PostgreSQL: {}", e);
                popup_state_setter(PopupState::Error(error_msg.clone()));
                Err(anyhow!(error_msg))
            }
        }
    }
}


pub struct PostgresRestoreSettings {
    // Restore Checkboxes
    pub no_data: bool,
    pub no_schema: bool,
    pub no_owner: bool,
    pub no_comments: bool,
}
use log::debug;

/// Configuration for Qdrant restore target
#[derive(Clone, Debug, Default)]
pub struct QdrantConfig {
    pub host: Option<String>,
    pub collection: Option<String>,
    pub api_key: Option<String>,
}

impl QdrantConfig {
    /// Get all focus fields for Qdrant settings
    pub fn focus_fields() -> &'static [super::FocusField] {
        debug!("Getting focus fields for Qdrant settings");
        use super::FocusField;
        &[
            FocusField::EsHost,     // Reusing EsHost for Qdrant host
            FocusField::EsIndex,    // Reusing EsIndex for collection
            FocusField::QdrantApiKey,
        ]
    }

    /// Get the field value for a given focus field
    pub fn get_field_value(&self, field: super::FocusField) -> String {
        debug!("Getting field value for Qdrant field: {:?}", field);
        use super::FocusField;
        let result = match field {
            FocusField::EsHost => self.host.clone().unwrap_or_default(),
            FocusField::EsIndex => self.collection.clone().unwrap_or_default(),
            FocusField::QdrantApiKey => self.api_key.clone().unwrap_or_default(),
            _ => String::new(),
        };
        // Mask sensitive information in logs
        let log_value = if field == FocusField::QdrantApiKey && !result.is_empty() {
            "[MASKED]".to_string()
        } else {
            result.clone()
        };
        debug!("Retrieved value: {}", log_value);
        result
    }

    /// Set a field value from a string
    pub fn set_field_value(&mut self, field: super::FocusField, value: String) {
        debug!("Setting field value for Qdrant field: {:?}", field);
        use super::FocusField;
        match field {
            FocusField::EsHost => {
                debug!("Setting Qdrant host to: {}", value);
                self.host = Some(value);
            },
            FocusField::EsIndex => {
                debug!("Setting Qdrant collection to: {}", value);
                self.collection = Some(value);
            },
            FocusField::QdrantApiKey => {
                debug!("Setting Qdrant API key to: [MASKED]");
                self.api_key = Some(value);
            },
            _ => {
                debug!("Ignoring attempt to set unrelated field: {:?}", field);
            },
        }
    }

    /// Check if a focus field belongs to this config
    pub fn contains_field(field: super::FocusField) -> bool {
        debug!("Checking if field {:?} belongs to Qdrant config", field);
        use super::FocusField;
        let result = matches!(field, 
            FocusField::EsHost | 
            FocusField::EsIndex |
            FocusField::QdrantApiKey
        );
        debug!("Field {:?} belongs to Qdrant config: {}", field, result);
        result
    }
}

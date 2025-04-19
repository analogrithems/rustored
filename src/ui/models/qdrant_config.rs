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
        use super::FocusField;
        &[
            FocusField::EsHost,     // Reusing EsHost for Qdrant host
            FocusField::EsIndex,    // Reusing EsIndex for collection
            FocusField::QdrantApiKey,
        ]
    }

    /// Get the field value for a given focus field
    pub fn get_field_value(&self, field: super::FocusField) -> String {
        use super::FocusField;
        match field {
            FocusField::EsHost => self.host.clone().unwrap_or_default(),
            FocusField::EsIndex => self.collection.clone().unwrap_or_default(),
            FocusField::QdrantApiKey => self.api_key.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    /// Set a field value from a string
    pub fn set_field_value(&mut self, field: super::FocusField, value: String) {
        use super::FocusField;
        match field {
            FocusField::EsHost => self.host = Some(value),
            FocusField::EsIndex => self.collection = Some(value),
            FocusField::QdrantApiKey => self.api_key = Some(value),
            _ => {},
        }
    }

    /// Check if a focus field belongs to this config
    pub fn contains_field(field: super::FocusField) -> bool {
        use super::FocusField;
        matches!(field, 
            FocusField::EsHost | 
            FocusField::EsIndex |
            FocusField::QdrantApiKey
        )
    }
}

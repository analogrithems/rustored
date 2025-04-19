/// Configuration for Elasticsearch restore target
#[derive(Clone, Debug, Default)]
pub struct ElasticsearchConfig {
    pub host: Option<String>,
    pub index: Option<String>,
}

impl ElasticsearchConfig {
    /// Get all focus fields for Elasticsearch settings
    pub fn focus_fields() -> &'static [super::FocusField] {
        use super::FocusField;
        &[
            FocusField::EsHost,
            FocusField::EsIndex,
        ]
    }

    /// Get the field value for a given focus field
    pub fn get_field_value(&self, field: super::FocusField) -> String {
        use super::FocusField;
        match field {
            FocusField::EsHost => self.host.clone().unwrap_or_default(),
            FocusField::EsIndex => self.index.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    /// Set a field value from a string
    pub fn set_field_value(&mut self, field: super::FocusField, value: String) {
        use super::FocusField;
        match field {
            FocusField::EsHost => self.host = Some(value),
            FocusField::EsIndex => self.index = Some(value),
            _ => {},
        }
    }

    /// Check if a focus field belongs to this config
    pub fn contains_field(field: super::FocusField) -> bool {
        use super::FocusField;
        matches!(field, 
            FocusField::EsHost | 
            FocusField::EsIndex
        )
    }
}

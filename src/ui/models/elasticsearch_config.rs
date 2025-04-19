use log::debug;

/// Configuration for Elasticsearch restore target
#[derive(Clone, Debug, Default)]
pub struct ElasticsearchConfig {
    pub host: Option<String>,
    pub index: Option<String>,
}

impl ElasticsearchConfig {
    /// Get all focus fields for Elasticsearch settings
    pub fn focus_fields() -> &'static [super::FocusField] {
        debug!("Getting focus fields for Elasticsearch settings");
        use super::FocusField;
        &[
            FocusField::EsHost,
            FocusField::EsIndex,
        ]
    }

    /// Get the field value for a given focus field
    pub fn get_field_value(&self, field: super::FocusField) -> String {
        debug!("Getting field value for Elasticsearch field: {:?}", field);
        use super::FocusField;
        let result = match field {
            FocusField::EsHost => self.host.clone().unwrap_or_default(),
            FocusField::EsIndex => self.index.clone().unwrap_or_default(),
            _ => String::new(),
        };
        debug!("Retrieved value: {}", if field.to_string().contains("password") { "[MASKED]" } else { &result });
        result
    }

    /// Set a field value from a string
    pub fn set_field_value(&mut self, field: super::FocusField, value: String) {
        debug!("Setting field value for Elasticsearch field: {:?}", field);
        use super::FocusField;
        match field {
            FocusField::EsHost => {
                debug!("Setting Elasticsearch host to: {}", value);
                self.host = Some(value);
            },
            FocusField::EsIndex => {
                debug!("Setting Elasticsearch index to: {}", value);
                self.index = Some(value);
            },
            _ => {
                debug!("Ignoring attempt to set unrelated field: {:?}", field);
            },
        }
    }

    /// Check if a focus field belongs to this config
    pub fn contains_field(field: super::FocusField) -> bool {
        debug!("Checking if field {:?} belongs to Elasticsearch config", field);
        use super::FocusField;
        let result = matches!(field, 
            FocusField::EsHost | 
            FocusField::EsIndex
        );
        debug!("Field {:?} belongs to Elasticsearch config: {}", field, result);
        result
    }
}

/// Configuration for S3 connection
#[derive(Clone, Debug)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub prefix: String,
    pub endpoint_url: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub path_style: bool,
    pub error_message: Option<String>,
}

impl S3Config {
    /// Get all focus fields for S3 settings
    pub fn focus_fields() -> &'static [super::FocusField] {
        use super::FocusField;
        &[
            FocusField::Bucket,
            FocusField::Region,
            FocusField::Prefix,
            FocusField::EndpointUrl,
            FocusField::AccessKeyId,
            FocusField::SecretAccessKey,
            FocusField::PathStyle,
        ]
    }

    /// Get the field value for a given focus field
    pub fn get_field_value(&self, field: super::FocusField) -> String {
        use super::FocusField;
        match field {
            FocusField::Bucket => self.bucket.clone(),
            FocusField::Region => self.region.clone(),
            FocusField::Prefix => self.prefix.clone(),
            FocusField::EndpointUrl => self.endpoint_url.clone(),
            FocusField::AccessKeyId => self.access_key_id.clone(),
            FocusField::SecretAccessKey => self.secret_access_key.clone(),
            FocusField::PathStyle => self.path_style.to_string(),
            _ => String::new(),
        }
    }

    /// Set a field value from a string
    pub fn set_field_value(&mut self, field: super::FocusField, value: String) {
        use super::FocusField;
        match field {
            FocusField::Bucket => self.bucket = value,
            FocusField::Region => self.region = value,
            FocusField::Prefix => self.prefix = value,
            FocusField::EndpointUrl => self.endpoint_url = value,
            FocusField::AccessKeyId => self.access_key_id = value,
            FocusField::SecretAccessKey => self.secret_access_key = value,
            FocusField::PathStyle => self.path_style = value.parse().unwrap_or(false),
            _ => {},
        }
    }

    /// Check if a focus field belongs to this config
    pub fn contains_field(field: super::FocusField) -> bool {
        use super::FocusField;
        matches!(field, 
            FocusField::Bucket | 
            FocusField::Region | 
            FocusField::Prefix |
            FocusField::EndpointUrl | 
            FocusField::AccessKeyId |
            FocusField::SecretAccessKey | 
            FocusField::PathStyle
        )
    }

    pub fn mask_secret(&self, secret: &str) -> String {
        if secret.len() <= 4 {
            return "*".repeat(secret.len());
        }
        let visible_chars = 4;
        let hidden_chars = secret.len() - visible_chars;
        format!("{}{}", "*".repeat(hidden_chars), &secret[hidden_chars..])
    }

    pub fn masked_access_key(&self) -> String {
        self.mask_secret(&self.access_key_id)
    }

    pub fn masked_secret_key(&self) -> String {
        self.mask_secret(&self.secret_access_key)
    }
}

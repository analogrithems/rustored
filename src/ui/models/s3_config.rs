/// Configuration for S3 connection
use anyhow::{anyhow, Result};
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::Credentials;
use crate::ui::models::PopupState;

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
    
    /// Verify S3 settings are valid
    pub fn verify_settings(&self) -> Result<()> {
        if self.bucket.is_empty() {
            return Err(anyhow!("Bucket name is required"));
        }

        if self.region.is_empty() {
            return Err(anyhow!("Region is required"));
        }

        if self.endpoint_url.is_empty() {
            return Err(anyhow!("Endpoint URL is required"));
        }

        if self.access_key_id.is_empty() {
            return Err(anyhow!("Access Key ID is required"));
        }

        if self.secret_access_key.is_empty() {
            return Err(anyhow!("Secret Access Key is required"));
        }

        Ok(())
    }
    
    /// Initialize S3 client with current settings
    pub fn create_client(&self) -> Result<S3Client> {
        self.verify_settings()?;
        
        let credentials = Credentials::new(
            &self.access_key_id,
            &self.secret_access_key,
            None, None, "rustored"
        );

        let mut config_builder = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .region(aws_sdk_s3::config::Region::new(self.region.clone()));

        if !self.endpoint_url.is_empty() {
            let endpoint_url = if !self.endpoint_url.starts_with("http") {
                format!("http://{}", self.endpoint_url)
            } else {
                self.endpoint_url.clone()
            };

            config_builder = config_builder.endpoint_url(endpoint_url);
        }

        if self.path_style {
            config_builder = config_builder.force_path_style(true);
        }

        // Add behavior version which is required by AWS SDK
        config_builder = config_builder.behavior_version(aws_sdk_s3::config::BehaviorVersion::latest());

        let config = config_builder.build();
        Ok(S3Client::from_conf(config))
    }
    
    /// Test S3 connection and return success or error
    pub async fn test_connection(&self, popup_state_setter: impl FnOnce(PopupState)) -> Result<()> {
        let client = match self.create_client() {
            Ok(client) => client,
            Err(e) => {
                let error_msg = format!("Failed to initialize S3 client: {}", e);
                popup_state_setter(PopupState::Error(error_msg.clone()));
                return Err(anyhow!(error_msg));
            }
        };
        
        match client.list_buckets().send().await {
            Ok(resp) => {
                let buckets = resp.buckets();
                let bucket_names: Vec<String> = buckets
                    .iter()
                    .filter_map(|b| b.name().map(|s| s.to_string()))
                    .collect();

                let result = format!("Successfully connected to S3!\nAvailable buckets: {}",
                    if bucket_names.is_empty() { "None".to_string() } else { bucket_names.join(", ") });
                popup_state_setter(PopupState::TestS3Result(result));
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Failed to connect to S3: {}", e);
                popup_state_setter(PopupState::Error(error_msg.clone()));
                Err(anyhow!(error_msg))
            }
        }
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

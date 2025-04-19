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
    pub test_s3_button: bool,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: String::from("my-bucket"),
            region: String::from("us-west-2"),
            prefix: String::new(),
            endpoint_url: String::new(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            path_style: false,
            error_message: None,
            test_s3_button: false,
        }
    }
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

        // Endpoint URL is optional for AWS S3
        // For testing purposes, allow anonymous access
        // In a real application, you'd want to validate credentials

        // Skip credential checks for now to make development easier
        // We'll use anonymous credentials if none are provided

        Ok(())
    }

    /// Initialize S3 client with current settings
    pub fn create_client(&self) -> Result<S3Client> {
        self.verify_settings()?;

        let mut config_builder = aws_sdk_s3::config::Builder::new()
            .region(aws_sdk_s3::config::Region::new(self.region.clone()));

        // Only add credentials if they are provided
        if !self.access_key_id.is_empty() && !self.secret_access_key.is_empty() {
            let credentials = Credentials::new(
                &self.access_key_id,
                &self.secret_access_key,
                None, None, "rustored"
            );
            config_builder = config_builder.credentials_provider(credentials);
        }

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
        if secret.is_empty() {
            return String::new();
        }
        if secret.len() <= 4 {
            return "*".repeat(secret.len());
        }
        // Show only first 4 characters, then mask the rest
        let visible_chars = 4;
        let hidden_chars = secret.len() - visible_chars;
        format!("{}{}", &secret[..visible_chars], "*".repeat(hidden_chars))
    }

    pub fn masked_access_key(&self) -> String {
        self.mask_secret(&self.access_key_id)
    }

    pub fn masked_secret_key(&self) -> String {
        if self.secret_access_key.is_empty() {
            return String::new();
        }
        // Fully mask every character with asterisks
        "*".repeat(self.secret_access_key.len())
    }
    
    /// Get the display text for the secret access key field
    /// Only shows the actual value when in edit mode
    pub fn get_secret_key_display(&self, is_editing: bool, input_buffer: &str) -> String {
        if is_editing {
            format!("Secret Access Key: {}", input_buffer)
        } else {
            String::from("Secret Access Key: [hidden]")
        }
    }
}

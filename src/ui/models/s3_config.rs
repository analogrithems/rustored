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

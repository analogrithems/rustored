use serde::{Deserialize, Serialize};
use std::{error::Error, fs};
use dirs;
use toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct S3Config {
    pub bucket: String,
    pub prefix: Option<String>,
    pub region: Option<String>,
    #[serde(skip_serializing)]
    pub access_key_id: String,
    #[serde(skip_serializing)]
    pub secret_access_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DataStoreConfig {
    Postgres { connection_string: String, tls: Option<TlsConfig> },
    ElasticSearch { url: String, username: Option<String>, password: Option<String>, tls: Option<TlsConfig> },
    Qdrant { url: String, api_key: Option<String>, tls: Option<TlsConfig> },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TlsConfig {
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawConfig {
    pub s3: S3Config,
    pub datastore: DataStoreConfig,
}

impl S3Config {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path = config_path()?;
        let content = fs::read_to_string(path)?;
        let raw: RawConfig = toml::from_str(&content)?;
        Ok(raw.s3)
    }
}

impl DataStoreConfig {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path = config_path()?;
        let content = fs::read_to_string(path)?;
        let raw: RawConfig = toml::from_str(&content)?;
        Ok(raw.datastore)
    }
}

fn config_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let mut dir = dirs::config_dir().ok_or("Cannot find config directory")?;
    dir.push("rustored");
    fs::create_dir_all(&dir)?;
    dir.push("config.toml");
    Ok(dir)
}

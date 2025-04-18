use serde::{Deserialize, Serialize};
use std::{error::Error, fs};
use dirs;
use toml;
use tokio_postgres::{NoTls};
use elasticsearch::{Elasticsearch, http::transport::Transport, PingParts};
use qdrant_client::prelude::QdrantClient;
use anyhow::Result;

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

    /// Test connectivity to the configured datastore.
    pub async fn test_connection(&self) -> Result<(), Box<dyn Error>> {
        match self {
            DataStoreConfig::Postgres { connection_string, tls: _ } => {
                // connect without TLS or implement TLS support later
                let (client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
                // spawn connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("Postgres connection error: {}", e);
                    }
                });
                Ok(())
            }
            DataStoreConfig::ElasticSearch { url, username: _, password: _, tls: _ } => {
                let transport = Transport::single_node(url)?;
                let client = Elasticsearch::new(transport);
                let res = client.ping(PingParts::None).send().await?;
                res.error_for_status_ref()?;
                Ok(())
            }
            DataStoreConfig::Qdrant { url, api_key: _, tls: _ } => {
                let mut builder = QdrantClient::new(&url);
                // health check if available
                builder.health().await?;
                Ok(())
            }
        }
    }
}

fn config_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let mut dir = dirs::config_dir().ok_or("Cannot find config directory")?;
    dir.push("rustored");
    fs::create_dir_all(&dir)?;
    dir.push("config.toml");
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn postgres_connection_invalid() {
        let cfg = DataStoreConfig::Postgres { connection_string: "postgres://invalid".to_string(), tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid Postgres URL");
    }

    #[tokio::test]
    async fn elasticsearch_connection_invalid() {
        let cfg = DataStoreConfig::ElasticSearch { url: "http://invalid:9200".to_string(), username: None, password: None, tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid ElasticSearch URL");
    }

    #[tokio::test]
    async fn qdrant_connection_invalid() {
        let cfg = DataStoreConfig::Qdrant { url: "http://invalid:6333".to_string(), api_key: None, tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid Qdrant URL");
    }
}

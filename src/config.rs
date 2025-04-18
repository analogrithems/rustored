use std::env;
use dotenvy::dotenv;
use tokio_postgres::NoTls;
use elasticsearch::{Elasticsearch, http::transport::Transport};
use reqwest;
use anyhow::{Result, Context, bail};

#[allow(dead_code)]
#[derive(Debug)]
// suppress unused credential fields until used
pub struct S3Config {
    pub bucket: String,
    pub prefix: Option<String>,
    pub region: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl S3Config {
    /// Load S3 configuration from environment (supports .env)
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        let bucket = env::var("S3_BUCKET").context("Missing S3_BUCKET")?;
        let prefix = env::var("S3_PREFIX").ok();
        let region = env::var("S3_REGION").ok();
        let access_key_id = env::var("S3_ACCESS_KEY_ID").context("Missing S3_ACCESS_KEY_ID")?;
        let secret_access_key = env::var("S3_SECRET_ACCESS_KEY").context("Missing S3_SECRET_ACCESS_KEY")?;
        Ok(S3Config { bucket, prefix, region, access_key_id, secret_access_key })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
// suppress unused tls fields until implemented
pub enum DataStoreConfig {
    Postgres { connection_string: String, tls: Option<TlsConfig> },
    ElasticSearch { url: String, username: Option<String>, password: Option<String>, tls: Option<TlsConfig> },
    Qdrant { url: String, api_key: Option<String>, tls: Option<TlsConfig> },
}

#[allow(dead_code)]
#[derive(Debug)]
// suppress until TLS support implemented
pub struct TlsConfig {
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
}

impl DataStoreConfig {
    /// Load DataStore configuration from env (supports .env)
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        let ds_type = env::var("DS_TYPE").context("Missing DS_TYPE")?.trim().to_lowercase();
        match ds_type.as_str() {
            "postgres" => {
                let conn = env::var("DS_POSTGRES_CONN").context("Missing DS_POSTGRES_CONN")?;
                Ok(DataStoreConfig::Postgres { connection_string: conn, tls: None })
            }
            "elasticsearch" => {
                let url = env::var("DS_ES_URL").context("Missing DS_ES_URL")?;
                let user = env::var("DS_ES_USER").ok();
                let pass = env::var("DS_ES_PASS").ok();
                Ok(DataStoreConfig::ElasticSearch { url, username: user, password: pass, tls: None })
            }
            "qdrant" => {
                let url = env::var("DS_QDRANT_URL").context("Missing DS_QDRANT_URL")?;
                let api = env::var("DS_QDRANT_API").ok();
                Ok(DataStoreConfig::Qdrant { url, api_key: api, tls: None })
            }
            _ => bail!("Unsupported DS_TYPE"),
        }
    }

    /// Test connectivity to the configured datastore.
    pub async fn test_connection(&self) -> Result<()> {
        match self {
            DataStoreConfig::Postgres { connection_string, tls: _ } => {
                // connect without TLS or implement TLS support later
                let (_client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
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
                let res = client.ping().send().await?;
                if !res.status_code().is_success() {
                    bail!("Elasticsearch ping failed");
                }
                Ok(())
            }
            DataStoreConfig::Qdrant { url, api_key: _, tls: _ } => {
                let health_url = format!("{}/health", url);
                let resp = reqwest::get(&health_url).await?;
                if !resp.status().is_success() {
                    bail!("Qdrant health check failed");
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn postgres_connection_invalid() {
        let cfg = DataStoreConfig::Postgres { connection_string: "postgres://invalid".to_string(), tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid Postgres URL");
    }

    #[tokio::test]
    #[serial]
    async fn elasticsearch_connection_invalid() {
        let cfg = DataStoreConfig::ElasticSearch { url: "http://invalid:9200".to_string(), username: None, password: None, tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid ElasticSearch URL");
    }

    #[tokio::test]
    #[serial]
    async fn qdrant_connection_invalid() {
        let cfg = DataStoreConfig::Qdrant { url: "http://invalid:6333".to_string(), api_key: None, tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid Qdrant URL");
    }

    #[test]
    #[serial]
    fn s3_from_env() {
        unsafe {
            env::set_var("S3_BUCKET", "b");
            env::set_var("S3_PREFIX", "p");
            env::set_var("S3_REGION", "r");
            env::set_var("S3_ACCESS_KEY_ID", "id");
            env::set_var("S3_SECRET_ACCESS_KEY", "key");
        }
        let cfg = S3Config::from_env().unwrap();
        assert_eq!(cfg.bucket, "b");
        assert_eq!(cfg.prefix.as_deref(), Some("p"));
        assert_eq!(cfg.region.as_deref(), Some("r"));
    }

    #[test]
    #[serial]
    fn ds_from_env_postgres() {
        unsafe {
            env::set_var("DS_TYPE", "postgres");
            env::set_var("DS_POSTGRES_CONN", "cstr");
        }
        let cfg = DataStoreConfig::from_env().unwrap();
        if let DataStoreConfig::Postgres { connection_string, .. } = cfg {
            assert_eq!(connection_string, "cstr");
        } else { panic!("Expected Postgres"); }
    }

    #[test]
    #[serial]
    fn ds_from_env_es() {
        unsafe {
            env::set_var("DS_TYPE", "elasticsearch");
            env::set_var("DS_ES_URL", "u");
        }
        let cfg = DataStoreConfig::from_env().unwrap();
        if let DataStoreConfig::ElasticSearch { url, .. } = cfg {
            assert_eq!(url, "u");
        } else { panic!("Expected ES"); }
    }

    #[test]
    #[serial]
    fn ds_from_env_qdrant() {
        unsafe {
            env::set_var("DS_TYPE", "qdrant");
            env::set_var("DS_QDRANT_URL", "u");
        }
        let cfg = DataStoreConfig::from_env().unwrap();
        if let DataStoreConfig::Qdrant { url, .. } = cfg {
            assert_eq!(url, "u");
        } else { panic!("Expected Qdrant"); }
    }
}

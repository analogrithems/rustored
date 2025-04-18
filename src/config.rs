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
    /// Whether to use path-style addressing
    pub path_style: Option<bool>,
    pub endpoint_url: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
}

impl S3Config {
    /// Load S3 configuration from environment (supports .env)
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        let bucket = env::var("S3_BUCKET").context("rustored::config::S3Config::from_env: Missing S3_BUCKET")?;
        let prefix = env::var("S3_PREFIX").ok();
        let region = env::var("S3_REGION").ok();
        let path_style = env::var("S3_PATH_STYLE").ok().and_then(|v| v.parse::<bool>().ok());
        let endpoint_url = env::var("S3_ENDPOINT_URL").ok();
        let access_key_id = env::var("S3_ACCESS_KEY_ID").context("rustored::config::S3Config::from_env: Missing S3_ACCESS_KEY_ID")?;
        let secret_access_key = env::var("S3_SECRET_ACCESS_KEY").context("rustored::config::S3Config::from_env: Missing S3_SECRET_ACCESS_KEY")?;
        Ok(S3Config { bucket, prefix, region, path_style, endpoint_url, access_key_id, secret_access_key })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
// suppress unused tls fields until implemented
pub enum DataStoreConfig {
    Postgres { connection_string: Option<String>, tls: Option<TlsConfig> },
    ElasticSearch { url: Option<String>, username: Option<String>, password: Option<String>, tls: Option<TlsConfig> },
    Qdrant { url: Option<String>, api_key: Option<String>, tls: Option<TlsConfig> },
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
        let ds_type = env::var("DS_TYPE").context("rustored::config::DataStoreConfig::from_env: Missing DS_TYPE")?.trim().to_lowercase();
        match ds_type.as_str() {
            "postgres" => {
                let conn = env::var("DS_POSTGRES_CONN").ok();
                Ok(DataStoreConfig::Postgres { connection_string: conn, tls: None })
            }
            "elasticsearch" => {
                let url = env::var("DS_ES_URL").ok();
                let user = env::var("DS_ES_USER").ok();
                let pass = env::var("DS_ES_PASS").ok();
                Ok(DataStoreConfig::ElasticSearch { url, username: user, password: pass, tls: None })
            }
            "qdrant" => {
                let url = env::var("DS_QDRANT_URL").ok();
                let api = env::var("DS_QDRANT_API").ok();
                Ok(DataStoreConfig::Qdrant { url, api_key: api, tls: None })
            }
            _ => bail!("rustored::config::DataStoreConfig::from_env: Unsupported DS_TYPE"),
        }
    }

    /// Test connectivity to the configured datastore.
    pub async fn test_connection(&self) -> Result<()> {
        match self {
            DataStoreConfig::Postgres { connection_string, tls: _ } => {
                let conn_str = connection_string.as_ref().context("rustored::config::DataStoreConfig::test_connection: Missing DS_POSTGRES_CONN")?;
                // connect without TLS or implement TLS support later
                let (_client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;
                // spawn connection handler
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("rustored::config::DataStoreConfig::test_connection: Postgres connection error: {}", e);
                    }
                });
                Ok(())
            }
            DataStoreConfig::ElasticSearch { url, username: _, password: _, tls: _ } => {
                let endpoint = url.as_ref().context("rustored::config::DataStoreConfig::test_connection: Missing DS_ES_URL")?;
                let transport = Transport::single_node(endpoint)?;
                let client = Elasticsearch::new(transport);
                let res = client.ping().send().await?;
                if !res.status_code().is_success() {
                    bail!("rustored::config::DataStoreConfig::test_connection: Elasticsearch ping failed");
                }
                Ok(())
            }
            DataStoreConfig::Qdrant { url, api_key: _, tls: _ } => {
                let endpoint = url.as_ref().context("rustored::config::DataStoreConfig::test_connection: Missing DS_QDRANT_URL")?;
                let health_url = format!("{}/health", endpoint);
                let resp = reqwest::get(&health_url).await?;
                if !resp.status().is_success() {
                    bail!("rustored::config::DataStoreConfig::test_connection: Qdrant health check failed");
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
        let cfg = DataStoreConfig::Postgres { connection_string: Some("postgres://invalid".to_string()), tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid Postgres URL");
    }

    #[tokio::test]
    #[serial]
    async fn elasticsearch_connection_invalid() {
        let cfg = DataStoreConfig::ElasticSearch { url: Some("http://invalid:9200".to_string()), username: None, password: None, tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid ElasticSearch URL");
    }

    #[tokio::test]
    #[serial]
    async fn qdrant_connection_invalid() {
        let cfg = DataStoreConfig::Qdrant { url: Some("http://invalid:6333".to_string()), api_key: None, tls: None };
        assert!(cfg.test_connection().await.is_err(), "Expected error for invalid Qdrant URL");
    }

    #[test]
    #[serial]
    fn s3_from_env() {
        unsafe {
            env::set_var("S3_BUCKET", "org-snapshots");
            env::set_var("S3_PREFIX", "backups");
            env::set_var("S3_REGION", "us-east-1");
            env::set_var("S3_ENDPOINT_URL", "https://s3.example.com");
            // leave S3_PATH_STYLE unset for default
            env::set_var("S3_ACCESS_KEY_ID", "id");
            env::set_var("S3_SECRET_ACCESS_KEY", "key");
        }
        let cfg = S3Config::from_env().unwrap();
        assert_eq!(cfg.bucket, "org-snapshots");
        assert_eq!(cfg.prefix.as_deref(), Some("backups"));
        assert_eq!(cfg.region.as_deref(), Some("us-east-1"));
        assert_eq!(cfg.path_style, None);
        assert_eq!(cfg.endpoint_url.as_deref(), Some("https://s3.example.com"));
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
            assert_eq!(connection_string.as_deref(), Some("cstr"));
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
            assert_eq!(url.as_deref(), Some("u"));
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
            assert_eq!(url.as_deref(), Some("u"));
        } else { panic!("Expected Qdrant"); }
    }
}

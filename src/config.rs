use tokio_postgres::NoTls;
use elasticsearch::{Elasticsearch, http::transport::Transport};
use reqwest;
use anyhow::{Result, Context, bail};
use crate::cli::Opt;

#[allow(dead_code)]
#[derive(Debug)]
// suppress unused credential fields until used
pub struct S3Config {
    pub bucket: Option<String>,
    pub prefix: Option<String>,
    pub region: Option<String>,
    /// Whether to use path-style addressing
    pub path_style: Option<bool>,
    pub endpoint_url: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

impl S3Config {
    /// Build S3Config directly from CLI options
    pub fn from_opt(opt: &Opt) -> Self {
        S3Config {
            bucket: opt.bucket.clone(),
            prefix: opt.prefix.clone(),
            region: opt.region.clone(),
            path_style: opt.path_style,
            endpoint_url: opt.endpoint_url.clone(),
            access_key_id: opt.access_key_id.clone(),
            secret_access_key: opt.secret_access_key.clone(),
        }
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
    /// Build DataStoreConfig directly from CLI options
    pub fn from_opt(opt: &Opt) -> Self {
        // determine datastore type or default to postgres
        let dtype = opt.ds_type
            .as_deref()
            .unwrap_or("postgres")
            .trim()
            .to_lowercase();
        match dtype.as_str() {
            "postgres" => DataStoreConfig::Postgres { connection_string: opt.ds_postgres_conn.clone(), tls: None },
            "elasticsearch" => DataStoreConfig::ElasticSearch { url: opt.ds_es_url.clone(), username: opt.ds_es_user.clone(), password: opt.ds_es_pass.clone(), tls: None },
            "qdrant" => DataStoreConfig::Qdrant { url: opt.ds_qdrant_url.clone(), api_key: opt.ds_qdrant_api.clone(), tls: None },
            _ => DataStoreConfig::Postgres { connection_string: opt.ds_postgres_conn.clone(), tls: None },
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
    use crate::cli::Opt;
    use serial_test::serial;
    use clap::Parser;

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
    fn s3_from_opt() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "org-snapshots", "--s3-prefix", "backups", "--s3-region", "us-east-1", "--s3-endpoint-url", "https://s3.example.com", "--s3-access-key-id", "id", "--s3-secret-access-key", "key"]);
        let cfg = S3Config::from_opt(&opt);
        assert_eq!(cfg.bucket.as_deref(), Some("org-snapshots"));
        assert_eq!(cfg.prefix.as_deref(), Some("backups"));
        assert_eq!(cfg.region.as_deref(), Some("us-east-1"));
        assert_eq!(cfg.path_style, Some(true));
        assert_eq!(cfg.endpoint_url.as_deref(), Some("https://s3.example.com"));
        assert_eq!(cfg.access_key_id.as_deref(), Some("id"));
        assert_eq!(cfg.secret_access_key.as_deref(), Some("key"));
    }

    #[test]
    fn ds_from_opt_postgres() {
        let opt = Opt::parse_from(&["rustored", "--ds-type", "postgres", "--ds-postgres-conn", "connstr"]);
        let ds = DataStoreConfig::from_opt(&opt);
        if let DataStoreConfig::Postgres { connection_string, .. } = ds {
            assert_eq!(connection_string.as_deref(), Some("connstr"));
        } else { panic!() }
    }

    #[test]
    fn ds_from_opt_elasticsearch() {
        let opt = Opt::parse_from(&["rustored", "--ds-type", "elasticsearch", "--ds-es-url", "u"]);
        let ds = DataStoreConfig::from_opt(&opt);
        if let DataStoreConfig::ElasticSearch { url, .. } = ds {
            assert_eq!(url.as_deref(), Some("u"));
        } else { panic!() }
    }

    #[test]
    fn ds_from_opt_qdrant() {
        let opt = Opt::parse_from(&["rustored", "--ds-type", "qdrant", "--ds-qdrant-url", "u"]);
        let ds = DataStoreConfig::from_opt(&opt);
        if let DataStoreConfig::Qdrant { url, .. } = ds {
            assert_eq!(url.as_deref(), Some("u"));
        } else { panic!() }
    }
}

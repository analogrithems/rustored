use crate::config::{S3Config, DataStoreConfig};
use std::error::Error;
use clap::Parser;

/// Command-line options for rustored
#[derive(Debug, Parser)]
#[command(name = "rustored", version)]
pub struct Opt {
    #[arg(long = "s3-bucket", env = "S3_BUCKET")]
    pub bucket: Option<String>,

    #[arg(long = "s3-prefix", env = "S3_PREFIX")]
    pub prefix: Option<String>,

    #[arg(long = "s3-region", env = "S3_REGION")]
    pub region: Option<String>,

    #[arg(long = "s3-endpoint-url", env = "S3_ENDPOINT_URL")]
    pub endpoint_url: Option<String>,

    #[arg(long = "s3-path-style", env = "S3_PATH_STYLE", default_value = "true")]
    pub path_style: Option<bool>,

    #[arg(long = "s3-access-key-id", env = "S3_ACCESS_KEY_ID")]
    pub access_key_id: Option<String>,

    #[arg(long = "s3-secret-access-key", env = "S3_SECRET_ACCESS_KEY")]
    pub secret_access_key: Option<String>,

    #[arg(long = "ds-type", env = "DS_TYPE")]
    pub ds_type: Option<String>,

    #[arg(long = "ds-postgres-conn", env = "DS_POSTGRES_CONN")]
    pub ds_postgres_conn: Option<String>,

    #[arg(long = "ds-es-url", env = "DS_ES_URL")]
    pub ds_es_url: Option<String>,

    #[arg(long = "ds-es-user", env = "DS_ES_USER")]
    pub ds_es_user: Option<String>,

    #[arg(long = "ds-es-pass", env = "DS_ES_PASS")]
    pub ds_es_pass: Option<String>,

    #[arg(long = "ds-qdrant-url", env = "DS_QDRANT_URL")]
    pub ds_qdrant_url: Option<String>,

    #[arg(long, env = "DS_QDRANT_API")]
    pub ds_qdrant_api: Option<String>,

    /// Path to splash image file
    #[arg(long, env = "SPLASH_IMAGE", default_value = "logo.png")]
    pub splash_image: String,
}

impl Opt {
    /// Convert CLI options into S3Config and DataStoreConfig
    pub fn into_configs(self) -> Result<(S3Config, DataStoreConfig), Box<dyn Error>> {
        // build configs directly from CLI options
        let s3 = S3Config::from_opt(&self);
        let ds = DataStoreConfig::from_opt(&self);
        Ok((s3, ds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::env;

    #[test]
    fn cli_env_defaults() {
        unsafe { env::remove_var("S3_BUCKET"); }
        // set only essential env
        unsafe { env::set_var("S3_BUCKET", "envb"); }
        unsafe { env::set_var("S3_ACCESS_KEY_ID", "id"); }
        unsafe { env::set_var("S3_SECRET_ACCESS_KEY", "key"); }
        unsafe { env::set_var("DS_TYPE", "postgres"); }
        unsafe { env::set_var("DS_POSTGRES_CONN", "connstr"); }
        let opt = Opt::parse_from(&["rustored"]);
        assert_eq!(opt.bucket, Some("envb".to_string()));
        // override via CLI
        let opt2 = Opt::parse_from(&["rustored", "--s3-bucket", "clib", "--s3-access-key-id", "id2", "--s3-secret-access-key", "key2", "--ds-type", "qdrant", "--ds-qdrant-url", "url"]);
        assert_eq!(opt2.bucket, Some("clib".to_string()));
        assert_eq!(opt2.ds_type, Some("qdrant".to_string()));
        assert_eq!(opt2.ds_qdrant_url, Some("url".to_string()));
    }

    #[test]
    fn into_configs_postgres() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "b", "--s3-access-key-id", "id", "--s3-secret-access-key", "key", "--ds-type", "postgres", "--ds-postgres-conn", "db"]);
        let (s3, ds) = opt.into_configs().unwrap();
        assert_eq!(s3.bucket, Some("b".to_string()));
        if let DataStoreConfig::Postgres { connection_string, .. } = ds {
            assert_eq!(connection_string.as_deref(), Some("db"));
        } else { panic!(); }
    }

    #[test]
    fn into_configs_es() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "b", "--s3-access-key-id", "id", "--s3-secret-access-key", "key", "--ds-type", "elasticsearch", "--ds-es-url", "u"]);
        let (_, ds) = opt.into_configs().unwrap();
        if let DataStoreConfig::ElasticSearch { url, .. } = ds {
            assert_eq!(url.as_deref(), Some("u"));
        } else { panic!(); }
    }

    #[test]
    fn into_configs_qdrant() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "b", "--s3-access-key-id", "id", "--s3-secret-access-key", "key", "--ds-type", "qdrant", "--ds-qdrant-url", "u"]);
        let (_, ds) = opt.into_configs().unwrap();
        if let DataStoreConfig::Qdrant { url, .. } = ds {
            assert_eq!(url.as_deref(), Some("u"));
        } else { panic!(); }
    }
}

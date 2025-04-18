use clap::Parser;
use crate::config::{S3Config, DataStoreConfig};
use std::error::Error;

/// Command-line options for rustored
#[derive(Debug, Parser)]
#[clap(name = "rustored", version)]
pub struct Opt {
    #[clap(long, env = "S3_BUCKET")]
    pub bucket: String,

    #[clap(long, env = "S3_PREFIX")]
    pub prefix: Option<String>,

    #[clap(long, env = "S3_REGION")]
    pub region: Option<String>,

    #[clap(long, env = "S3_ACCESS_KEY_ID")]
    pub access_key_id: String,

    #[clap(long, env = "S3_SECRET_ACCESS_KEY")]
    pub secret_access_key: String,

    #[clap(long, env = "DS_TYPE")]
    pub ds_type: String,

    #[clap(long, env = "DS_POSTGRES_CONN")]
    pub ds_postgres_conn: Option<String>,

    #[clap(long, env = "DS_ES_URL")]
    pub ds_es_url: Option<String>,

    #[clap(long, env = "DS_ES_USER")]
    pub ds_es_user: Option<String>,

    #[clap(long, env = "DS_ES_PASS")]
    pub ds_es_pass: Option<String>,

    #[clap(long, env = "DS_QDRANT_URL")]
    pub ds_qdrant_url: Option<String>,

    #[clap(long, env = "DS_QDRANT_API")]
    pub ds_qdrant_api: Option<String>,
}

impl Opt {
    /// Convert CLI options into S3Config and DataStoreConfig
    pub fn into_configs(self) -> Result<(S3Config, DataStoreConfig), Box<dyn Error>> {
        let s3 = S3Config {
            bucket: self.bucket,
            prefix: self.prefix.unwrap_or_default(),
            region: self.region.unwrap_or_default(),
            path_style: None,
            endpoint_url: None,
            access_key_id: self.access_key_id,
            secret_access_key: self.secret_access_key,
        };
        let ds = match self.ds_type.as_str() {
            "postgres" => DataStoreConfig::Postgres { connection_string: self.ds_postgres_conn.ok_or("Missing DS_POSTGRES_CONN")?, tls: None },
            "elasticsearch" => DataStoreConfig::ElasticSearch { url: self.ds_es_url.ok_or("Missing DS_ES_URL")?, username: self.ds_es_user, password: self.ds_es_pass, tls: None },
            "qdrant" => DataStoreConfig::Qdrant { url: self.ds_qdrant_url.ok_or("Missing DS_QDRANT_URL")?, api_key: self.ds_qdrant_api, tls: None },
            _ => return Err("Unsupported DS_TYPE".into()),
        };
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
        env::remove_var("S3_BUCKET");
        // set only essential env
        env::set_var("S3_BUCKET", "envb");
        env::set_var("S3_ACCESS_KEY_ID", "id");
        env::set_var("S3_SECRET_ACCESS_KEY", "key");
        env::set_var("DS_TYPE", "postgres");
        env::set_var("DS_POSTGRES_CONN", "connstr");
        let opt = Opt::parse_from(&["rustored"]);
        assert_eq!(opt.bucket, "envb");
        // override via CLI
        let opt2 = Opt::parse_from(&["rustored", "--s3-bucket", "clib", "--s3-access-key-id", "id2", "--s3-secret-access-key", "key2", "--ds-type", "qdrant", "--ds-qdrant-url", "url"]);
        assert_eq!(opt2.bucket, "clib");
        assert_eq!(opt2.ds_type, "qdrant");
        assert_eq!(opt2.ds_qdrant_url.unwrap(), "url");
    }

    #[test]
    fn into_configs_postgres() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "b", "--s3-access-key-id", "id", "--s3-secret-access-key", "key", "--ds-type", "postgres", "--ds-postgres-conn", "db"]);
        let (s3, ds) = opt.into_configs().unwrap();
        assert_eq!(s3.bucket, "b");
        if let DataStoreConfig::Postgres { connection_string, .. } = ds {
            assert_eq!(connection_string, "db");
        } else { panic!(); }
    }

    #[test]
    fn into_configs_es() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "b", "--s3-access-key-id", "id", "--s3-secret-access-key", "key", "--ds-type", "elasticsearch", "--ds-es-url", "u"]);
        let (_, ds) = opt.into_configs().unwrap();
        if let DataStoreConfig::ElasticSearch { url, .. } = ds {
            assert_eq!(url, "u");
        } else { panic!(); }
    }

    #[test]
    fn into_configs_qdrant() {
        let opt = Opt::parse_from(&["rustored", "--s3-bucket", "b", "--s3-access-key-id", "id", "--s3-secret-access-key", "key", "--ds-type", "qdrant", "--ds-qdrant-url", "u"]);
        let (_, ds) = opt.into_configs().unwrap();
        if let DataStoreConfig::Qdrant { url, .. } = ds {
            assert_eq!(url, "u");
        } else { panic!(); }
    }
}

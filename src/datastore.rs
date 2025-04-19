use anyhow::Result;

pub enum DatastoreRestoreTarget {
    Postgres,
    Elasticsearch {
        host: String,
        index: String,
    },
    Qdrant {
        host: String,
        collection: String,
        api_key: Option<String>,
    },
}

impl DatastoreRestoreTarget {
    pub async fn restore(&self, name: &str, input: &str) -> Result<()> {
        match self {
            DatastoreRestoreTarget::Postgres => {
                // Call existing postgres restore logic
                crate::backup::restore_database(name, input, "localhost", 5432, None, None, false)
            }
            DatastoreRestoreTarget::Elasticsearch { host, index } => {
                // TODO: Implement Elasticsearch restore logic
                println!("[STUB] Would restore {input} to Elasticsearch at {host}, index {index}");
                Ok(())
            }
            DatastoreRestoreTarget::Qdrant { host, collection, api_key: _ } => {
                // TODO: Implement Qdrant restore logic
                println!("[STUB] Would restore {input} to Qdrant at {host}, collection {collection}");
                Ok(())
            }
        }
    }
}

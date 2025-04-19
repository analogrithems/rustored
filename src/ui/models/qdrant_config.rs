/// Configuration for Qdrant restore target
#[derive(Clone, Debug, Default)]
pub struct QdrantConfig {
    pub host: Option<String>,
    pub collection: Option<String>,
    pub api_key: Option<String>,
}

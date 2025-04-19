/// Configuration for Elasticsearch restore target
#[derive(Clone, Debug, Default)]
pub struct ElasticsearchConfig {
    pub host: Option<String>,
    pub index: Option<String>,
}

// We need to use the crate name directly since we're in an integration test
use rustored::ui::models::{PopupState, PostgresConfig, ElasticsearchConfig, QdrantConfig, RestoreTarget};
use rustored::ui::restore_browser::RestoreBrowser;

#[test]
fn test_restore_browser_creation() {
    let restore_browser = RestoreBrowser::new(
        RestoreTarget::Postgres,
        Some(PostgresConfig::default()),
        Some(ElasticsearchConfig::default()),
        Some(QdrantConfig::default()),
    );
    
    assert_eq!(restore_browser.restore_target, RestoreTarget::Postgres);
    assert!(restore_browser.postgres_config.is_some());
    assert!(restore_browser.elasticsearch_config.is_some());
    assert!(restore_browser.qdrant_config.is_some());
    assert_eq!(restore_browser.popup_state, PopupState::Hidden);
}

#[test]
fn test_restore_browser_with_postgres_target() {
    let restore_browser = RestoreBrowser::new(
        RestoreTarget::Postgres,
        Some(PostgresConfig::default()),
        None,
        None,
    );
    
    assert_eq!(restore_browser.restore_target, RestoreTarget::Postgres);
    assert!(restore_browser.postgres_config.is_some());
    assert!(restore_browser.elasticsearch_config.is_none());
    assert!(restore_browser.qdrant_config.is_none());
}

#[test]
fn test_restore_browser_with_elasticsearch_target() {
    let restore_browser = RestoreBrowser::new(
        RestoreTarget::Elasticsearch,
        None,
        Some(ElasticsearchConfig::default()),
        None,
    );
    
    assert_eq!(restore_browser.restore_target, RestoreTarget::Elasticsearch);
    assert!(restore_browser.postgres_config.is_none());
    assert!(restore_browser.elasticsearch_config.is_some());
    assert!(restore_browser.qdrant_config.is_none());
}

#[test]
fn test_restore_browser_with_qdrant_target() {
    let restore_browser = RestoreBrowser::new(
        RestoreTarget::Qdrant,
        None,
        None,
        Some(QdrantConfig::default()),
    );
    
    assert_eq!(restore_browser.restore_target, RestoreTarget::Qdrant);
    assert!(restore_browser.postgres_config.is_none());
    assert!(restore_browser.elasticsearch_config.is_none());
    assert!(restore_browser.qdrant_config.is_some());
}

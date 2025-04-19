use insta::assert_debug_snapshot;

// We need to use the crate name directly since we're in an integration test
use rustored::ui::models::{BackupMetadata, FocusField, InputMode, PopupState, PostgresConfig, S3Config, ElasticsearchConfig, QdrantConfig, RestoreTarget};

// Helper function to create a test timestamp with a fixed value
fn create_test_aws_datetime() -> f64 {
    // Use a fixed timestamp for testing (2023-01-01 12:00:00 UTC)
    1672574400.0
}

#[test]
fn test_s3_config() {
    let s3_config = S3Config {
        bucket: "test-bucket".to_string(),
        region: "us-west-2".to_string(),
        prefix: "test-prefix".to_string(),
        endpoint_url: "https://test-endpoint.com".to_string(),
        access_key_id: "test-access-key".to_string(),
        secret_access_key: "test-secret-key".to_string(),
        path_style: false,
        error_message: None,
        test_s3_button: false,
    };

    assert_debug_snapshot!(s3_config);
}

#[test]
fn test_s3_config_focus_fields() {
    // Test that we can get all focus fields for S3 settings
    let fields = S3Config::focus_fields();
    
    // Verify we have the expected number of fields
    assert_eq!(fields.len(), 7);
    
    // Verify all expected fields are present
    assert!(fields.contains(&FocusField::Bucket));
    assert!(fields.contains(&FocusField::Region));
    assert!(fields.contains(&FocusField::Prefix));
    assert!(fields.contains(&FocusField::EndpointUrl));
    assert!(fields.contains(&FocusField::AccessKeyId));
    assert!(fields.contains(&FocusField::SecretAccessKey));
    assert!(fields.contains(&FocusField::PathStyle));
}

#[test]
fn test_s3_config_contains_field() {
    // Test that contains_field correctly identifies S3 fields
    assert!(S3Config::contains_field(FocusField::Bucket));
    assert!(S3Config::contains_field(FocusField::Region));
    assert!(S3Config::contains_field(FocusField::Prefix));
    assert!(S3Config::contains_field(FocusField::EndpointUrl));
    assert!(S3Config::contains_field(FocusField::AccessKeyId));
    assert!(S3Config::contains_field(FocusField::SecretAccessKey));
    assert!(S3Config::contains_field(FocusField::PathStyle));
    
    // Test that it correctly rejects non-S3 fields
    assert!(!S3Config::contains_field(FocusField::PgHost));
    assert!(!S3Config::contains_field(FocusField::SnapshotList));
    assert!(!S3Config::contains_field(FocusField::RestoreTarget));
}

#[test]
fn test_s3_config_get_field_value() {
    let s3_config = S3Config {
        bucket: "test-bucket".to_string(),
        region: "us-west-2".to_string(),
        prefix: "test-prefix".to_string(),
        endpoint_url: "https://test-endpoint.com".to_string(),
        access_key_id: "test-access-key".to_string(),
        secret_access_key: "test-secret-key".to_string(),
        path_style: true,
        error_message: None,
        test_s3_button: false,
    };
    
    // Test getting field values
    assert_eq!(s3_config.get_field_value(FocusField::Bucket), "test-bucket");
    assert_eq!(s3_config.get_field_value(FocusField::Region), "us-west-2");
    assert_eq!(s3_config.get_field_value(FocusField::Prefix), "test-prefix");
    assert_eq!(s3_config.get_field_value(FocusField::EndpointUrl), "https://test-endpoint.com");
    assert_eq!(s3_config.get_field_value(FocusField::AccessKeyId), "test-access-key");
    assert_eq!(s3_config.get_field_value(FocusField::SecretAccessKey), "test-secret-key");
    assert_eq!(s3_config.get_field_value(FocusField::PathStyle), "true");
    
    // Test getting a non-S3 field (should return empty string)
    assert_eq!(s3_config.get_field_value(FocusField::PgHost), "");
}

#[test]
fn test_s3_config_set_field_value() {
    let mut s3_config = S3Config {
        bucket: "".to_string(),
        region: "".to_string(),
        prefix: "".to_string(),
        endpoint_url: "".to_string(),
        access_key_id: "".to_string(),
        secret_access_key: "".to_string(),
        path_style: false,
        error_message: None,
        test_s3_button: false,
    };
    
    // Test setting field values
    s3_config.set_field_value(FocusField::Bucket, "new-bucket".to_string());
    s3_config.set_field_value(FocusField::Region, "eu-west-1".to_string());
    s3_config.set_field_value(FocusField::Prefix, "new-prefix".to_string());
    s3_config.set_field_value(FocusField::EndpointUrl, "https://new-endpoint.com".to_string());
    s3_config.set_field_value(FocusField::AccessKeyId, "new-access-key".to_string());
    s3_config.set_field_value(FocusField::SecretAccessKey, "new-secret-key".to_string());
    s3_config.set_field_value(FocusField::PathStyle, "true".to_string());
    
    // Verify the values were set correctly
    assert_eq!(s3_config.bucket, "new-bucket");
    assert_eq!(s3_config.region, "eu-west-1");
    assert_eq!(s3_config.prefix, "new-prefix");
    assert_eq!(s3_config.endpoint_url, "https://new-endpoint.com");
    assert_eq!(s3_config.access_key_id, "new-access-key");
    assert_eq!(s3_config.secret_access_key, "new-secret-key");
    assert_eq!(s3_config.path_style, true);
    
    // Test setting a non-S3 field (should have no effect)
    s3_config.set_field_value(FocusField::PgHost, "should-not-change-anything".to_string());
    assert_eq!(s3_config.bucket, "new-bucket"); // Verify no change
}

#[test]
fn test_postgres_config() {
    let pg_config = PostgresConfig {
        host: Some("localhost".to_string()),
        port: Some(5432),
        username: Some("postgres".to_string()),
        password: Some("password".to_string()),
        use_ssl: false,
        db_name: Some("postgres".to_string()),
    };

    assert_debug_snapshot!(pg_config);
}

#[test]
fn test_postgres_config_focus_fields() {
    // Test that we can get all focus fields for PostgreSQL settings
    let fields = PostgresConfig::focus_fields();
    
    // Verify we have the expected number of fields
    assert_eq!(fields.len(), 6);
    
    // Verify all expected fields are present
    assert!(fields.contains(&FocusField::PgHost));
    assert!(fields.contains(&FocusField::PgPort));
    assert!(fields.contains(&FocusField::PgUsername));
    assert!(fields.contains(&FocusField::PgPassword));
    assert!(fields.contains(&FocusField::PgSsl));
    assert!(fields.contains(&FocusField::PgDbName));
}

#[test]
fn test_postgres_config_contains_field() {
    // Test that contains_field correctly identifies PostgreSQL fields
    assert!(PostgresConfig::contains_field(FocusField::PgHost));
    assert!(PostgresConfig::contains_field(FocusField::PgPort));
    assert!(PostgresConfig::contains_field(FocusField::PgUsername));
    assert!(PostgresConfig::contains_field(FocusField::PgPassword));
    assert!(PostgresConfig::contains_field(FocusField::PgSsl));
    assert!(PostgresConfig::contains_field(FocusField::PgDbName));
    
    // Test that it correctly rejects non-PostgreSQL fields
    assert!(!PostgresConfig::contains_field(FocusField::Bucket));
    assert!(!PostgresConfig::contains_field(FocusField::SnapshotList));
    assert!(!PostgresConfig::contains_field(FocusField::RestoreTarget));
}

#[test]
fn test_postgres_config_get_field_value() {
    let pg_config = PostgresConfig {
        host: Some("localhost".to_string()),
        port: Some(5432),
        username: Some("postgres".to_string()),
        password: Some("password".to_string()),
        use_ssl: true,
        db_name: Some("postgres".to_string()),
    };
    
    // Test getting field values
    assert_eq!(pg_config.get_field_value(FocusField::PgHost), "localhost");
    assert_eq!(pg_config.get_field_value(FocusField::PgPort), "5432");
    assert_eq!(pg_config.get_field_value(FocusField::PgUsername), "postgres");
    assert_eq!(pg_config.get_field_value(FocusField::PgPassword), "password");
    assert_eq!(pg_config.get_field_value(FocusField::PgSsl), "true");
    assert_eq!(pg_config.get_field_value(FocusField::PgDbName), "postgres");
    
    // Test getting a non-PostgreSQL field (should return empty string)
    assert_eq!(pg_config.get_field_value(FocusField::Bucket), "");
    
    // Test with None values
    let empty_pg_config = PostgresConfig {
        host: None,
        port: None,
        username: None,
        password: None,
        use_ssl: false,
        db_name: None,
    };
    
    assert_eq!(empty_pg_config.get_field_value(FocusField::PgHost), "");
    assert_eq!(empty_pg_config.get_field_value(FocusField::PgPort), "");
    assert_eq!(empty_pg_config.get_field_value(FocusField::PgUsername), "");
    assert_eq!(empty_pg_config.get_field_value(FocusField::PgPassword), "");
    assert_eq!(empty_pg_config.get_field_value(FocusField::PgSsl), "false");
    assert_eq!(empty_pg_config.get_field_value(FocusField::PgDbName), "");
}

#[test]
fn test_postgres_config_set_field_value() {
    let mut pg_config = PostgresConfig {
        host: None,
        port: None,
        username: None,
        password: None,
        use_ssl: false,
        db_name: None,
    };
    
    // Test setting field values
    pg_config.set_field_value(FocusField::PgHost, "new-host".to_string());
    pg_config.set_field_value(FocusField::PgPort, "5433".to_string());
    pg_config.set_field_value(FocusField::PgUsername, "new-user".to_string());
    pg_config.set_field_value(FocusField::PgPassword, "new-password".to_string());
    pg_config.set_field_value(FocusField::PgSsl, "true".to_string());
    pg_config.set_field_value(FocusField::PgDbName, "new-database".to_string());
    
    // Verify the values were set correctly
    assert_eq!(pg_config.host, Some("new-host".to_string()));
    assert_eq!(pg_config.port, Some(5433));
    assert_eq!(pg_config.username, Some("new-user".to_string()));
    assert_eq!(pg_config.password, Some("new-password".to_string()));
    assert_eq!(pg_config.use_ssl, true);
    assert_eq!(pg_config.db_name, Some("new-database".to_string()));
    
    // Test setting a non-PostgreSQL field (should have no effect)
    pg_config.set_field_value(FocusField::Bucket, "should-not-change-anything".to_string());
    assert_eq!(pg_config.host, Some("new-host".to_string())); // Verify no change
    
    // Test with invalid port value
    pg_config.set_field_value(FocusField::PgPort, "not-a-number".to_string());
    assert_eq!(pg_config.port, None); // Should be None when parse fails
}

#[test]
fn test_elasticsearch_config() {
    let es_config = ElasticsearchConfig {
        host: Some("http://localhost:9200".to_string()),
        index: Some("test-index".to_string()),
    };

    assert_debug_snapshot!(es_config);
}

#[test]
fn test_elasticsearch_config_focus_fields() {
    // Test that we can get all focus fields for Elasticsearch settings
    let fields = ElasticsearchConfig::focus_fields();
    
    // Verify we have the expected number of fields
    assert_eq!(fields.len(), 2);
    
    // Verify all expected fields are present
    assert!(fields.contains(&FocusField::EsHost));
    assert!(fields.contains(&FocusField::EsIndex));
}

#[test]
fn test_elasticsearch_config_contains_field() {
    // Test that contains_field correctly identifies Elasticsearch fields
    assert!(ElasticsearchConfig::contains_field(FocusField::EsHost));
    assert!(ElasticsearchConfig::contains_field(FocusField::EsIndex));
    
    // Test that it correctly rejects non-Elasticsearch fields
    assert!(!ElasticsearchConfig::contains_field(FocusField::Bucket));
    assert!(!ElasticsearchConfig::contains_field(FocusField::PgHost));
    assert!(!ElasticsearchConfig::contains_field(FocusField::QdrantApiKey));
}

#[test]
fn test_elasticsearch_config_get_field_value() {
    let es_config = ElasticsearchConfig {
        host: Some("http://localhost:9200".to_string()),
        index: Some("test-index".to_string()),
    };
    
    // Test getting field values
    assert_eq!(es_config.get_field_value(FocusField::EsHost), "http://localhost:9200");
    assert_eq!(es_config.get_field_value(FocusField::EsIndex), "test-index");
    
    // Test getting a non-Elasticsearch field (should return empty string)
    assert_eq!(es_config.get_field_value(FocusField::Bucket), "");
    
    // Test with None values
    let empty_es_config = ElasticsearchConfig {
        host: None,
        index: None,
    };
    
    assert_eq!(empty_es_config.get_field_value(FocusField::EsHost), "");
    assert_eq!(empty_es_config.get_field_value(FocusField::EsIndex), "");
}

#[test]
fn test_elasticsearch_config_set_field_value() {
    let mut es_config = ElasticsearchConfig {
        host: None,
        index: None,
    };
    
    // Test setting field values
    es_config.set_field_value(FocusField::EsHost, "http://new-host:9200".to_string());
    es_config.set_field_value(FocusField::EsIndex, "new-index".to_string());
    
    // Verify the values were set correctly
    assert_eq!(es_config.host, Some("http://new-host:9200".to_string()));
    assert_eq!(es_config.index, Some("new-index".to_string()));
    
    // Test setting a non-Elasticsearch field (should have no effect)
    es_config.set_field_value(FocusField::Bucket, "should-not-change-anything".to_string());
    assert_eq!(es_config.host, Some("http://new-host:9200".to_string())); // Verify no change
}

#[test]
fn test_qdrant_config() {
    let qdrant_config = QdrantConfig {
        host: Some("http://localhost:6333".to_string()),
        collection: Some("test-collection".to_string()),
        api_key: Some("test-api-key".to_string()),
    };

    assert_debug_snapshot!(qdrant_config);
}

#[test]
fn test_qdrant_config_focus_fields() {
    // Test that we can get all focus fields for Qdrant settings
    let fields = QdrantConfig::focus_fields();
    
    // Verify we have the expected number of fields
    assert_eq!(fields.len(), 3);
    
    // Verify all expected fields are present
    assert!(fields.contains(&FocusField::EsHost)); // Reused for Qdrant host
    assert!(fields.contains(&FocusField::EsIndex)); // Reused for collection
    assert!(fields.contains(&FocusField::QdrantApiKey));
}

#[test]
fn test_qdrant_config_contains_field() {
    // Test that contains_field correctly identifies Qdrant fields
    assert!(QdrantConfig::contains_field(FocusField::EsHost)); // Reused for Qdrant host
    assert!(QdrantConfig::contains_field(FocusField::EsIndex)); // Reused for collection
    assert!(QdrantConfig::contains_field(FocusField::QdrantApiKey));
    
    // Test that it correctly rejects non-Qdrant fields
    assert!(!QdrantConfig::contains_field(FocusField::Bucket));
    assert!(!QdrantConfig::contains_field(FocusField::PgHost));
}

#[test]
fn test_qdrant_config_get_field_value() {
    let qdrant_config = QdrantConfig {
        host: Some("http://localhost:6333".to_string()),
        collection: Some("test-collection".to_string()),
        api_key: Some("test-api-key".to_string()),
    };
    
    // Test getting field values
    assert_eq!(qdrant_config.get_field_value(FocusField::EsHost), "http://localhost:6333");
    assert_eq!(qdrant_config.get_field_value(FocusField::EsIndex), "test-collection");
    assert_eq!(qdrant_config.get_field_value(FocusField::QdrantApiKey), "test-api-key");
    
    // Test getting a non-Qdrant field (should return empty string)
    assert_eq!(qdrant_config.get_field_value(FocusField::Bucket), "");
    
    // Test with None values
    let empty_qdrant_config = QdrantConfig {
        host: None,
        collection: None,
        api_key: None,
    };
    
    assert_eq!(empty_qdrant_config.get_field_value(FocusField::EsHost), "");
    assert_eq!(empty_qdrant_config.get_field_value(FocusField::EsIndex), "");
    assert_eq!(empty_qdrant_config.get_field_value(FocusField::QdrantApiKey), "");
}

#[test]
fn test_qdrant_config_set_field_value() {
    let mut qdrant_config = QdrantConfig {
        host: None,
        collection: None,
        api_key: None,
    };
    
    // Test setting field values
    qdrant_config.set_field_value(FocusField::EsHost, "http://new-host:6333".to_string());
    qdrant_config.set_field_value(FocusField::EsIndex, "new-collection".to_string());
    qdrant_config.set_field_value(FocusField::QdrantApiKey, "new-api-key".to_string());
    
    // Verify the values were set correctly
    assert_eq!(qdrant_config.host, Some("http://new-host:6333".to_string()));
    assert_eq!(qdrant_config.collection, Some("new-collection".to_string()));
    assert_eq!(qdrant_config.api_key, Some("new-api-key".to_string()));
    
    // Test setting a non-Qdrant field (should have no effect)
    qdrant_config.set_field_value(FocusField::Bucket, "should-not-change-anything".to_string());
    assert_eq!(qdrant_config.host, Some("http://new-host:6333".to_string())); // Verify no change
}

#[test]
fn test_backup_metadata() {
    let datetime = create_test_aws_datetime();
    let backup = BackupMetadata {
        key: "test-snapshot-1.sql.gz".to_string(),
        size: 1024 * 1024 * 10, // 10 MB
        last_modified: datetime,
    };

    assert_debug_snapshot!(backup);
}

#[test]
fn test_popup_states() {
    let datetime = create_test_aws_datetime();
    let backup = BackupMetadata {
        key: "test-snapshot-1.sql.gz".to_string(),
        size: 1024 * 1024 * 10, // 10 MB
        last_modified: datetime,
    };

    let hidden = PopupState::Hidden;
    let confirm_restore = PopupState::ConfirmRestore(backup.clone());
    let downloading = PopupState::Downloading(backup.clone(), 0.5, 1024.0 * 1024.0);
    let confirm_cancel = PopupState::ConfirmCancel(backup.clone(), 0.5, 1024.0 * 1024.0);
    let error = PopupState::Error("Test error message".to_string());
    let success = PopupState::Success("Test success message".to_string());

    assert_debug_snapshot!("popup_state_hidden", hidden);
    assert_debug_snapshot!("popup_state_confirm_restore", confirm_restore);
    assert_debug_snapshot!("popup_state_downloading", downloading);
    assert_debug_snapshot!("popup_state_confirm_cancel", confirm_cancel);
    assert_debug_snapshot!("popup_state_error", error);
    assert_debug_snapshot!("popup_state_success", success);
}

#[test]
fn test_focus_fields() {
    assert_debug_snapshot!("focus_field_snapshot_list", FocusField::SnapshotList);
    assert_debug_snapshot!("focus_field_bucket", FocusField::Bucket);
    assert_debug_snapshot!("focus_field_region", FocusField::Region);
    assert_debug_snapshot!("focus_field_prefix", FocusField::Prefix);
    assert_debug_snapshot!("focus_field_endpoint_url", FocusField::EndpointUrl);
    assert_debug_snapshot!("focus_field_access_key_id", FocusField::AccessKeyId);
    assert_debug_snapshot!("focus_field_secret_access_key", FocusField::SecretAccessKey);
    assert_debug_snapshot!("focus_field_path_style", FocusField::PathStyle);
}

#[test]
fn test_restore_target() {
    assert_debug_snapshot!("restore_target_postgres", RestoreTarget::Postgres);
    assert_debug_snapshot!("restore_target_elasticsearch", RestoreTarget::Elasticsearch);
    assert_debug_snapshot!("restore_target_qdrant", RestoreTarget::Qdrant);
}

#[test]
fn test_restore_target_focus_fields() {
    // Test that we can get focus fields for each restore target
    let postgres_fields = RestoreTarget::Postgres.focus_fields();
    let elasticsearch_fields = RestoreTarget::Elasticsearch.focus_fields();
    let qdrant_fields = RestoreTarget::Qdrant.focus_fields();
    
    // Verify we have the expected number of fields for each target
    assert_eq!(postgres_fields.len(), 6);
    assert_eq!(elasticsearch_fields.len(), 2);
    assert_eq!(qdrant_fields.len(), 3);
    
    // Verify first field for each target
    assert_eq!(RestoreTarget::Postgres.first_focus_field(), FocusField::PgHost);
    assert_eq!(RestoreTarget::Elasticsearch.first_focus_field(), FocusField::EsHost);
    assert_eq!(RestoreTarget::Qdrant.first_focus_field(), FocusField::QdrantApiKey);
    
    // Verify fields for Postgres
    assert!(postgres_fields.contains(&FocusField::PgHost));
    assert!(postgres_fields.contains(&FocusField::PgPort));
    assert!(postgres_fields.contains(&FocusField::PgUsername));
    assert!(postgres_fields.contains(&FocusField::PgPassword));
    assert!(postgres_fields.contains(&FocusField::PgSsl));
    assert!(postgres_fields.contains(&FocusField::PgDbName));
    
    // Verify fields for Elasticsearch
    assert!(elasticsearch_fields.contains(&FocusField::EsHost));
    assert!(elasticsearch_fields.contains(&FocusField::EsIndex));
    
    // Verify fields for Qdrant
    assert!(qdrant_fields.contains(&FocusField::EsHost));
    assert!(qdrant_fields.contains(&FocusField::EsIndex));
    assert!(qdrant_fields.contains(&FocusField::QdrantApiKey));
}

#[test]
fn test_input_modes() {
    assert_debug_snapshot!("input_mode_normal", InputMode::Normal);
    assert_debug_snapshot!("input_mode_editing", InputMode::Editing);
}

use rustored::ui::models::s3_config::S3Config;

// This test verifies that the S3 secret key is properly masked
#[test]
fn test_s3_secret_key_masking() {
    // Create a new S3Config with a secret key
    let mut s3_config = S3Config::default();
    s3_config.secret_access_key = "supersecret".to_string();
    
    // Test that the secret key is fully masked when not editing
    let display_text = s3_config.get_secret_key_display(false, "");
    assert_eq!(display_text, "Secret Access Key: [hidden]", "Secret key should be hidden when not editing");
    
    // Test that the secret key is visible when editing
    let input_buffer = "newsecret";
    let display_text = s3_config.get_secret_key_display(true, input_buffer);
    assert_eq!(display_text, format!("Secret Access Key: {}", input_buffer), "Secret key should be visible when editing");
}



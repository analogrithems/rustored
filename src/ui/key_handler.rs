// This module contains key handling logic for the Rustored application
// It processes keyboard events and updates application state accordingly

use crate::ui::models::{PopupState, InputMode, FocusField, RestoreTarget};
use crate::ui::rustored::RustoredApp;
use crossterm::event::{KeyCode, KeyEvent};
use anyhow::Result;
use log::debug;

/// Handle popup key events
/// 
/// This function processes key events when a popup is displayed
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
/// * `key` - The key event to process
/// 
/// # Returns
/// 
/// A Result containing an Option<String> which is Some if a snapshot path is returned
pub async fn handle_popup_events(app: &mut RustoredApp, key: KeyEvent) -> Result<Option<String>> {
    debug!("Handling popup key event: {:?}", key);
    
    match &app.popup_state {
        PopupState::ConfirmRestore(snapshot) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Download the snapshot
                    let tmp_path = std::env::temp_dir().join(format!("rustored_snapshot_{}", snapshot.key.replace("/", "_")));
                    return app.snapshot_browser.download_snapshot(snapshot, &tmp_path).await;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    app.popup_state = PopupState::Hidden;
                }
                _ => {}
            }
            return Ok(None);
        }
        PopupState::ConfirmCancel(_, _, _) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.popup_state = PopupState::Error("Download cancelled".to_string());
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    // Resume downloading
                    if let PopupState::ConfirmCancel(snapshot, progress, rate) = &app.popup_state {
                        app.popup_state = PopupState::Downloading(snapshot.clone(), *progress, *rate);
                    }
                }
                _ => {}
            }
            return Ok(None);
        }
        PopupState::Downloading(_, _, _) => {
            if key.code == KeyCode::Esc {
                // Ask for confirmation
                if let PopupState::Downloading(snapshot, progress, rate) = &app.popup_state {
                    app.popup_state = PopupState::ConfirmCancel(snapshot.clone(), *progress, *rate);
                }
            }
            return Ok(None);
        }
        PopupState::Error(_) | PopupState::Success(_) => {
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                app.popup_state = PopupState::Hidden;
            }
            return Ok(None);
        }
        _ => {}
    }
    
    Ok(None)
}

/// Handle editing mode key events
/// 
/// This function processes key events when in editing mode
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
/// * `key` - The key event to process
/// 
/// # Returns
/// 
/// A Result containing an Option<String> which is Some if a snapshot path is returned
pub async fn handle_editing_mode(app: &mut RustoredApp, key: KeyEvent) -> Result<Option<String>> {
    debug!("Handling editing mode key event: {:?}", key);
    
    match key.code {
        KeyCode::Enter => {
            // Apply the edited value
            match app.focus {
                FocusField::Bucket => app.s3_config.bucket = app.input_buffer.clone(),
                FocusField::Region => app.s3_config.region = app.input_buffer.clone(),
                FocusField::Prefix => app.s3_config.prefix = app.input_buffer.clone(),
                FocusField::EndpointUrl => app.s3_config.endpoint_url = app.input_buffer.clone(),
                FocusField::AccessKeyId => app.s3_config.access_key_id = app.input_buffer.clone(),
                FocusField::SecretAccessKey => app.s3_config.secret_access_key = app.input_buffer.clone(),
                FocusField::PathStyle => {
                    app.s3_config.path_style = app.input_buffer.to_lowercase() == "true";
                }
                FocusField::PgHost => {
                    if let Some(host) = &mut app.pg_config.host {
                        *host = app.input_buffer.clone();
                    } else {
                        app.pg_config.host = Some(app.input_buffer.clone());
                    }
                }
                FocusField::PgPort => {
                    if let Ok(port) = app.input_buffer.parse::<u16>() {
                        app.pg_config.port = Some(port);
                    }
                }
                FocusField::PgUsername => {
                    if let Some(username) = &mut app.pg_config.username {
                        *username = app.input_buffer.clone();
                    } else {
                        app.pg_config.username = Some(app.input_buffer.clone());
                    }
                }
                FocusField::PgPassword => {
                    if let Some(password) = &mut app.pg_config.password {
                        *password = app.input_buffer.clone();
                    } else {
                        app.pg_config.password = Some(app.input_buffer.clone());
                    }
                }
                FocusField::PgSsl => {
                    app.pg_config.use_ssl = app.input_buffer.to_lowercase() == "true";
                }
                FocusField::PgDbName => {
                    if let Some(db_name) = &mut app.pg_config.db_name {
                        *db_name = app.input_buffer.clone();
                    } else {
                        app.pg_config.db_name = Some(app.input_buffer.clone());
                    }
                }
                FocusField::EsHost => {
                    if let Some(host) = &mut app.es_config.host {
                        *host = app.input_buffer.clone();
                    } else {
                        app.es_config.host = Some(app.input_buffer.clone());
                    }
                }
                FocusField::EsIndex => {
                    if let Some(index) = &mut app.es_config.index {
                        *index = app.input_buffer.clone();
                    } else {
                        app.es_config.index = Some(app.input_buffer.clone());
                    }
                }
                FocusField::QdrantApiKey => {
                    if let Some(api_key) = &mut app.qdrant_config.api_key {
                        *api_key = app.input_buffer.clone();
                    } else {
                        app.qdrant_config.api_key = Some(app.input_buffer.clone());
                    }
                }
                _ => {}
            }
            app.input_mode = InputMode::Normal;
            
            // Update S3 client with new settings if S3 settings were changed
            if matches!(app.focus, 
                FocusField::Bucket | 
                FocusField::Region | 
                FocusField::Prefix | 
                FocusField::EndpointUrl | 
                FocusField::AccessKeyId | 
                FocusField::SecretAccessKey | 
                FocusField::PathStyle
            ) {
                app.snapshot_browser.s3_config = app.s3_config.clone();
                let _ = app.snapshot_browser.init_client().await;
                
                // Reload snapshots with new settings
                if let Err(e) = app.snapshot_browser.load_snapshots().await {
                    debug!("Failed to load snapshots: {}", e);
                }
            }
        }
        KeyCode::Esc => {
            // Cancel editing
            app.input_mode = InputMode::Normal;
            app.input_buffer.clear();
        }
        KeyCode::Backspace => {
            // Remove character
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            // Add character
            app.input_buffer.push(c);
        }
        _ => {}
    }
    
    Ok(None)
}

/// Handle normal mode key events
/// 
/// This function processes key events when in normal mode
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
/// * `key` - The key event to process
/// 
/// # Returns
/// 
/// A Result containing an Option<String> which is Some if a snapshot path is returned
pub async fn handle_normal_mode(app: &mut RustoredApp, key: KeyEvent) -> Result<Option<String>> {
    debug!("Handling normal mode key event: {:?}", key);
    
    match key.code {
        KeyCode::Char('q') => {
            // Quit
            return Ok(Some("quit".to_string()));
        }
        KeyCode::Char('r') => {
            // Reload snapshots
            if let Err(e) = app.snapshot_browser.load_snapshots().await {
                debug!("Failed to load snapshots: {}", e);
            }
        }
        KeyCode::Char('t') => {
            // Test S3 connection when focus is on S3 settings window
            if matches!(app.focus, 
                FocusField::Bucket | 
                FocusField::Region | 
                FocusField::Prefix | 
                FocusField::EndpointUrl | 
                FocusField::AccessKeyId | 
                FocusField::SecretAccessKey | 
                FocusField::PathStyle
            ) {
                // Show testing popup
                app.popup_state = PopupState::TestingS3;
                
                // Test connection and update popup state with result
                if let Err(e) = app.s3_config.test_connection(|state| app.popup_state = state).await {
                    debug!("S3 connection test failed: {}", e);
                }
            }
        },
        KeyCode::Char('p') => {
            // Test PostgreSQL connection when focus is on PostgreSQL settings window
            if matches!(app.focus, 
                FocusField::PgHost | 
                FocusField::PgPort | 
                FocusField::PgUsername | 
                FocusField::PgPassword | 
                FocusField::PgSsl | 
                FocusField::PgDbName
            ) {
                // Only test if required fields are set
                if app.pg_config.host.is_some() && 
                   app.pg_config.port.is_some() && 
                   app.pg_config.db_name.is_some() {
                    // Show testing popup
                    app.popup_state = PopupState::TestingPg;
                    
                    // Test connection and update popup state with result
                    if let Err(e) = app.pg_config.test_connection(|state| app.popup_state = state).await {
                        debug!("PostgreSQL connection test failed: {}", e);
                    }
                }
            }
        }
        KeyCode::Tab => handle_tab_navigation(app),
        KeyCode::Up => handle_up_navigation(app),
        KeyCode::Down => handle_down_navigation(app),
        KeyCode::Enter => handle_enter_key(app),
        _ => {}
    }
    
    Ok(None)
}

/// Handle Tab key navigation
/// 
/// This function processes Tab key presses to navigate between main UI sections
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
fn handle_tab_navigation(app: &mut RustoredApp) {
    debug!("Handling Tab navigation, current focus: {:?}", app.focus);
    
    // Cycle between main window sections only
    app.focus = match app.focus {
        // S3 Settings fields - move to Restore Target settings
        FocusField::Bucket | 
        FocusField::Region | 
        FocusField::Prefix | 
        FocusField::EndpointUrl | 
        FocusField::AccessKeyId | 
        FocusField::SecretAccessKey | 
        FocusField::PathStyle => {
            // Move to restore target settings
            match app.restore_target {
                RestoreTarget::Postgres => FocusField::PgHost,
                RestoreTarget::Elasticsearch => FocusField::EsHost,
                RestoreTarget::Qdrant => FocusField::QdrantApiKey,
            }
        }
        // Restore Target settings - move to Snapshot List
        FocusField::PgHost | 
        FocusField::PgPort | 
        FocusField::PgUsername | 
        FocusField::PgPassword | 
        FocusField::PgSsl | 
        FocusField::PgDbName |
        FocusField::EsHost |
        FocusField::EsIndex |
        FocusField::QdrantApiKey => FocusField::SnapshotList,
        // Snapshot list - move back to S3 Settings
        FocusField::SnapshotList => FocusField::Bucket,
        // Default case
        _ => FocusField::Bucket,
    };
    
    debug!("New focus after Tab navigation: {:?}", app.focus);
}

/// Handle Up key navigation
/// 
/// This function processes Up key presses to navigate within UI sections
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
fn handle_up_navigation(app: &mut RustoredApp) {
    debug!("Handling Up navigation, current focus: {:?}", app.focus);
    
    match app.focus {
        FocusField::SnapshotList => {
            // Navigate snapshot list
            // Navigate snapshot list
            if !app.snapshot_browser.snapshots.is_empty() {
                app.snapshot_browser.selected_index = if app.snapshot_browser.selected_index == 0 {
                    app.snapshot_browser.snapshots.len() - 1
                } else {
                    app.snapshot_browser.selected_index - 1
                };
            }
        }
        _ => {
            // Navigate within settings panels
            let focus_fields = match app.focus {
                // S3 Settings fields
                FocusField::Bucket | 
                FocusField::Region | 
                FocusField::Prefix | 
                FocusField::EndpointUrl | 
                FocusField::AccessKeyId | 
                FocusField::SecretAccessKey | 
                FocusField::PathStyle => crate::ui::models::S3Config::focus_fields(),
                
                // PostgreSQL Settings fields
                FocusField::PgHost | 
                FocusField::PgPort | 
                FocusField::PgUsername | 
                FocusField::PgPassword | 
                FocusField::PgSsl | 
                FocusField::PgDbName => crate::ui::models::PostgresConfig::focus_fields(),
                
                // Elasticsearch Settings fields
                FocusField::EsHost | 
                FocusField::EsIndex => crate::ui::models::ElasticsearchConfig::focus_fields(),
                
                // Qdrant Settings fields
                FocusField::QdrantApiKey => crate::ui::models::QdrantConfig::focus_fields(),
                
                // Default case
                _ => &[],
            };
            
            if !focus_fields.is_empty() {
                // Find current index
                let current_index = focus_fields.iter().position(|&f| f == app.focus).unwrap_or(0);
                // Move to previous field (wrap around if at first field)
                let prev_index = if current_index == 0 { focus_fields.len() - 1 } else { current_index - 1 };
                app.focus = focus_fields[prev_index];
            }
        }
    }
    
    debug!("New focus after Up navigation: {:?}", app.focus);
}

/// Handle Down key navigation
/// 
/// This function processes Down key presses to navigate within UI sections
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
fn handle_down_navigation(app: &mut RustoredApp) {
    debug!("Handling Down navigation, current focus: {:?}", app.focus);
    
    match app.focus {
        FocusField::SnapshotList => {
            // Navigate snapshot list
            // Navigate snapshot list
            if !app.snapshot_browser.snapshots.is_empty() {
                app.snapshot_browser.selected_index = 
                    (app.snapshot_browser.selected_index + 1) % app.snapshot_browser.snapshots.len();
            }
        }
        _ => {
            // Navigate within settings panels
            let focus_fields = match app.focus {
                // S3 Settings fields
                FocusField::Bucket | 
                FocusField::Region | 
                FocusField::Prefix | 
                FocusField::EndpointUrl | 
                FocusField::AccessKeyId | 
                FocusField::SecretAccessKey | 
                FocusField::PathStyle => crate::ui::models::S3Config::focus_fields(),
                
                // PostgreSQL Settings fields
                FocusField::PgHost | 
                FocusField::PgPort | 
                FocusField::PgUsername | 
                FocusField::PgPassword | 
                FocusField::PgSsl | 
                FocusField::PgDbName => crate::ui::models::PostgresConfig::focus_fields(),
                
                // Elasticsearch Settings fields
                FocusField::EsHost | 
                FocusField::EsIndex => crate::ui::models::ElasticsearchConfig::focus_fields(),
                
                // Qdrant Settings fields
                FocusField::QdrantApiKey => crate::ui::models::QdrantConfig::focus_fields(),
                
                // Default case
                _ => &[],
            };
            
            if !focus_fields.is_empty() {
                // Find current index
                let current_index = focus_fields.iter().position(|&f| f == app.focus).unwrap_or(0);
                // Move to next field (wrap around if at last field)
                let next_index = (current_index + 1) % focus_fields.len();
                app.focus = focus_fields[next_index];
            }
        }
    }
    
    debug!("New focus after Down navigation: {:?}", app.focus);
}

/// Handle Enter key press
/// 
/// This function processes Enter key presses to edit fields or select snapshots
/// 
/// # Arguments
/// 
/// * `app` - A mutable reference to the RustoredApp
fn handle_enter_key(app: &mut RustoredApp) {
    debug!("Handling Enter key press, current focus: {:?}", app.focus);
    
    match app.focus {
        FocusField::SnapshotList => {
            // Select a snapshot for restoration
            // Select a snapshot for restoration if one is available
            if !app.snapshot_browser.snapshots.is_empty() {
                let snapshot = &app.snapshot_browser.snapshots[app.snapshot_browser.selected_index];
                app.popup_state = PopupState::ConfirmRestore(snapshot.clone());
            }
        }
        _ => {
            // Enter edit mode for the current field
            app.input_mode = InputMode::Editing;
            
            // Set input buffer to current field value
            app.input_buffer = match app.focus {
                // S3 Settings fields
                FocusField::Bucket => app.s3_config.bucket.clone(),
                FocusField::Region => app.s3_config.region.clone(),
                FocusField::Prefix => app.s3_config.prefix.clone(),
                FocusField::EndpointUrl => app.s3_config.endpoint_url.clone(),
                FocusField::AccessKeyId => app.s3_config.access_key_id.clone(),
                FocusField::SecretAccessKey => app.s3_config.secret_access_key.clone(),
                FocusField::PathStyle => app.s3_config.path_style.to_string(),
                
                // PostgreSQL Settings fields
                FocusField::PgHost => app.pg_config.host.clone().unwrap_or_default(),
                FocusField::PgPort => app.pg_config.port.map(|p| p.to_string()).unwrap_or_default(),
                FocusField::PgUsername => app.pg_config.username.clone().unwrap_or_default(),
                FocusField::PgPassword => app.pg_config.password.clone().unwrap_or_default(),
                FocusField::PgSsl => app.pg_config.use_ssl.to_string(),
                FocusField::PgDbName => app.pg_config.db_name.clone().unwrap_or_default(),
                
                // Elasticsearch Settings fields
                FocusField::EsHost => app.es_config.host.clone().unwrap_or_default(),
                FocusField::EsIndex => app.es_config.index.clone().unwrap_or_default(),
                
                // Qdrant Settings fields
                FocusField::QdrantApiKey => app.qdrant_config.api_key.clone().unwrap_or_default(),
                
                // Default case
                _ => String::new(),
            };
        }
    }
    
    debug!("After Enter key press: input_mode={:?}, input_buffer={}", app.input_mode, app.input_buffer);
}

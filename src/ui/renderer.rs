use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear, Tabs},
    Frame,
};
use chrono::{DateTime, Utc};

use crate::ui::models::{FocusField, PopupState};
use crate::ui::browser::SnapshotBrowser;

/// Helper function to create a centered rect
pub fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Render the UI
pub fn ui<B: Backend>(f: &mut Frame, browser: &mut SnapshotBrowser) {
    // We'll handle the editing mode overlay at the end to ensure it doesn't hide the UI
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Title
                Constraint::Length(9),  // Restore Target & Connection Fields
                Constraint::Length(8),  // S3 Settings
                Constraint::Min(10),    // Snapshot List
            ]
            .as_ref(),
        )
        .split(f.size());

    // Title
    let title = Paragraph::new("Rustored Snapshot Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Split restore area into tabs and connection field sections
    let restore_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(chunks[1]);

    // Restore Target Tabs
    let titles: Vec<Line> = ["Postgres", "Elasticsearch", "Qdrant"]
        .iter()
        .map(|t| Line::from(Span::raw(*t)))
        .collect();
    let selected = match browser.restore_target {
        crate::datastore::RestoreTarget::Postgres => 0,
        crate::datastore::RestoreTarget::Elasticsearch => 1,
        crate::datastore::RestoreTarget::Qdrant => 2,
    };

    // Tabs at top of restore area
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Restore Target"))
        .highlight_style(Style::default().fg(Color::Green))
        .select(selected);
    f.render_widget(tabs, restore_chunks[0]);

    // Determine block title based on selected datastore
    let block_title = match browser.restore_target {
        crate::datastore::RestoreTarget::Postgres => "PostgreSQL Settings",
        crate::datastore::RestoreTarget::Elasticsearch => "Elasticsearch Settings",
        crate::datastore::RestoreTarget::Qdrant => "Qdrant Settings",
    };

    // Datastore connection fields within a dynamic titled block
    f.render_widget(Block::default().borders(Borders::ALL).title(block_title), restore_chunks[1]);
    if browser.restore_target == crate::datastore::RestoreTarget::Postgres {
        // Editable PostgresConfig fields
        let field_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(vec![Constraint::Length(1); 6])
            .split(restore_chunks[1]);
        let fields = [
            (crate::ui::models::FocusField::PgHost, format!("Host: {}", browser.pg_config.host.as_deref().unwrap_or(""))),
            (crate::ui::models::FocusField::PgPort, format!("Port: {}", browser.pg_config.port.map(|p| p.to_string()).unwrap_or_default())),
            (crate::ui::models::FocusField::PgUsername, format!("Username: {}", browser.pg_config.username.as_deref().unwrap_or(""))),
            (crate::ui::models::FocusField::PgPassword, format!("Password: {}", if browser.pg_config.password.is_some() { "********" } else { "" })),
            (crate::ui::models::FocusField::PgSsl, format!("SSL: {}", browser.pg_config.use_ssl)),
            (crate::ui::models::FocusField::PgDbName, format!("Database: {}", browser.pg_config.db_name.as_deref().unwrap_or(""))),
        ];
        for (i, (focus_field, text)) in fields.iter().enumerate() {
            let style = if browser.focus == *focus_field {
                if browser.input_mode == crate::ui::models::InputMode::Editing {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                }
            } else {
                Style::default()
            };
            let content = if browser.focus == *focus_field && browser.input_mode == crate::ui::models::InputMode::Editing {
                browser.input_buffer.clone()
            } else {
                text.clone()
            };
            let para = Paragraph::new(content).style(style);
            f.render_widget(para, field_chunks[i]);
        }
    } else {
        let mut conn_lines = vec![];
        match browser.restore_target {
            crate::datastore::RestoreTarget::Postgres => {
                // Postgres connection fields
                conn_lines.push(Line::from(format!("Host: {}", browser.pg_config.host.as_deref().unwrap_or(""))));
                conn_lines.push(Line::from(format!("Port: {}", browser.pg_config.port.map(|p| p.to_string()).unwrap_or_default())));
                conn_lines.push(Line::from(format!("Username: {}", browser.pg_config.username.as_deref().unwrap_or(""))));
                conn_lines.push(Line::from(format!("Password: {}", if browser.pg_config.password.is_some() { "********" } else { "" })));
                conn_lines.push(Line::from(format!("SSL: {}", browser.pg_config.use_ssl)));
                conn_lines.push(Line::from(format!("Database: {}", browser.pg_config.db_name.as_deref().unwrap_or(""))));
            },
            crate::datastore::RestoreTarget::Elasticsearch => {
                conn_lines.push(Line::from(format!("Elasticsearch Host: {}", browser.es_config.host.as_deref().unwrap_or(""))));
                conn_lines.push(Line::from(format!("Index: {}", browser.es_config.index.as_deref().unwrap_or(""))));
            },
            crate::datastore::RestoreTarget::Qdrant => {
                conn_lines.push(Line::from(format!("Qdrant Host: {}", browser.qdrant_config.host.as_deref().unwrap_or(""))));
                conn_lines.push(Line::from(format!("Collection: {}", browser.qdrant_config.collection.as_deref().unwrap_or(""))));
                conn_lines.push(Line::from(format!("API Key: {}", browser.qdrant_config.api_key.as_deref().unwrap_or(""))));
            },
        }
        let conn_block = Paragraph::new(conn_lines)
            .block(Block::default().borders(Borders::ALL).title(block_title))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(conn_block, restore_chunks[1]);
    }

    // S3 Settings
    let s3_settings_block = Block::default()
        .title("S3 Settings")
        .borders(Borders::ALL);
    f.render_widget(s3_settings_block, chunks[2]);

    // S3 Settings Content
    let s3_settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![Constraint::Length(1); 7])
        .split(chunks[2]);

    // Bucket
    let bucket_style = if browser.focus == FocusField::Bucket {
        if browser.input_mode == crate::ui::models::InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let bucket_text = if browser.focus == FocusField::Bucket && browser.input_mode == crate::ui::models::InputMode::Editing {
        format!("Bucket: {}", browser.input_buffer)
    } else {
        format!("Bucket: {}", browser.s3_config.bucket)
    };

    let bucket = Paragraph::new(bucket_text)
        .style(bucket_style);
    f.render_widget(bucket, s3_settings_chunks[0]);

    // Region
    let region_style = if browser.focus == FocusField::Region {
        if browser.input_mode == crate::ui::models::InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let region_text = if browser.focus == FocusField::Region && browser.input_mode == crate::ui::models::InputMode::Editing {
        format!("Region: {}", browser.input_buffer)
    } else {
        format!("Region: {}", browser.s3_config.region)
    };

    let region = Paragraph::new(region_text)
        .style(region_style);
    f.render_widget(region, s3_settings_chunks[1]);

    // Prefix
    let prefix_style = if browser.focus == FocusField::Prefix {
        if browser.input_mode == crate::ui::models::InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let prefix_text = if browser.focus == FocusField::Prefix && browser.input_mode == crate::ui::models::InputMode::Editing {
        format!("Prefix: {}", browser.input_buffer)
    } else {
        format!("Prefix: {}", browser.s3_config.prefix)
    };

    let prefix = Paragraph::new(prefix_text)
        .style(prefix_style);
    f.render_widget(prefix, s3_settings_chunks[2]);

    // Endpoint URL
    let endpoint_style = if browser.focus == FocusField::EndpointUrl {
        if browser.input_mode == crate::ui::models::InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let endpoint_text = if browser.focus == FocusField::EndpointUrl && browser.input_mode == crate::ui::models::InputMode::Editing {
        format!("Endpoint URL: {}", browser.input_buffer)
    } else {
        format!("Endpoint URL: {}", browser.s3_config.endpoint_url)
    };

    let endpoint = Paragraph::new(endpoint_text)
        .style(endpoint_style);
    f.render_widget(endpoint, s3_settings_chunks[3]);

    // Access Key ID
    let access_key_style = if browser.focus == FocusField::AccessKeyId {
        if browser.input_mode == crate::ui::models::InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let access_key_text = if browser.focus == FocusField::AccessKeyId && browser.input_mode == crate::ui::models::InputMode::Editing {
        format!("Access Key ID: {}", browser.input_buffer)
    } else {
        format!("Access Key ID: {}", browser.s3_config.masked_access_key())
    };

    let access_key = Paragraph::new(access_key_text)
        .style(access_key_style);
    f.render_widget(access_key, s3_settings_chunks[4]);

    // Secret Access Key
    let secret_key_style = if browser.focus == FocusField::SecretAccessKey {
        if browser.input_mode == crate::ui::models::InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let secret_key_text = if browser.focus == FocusField::SecretAccessKey && browser.input_mode == crate::ui::models::InputMode::Editing {
        format!("Secret Access Key: {}", browser.input_buffer)
    } else {
        format!("Secret Access Key: {}", browser.s3_config.masked_secret_key())
    };

    let secret_key = Paragraph::new(secret_key_text)
        .style(secret_key_style);
    f.render_widget(secret_key, s3_settings_chunks[5]);

    // Path Style
    let path_style_style = if browser.focus == FocusField::PathStyle {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let path_style_text = format!("Path Style: {}", browser.s3_config.path_style);
    let path_style = Paragraph::new(path_style_text)
        .style(path_style_style);
    f.render_widget(path_style, s3_settings_chunks[6]);

    // Snapshot List
    let snapshot_style = if browser.focus == FocusField::SnapshotList {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let snapshot_block = Block::default()
        .title("Snapshots")
        .borders(Borders::ALL)
        .style(snapshot_style);

    let snapshot_items: Vec<ListItem> = browser.snapshots
        .iter()
        .enumerate()
        .map(|(i, snapshot)| {
            // Convert AWS DateTime to chrono DateTime
            let timestamp = snapshot.last_modified.as_secs_f64();
            let dt: DateTime<Utc> = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
            let formatted_date = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            let size_mb = snapshot.size as f64 / 1024.0 / 1024.0;
            let content = format!("{} - {:.2} MB - {}", snapshot.key, size_mb, formatted_date);
            let style = if i == browser.selected_index {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![Span::styled(content, style)]))
        })
        .collect();

    let snapshot_list = List::new(snapshot_items)
        .block(snapshot_block);
    f.render_widget(snapshot_list, chunks[3]);

    // Show help text at the bottom
    let help_text = match browser.input_mode {
        crate::ui::models::InputMode::Normal => "Press 'q' to quit, 'e' to edit, 't' to test connection, 'r' to refresh, Enter to select",
        crate::ui::models::InputMode::Editing => "Press Esc to cancel, Enter to save",
    };
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    let help_rect = Rect::new(
        chunks[3].x,
        chunks[3].y + chunks[3].height - 1,
        chunks[3].width,
        1,
    );
    f.render_widget(help_paragraph, help_rect);

    // We'll handle popups at the end to ensure they're on top

    // Show popup if needed - render last to ensure they're on top
    match &browser.popup_state {
        PopupState::ConfirmRestore(snapshot) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Are you sure you want to restore this backup '{}'?", snapshot.key))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press 'y' to confirm, 'n' to cancel")]),
            ])
            .block(Block::default().title("Confirm Restore").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Downloading(snapshot, progress, rate) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let rate_mb = *rate / 1024.0 / 1024.0;
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Downloading: {}", snapshot.key))]),
                Line::from(vec![Span::raw(format!("Progress: {:.1}% ({:.2} MB/s)", *progress * 100.0, rate_mb))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press Esc to cancel")]),
            ])
            .block(Block::default().title("Downloading").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::ConfirmCancel(snapshot, progress, rate) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let rate_mb = *rate / 1024.0 / 1024.0;
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Cancel download of: {}", snapshot.key))]),
                Line::from(vec![Span::raw(format!("Progress: {:.1}% ({:.2} MB/s)", *progress * 100.0, rate_mb))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press 'y' to confirm cancel, 'n' to continue downloading")]),
            ])
            .block(Block::default().title("Confirm Cancel").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Error(message) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::styled(
                    message,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press Esc to dismiss")]),
            ])
            .block(Block::default().title("Error").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Success(message) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::styled(
                    message,
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )]),
            ])
            .block(Block::default().title("Success").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::TestS3Result(result) => {
            let area = centered_rect(60, 7, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw("S3 Connection Test")]),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    result,
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press Esc to dismiss")]),
            ])
            .block(Block::default().title("Test Result").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::TestPgResult(result) => {
            let area = centered_rect(60, 7, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw("PostgreSQL Connection Test")]),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    result,
                    Style::default().add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press Esc to dismiss")]),
            ])
            .block(Block::default().title("Test Result").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Restoring(snapshot, progress) => {
            let area = centered_rect(60, 7, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);

            // Create a progress bar
            let _progress_percent = (*progress * 100.0) as u16;
            let progress_bar_width = 50;
            let filled_width = (progress_bar_width as f32 * *progress) as usize;
            let empty_width = progress_bar_width as usize - filled_width;

            let progress_bar = format!(
                "[{}{}] {:.1}%",
                "=".repeat(filled_width),
                " ".repeat(empty_width),
                *progress * 100.0
            );

            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Restoring database from: {}", snapshot.key))]),
                Line::from(vec![]),
                Line::from(vec![Span::styled(
                    progress_bar,
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("This operation cannot be cancelled")]),
            ])
            .block(Block::default().title("Restoring Database").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        _ => {}
    }

    if let Some(error) = &browser.s3_config.error_message {
        let error_block = Block::default()
            .title("Error")
            .borders(Borders::ALL);
        let error_paragraph = Paragraph::new(error.as_str())
            .block(error_block);
        f.render_widget(error_paragraph, chunks[3]);
    }

    // If in editing mode, render an editing indicator at the highest z-layer
    if browser.input_mode == crate::ui::models::InputMode::Editing {
        // Create a floating box for the editing field
        let mut field_area = match browser.focus {
            FocusField::Bucket => s3_settings_chunks[0],
            FocusField::Region => s3_settings_chunks[1],
            FocusField::Prefix => s3_settings_chunks[2],
            FocusField::EndpointUrl => s3_settings_chunks[3],
            FocusField::AccessKeyId => s3_settings_chunks[4],
            FocusField::SecretAccessKey => s3_settings_chunks[5],
            FocusField::PathStyle => s3_settings_chunks[6],
            FocusField::PgHost => return,
            FocusField::PgPort => return,
            FocusField::PgUsername => return,
            FocusField::PgPassword => return,
            FocusField::PgSsl => return,
            FocusField::PgDbName => return,
            _ => return,
        };

        // Make the field area slightly larger to ensure it stands out
        field_area = Rect {
            x: field_area.x.saturating_sub(1),
            y: field_area.y.saturating_sub(1),
            width: field_area.width + 2,
            height: field_area.height + 2,
        };

        // Clear a larger area around the field to ensure our input field is visible
        f.render_widget(Clear, field_area);

        // Create a floating input box with border
        let input_content = match browser.focus {
            FocusField::Bucket => browser.input_buffer.clone(),
            FocusField::Region => browser.input_buffer.clone(),
            FocusField::Prefix => browser.input_buffer.clone(),
            FocusField::EndpointUrl => browser.input_buffer.clone(),
            FocusField::AccessKeyId => browser.input_buffer.clone(),
            FocusField::SecretAccessKey => browser.input_buffer.clone(),
            FocusField::PathStyle => browser.input_buffer.clone(),
            FocusField::PgHost => return,
            FocusField::PgPort => return,
            FocusField::PgUsername => return,
            FocusField::PgPassword => return,
            FocusField::PgSsl => return,
            FocusField::PgDbName => return,
            _ => String::new(),
        };

        // Get the field label
        let field_label = match browser.focus {
            FocusField::Bucket => "Bucket",
            FocusField::Region => "Region",
            FocusField::Prefix => "Prefix",
            FocusField::EndpointUrl => "Endpoint URL",
            FocusField::AccessKeyId => "Access Key ID",
            FocusField::SecretAccessKey => "Secret Access Key",
            FocusField::PathStyle => "Path Style",
            FocusField::PgHost => return,
            FocusField::PgPort => return,
            FocusField::PgUsername => return,
            FocusField::PgPassword => return,
            FocusField::PgSsl => return,
            FocusField::PgDbName => return,
            _ => "",
        };

        // Create a more prominent input box with a double border and background color
        let input_box = Paragraph::new(input_content)
            .style(Style::default().fg(Color::Green).bg(Color::Black))
            .block(Block::default()
                .title(format!(" Editing {} ", field_label))
                .title_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Double)
                .border_style(Style::default().fg(Color::Green)));

        f.render_widget(input_box, field_area);

        // Create a small area for the editing mode indicator at the top of the screen
        let indicator_area = Rect {
            x: f.size().width - 20,
            y: 0,
            width: 20,
            height: 1,
        };

        // Render the editing indicator
        let editing_indicator = Paragraph::new("EDITING MODE")
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        f.render_widget(editing_indicator, indicator_area);
    }
}

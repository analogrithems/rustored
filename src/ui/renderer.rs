use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Clear, Table, Row, Cell},
    Frame,
};

use crate::ui::models::{FocusField, InputMode, RestoreTarget, PopupState};
use crate::ui::rustored::RustoredApp;
use crate::ui::components::popups;

/// Helper function to create a centered rect using up certain percentage of the available rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
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

/// Render PostgreSQL settings
fn render_postgres_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    let settings_block = Block::default()
        .title("PostgreSQL Settings")
        .borders(Borders::ALL);
    f.render_widget(settings_block, area);

    // Create layout for settings fields
    let settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Host
                Constraint::Length(1), // Port
                Constraint::Length(1), // Username
                Constraint::Length(1), // Password
                Constraint::Length(1), // SSL
                Constraint::Length(1), // DB Name
                Constraint::Min(1),    // Flexible space
                Constraint::Length(1), // Help legend
            ]
            .as_ref(),
        )
        .split(area);

    // Render each field
    let normal_style = Style::default().fg(Color::White);
    let focused_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    // Host field
    let host_style = if app.focus == FocusField::PgHost { focused_style } else { normal_style };
    let host_value = app.pg_config.host.clone().unwrap_or_default();
    let host_text = format!("Host: {}", host_value);
    let host_paragraph = Paragraph::new(host_text).style(host_style);
    f.render_widget(host_paragraph, settings_chunks[0]);

    // Port field
    let port_style = if app.focus == FocusField::PgPort { focused_style } else { normal_style };
    let port_value = app.pg_config.port.map_or("5432".to_string(), |p| p.to_string());
    let port_text = format!("Port: {}", port_value);
    let port_paragraph = Paragraph::new(port_text).style(port_style);
    f.render_widget(port_paragraph, settings_chunks[1]);

    // Username field
    let username_style = if app.focus == FocusField::PgUsername { focused_style } else { normal_style };
    let username_value = app.pg_config.username.clone().unwrap_or_default();
    let username_text = format!("Username: {}", username_value);
    let username_paragraph = Paragraph::new(username_text).style(username_style);
    f.render_widget(username_paragraph, settings_chunks[2]);

    // Password field
    let password_style = if app.focus == FocusField::PgPassword { focused_style } else { normal_style };
    let password_value = app.pg_config.password.clone().map_or(String::new(), |p| "*".repeat(p.len()));
    let password_text = format!("Password: {}", password_value);
    let password_paragraph = Paragraph::new(password_text).style(password_style);
    f.render_widget(password_paragraph, settings_chunks[3]);

    // SSL field
    let ssl_style = if app.focus == FocusField::PgSsl { focused_style } else { normal_style };
    let ssl_text = format!("Use SSL: {}", app.pg_config.use_ssl);
    let ssl_paragraph = Paragraph::new(ssl_text).style(ssl_style);
    f.render_widget(ssl_paragraph, settings_chunks[4]);

    // DB Name field
    let db_name_style = if app.focus == FocusField::PgDbName { focused_style } else { normal_style };
    let db_name_value = app.pg_config.db_name.clone().unwrap_or_default();
    let db_name_text = format!("Database: {}", db_name_value);
    let db_name_paragraph = Paragraph::new(db_name_text).style(db_name_style);
    f.render_widget(db_name_paragraph, settings_chunks[5]);

    // Only show PG connection test option if required fields are set
    let has_required_fields = app.pg_config.host.is_some() &&
                             app.pg_config.port.is_some() &&
                             app.pg_config.db_name.is_some();

    // Create help legend text
    let mut help_items = Vec::new();

    // Always show navigation help
    help_items.push(Span::styled("↑↓", Style::default().fg(Color::Yellow)));
    help_items.push(Span::raw(" Navigate "));

    // Show test connection option if fields are set
    if has_required_fields {
        help_items.push(Span::styled("[p]", Style::default().fg(Color::Yellow)));
        help_items.push(Span::raw(" Test Connection "));
    }

    // Create the help legend
    let help_text = Line::from(help_items);
    let help_legend = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    // Render the help legend at the bottom of the window
    f.render_widget(help_legend, settings_chunks[7]);
}

/// Render Elasticsearch settings
fn render_elasticsearch_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    let settings_block = Block::default()
        .title("Elasticsearch Settings")
        .borders(Borders::ALL);
    f.render_widget(settings_block, area);

    // Create layout for settings fields
    let settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Host
                Constraint::Length(1), // Index
            ]
            .as_ref(),
        )
        .split(area);

    // Render each field
    let normal_style = Style::default().fg(Color::White);
    let focused_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    // Host field
    let host_style = if app.focus == FocusField::EsHost { focused_style } else { normal_style };
    let host_value = app.es_config.host.clone().unwrap_or_default();
    let host_text = format!("Host: {}", host_value);
    let host_paragraph = Paragraph::new(host_text).style(host_style);
    f.render_widget(host_paragraph, settings_chunks[0]);

    // Index field
    let index_style = if app.focus == FocusField::EsIndex { focused_style } else { normal_style };
    let index_value = app.es_config.index.clone().unwrap_or_default();
    let index_text = format!("Index: {}", index_value);
    let index_paragraph = Paragraph::new(index_text).style(index_style);
    f.render_widget(index_paragraph, settings_chunks[1]);
}

/// Render Qdrant settings
fn render_qdrant_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    let settings_block = Block::default()
        .title("Qdrant Settings")
        .borders(Borders::ALL);
    f.render_widget(settings_block, area);

    // Create layout for settings fields
    let settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Host
                Constraint::Length(1), // Collection
                Constraint::Length(1), // API Key
            ]
            .as_ref(),
        )
        .split(area);

    // Render each field
    let normal_style = Style::default().fg(Color::White);
    let focused_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    // Host field (reusing EsHost focus field)
    let host_style = if app.focus == FocusField::EsHost { focused_style } else { normal_style };
    let host_value = app.qdrant_config.host.clone().unwrap_or_default();
    let host_text = format!("Host: {}", host_value);
    let host_paragraph = Paragraph::new(host_text).style(host_style);
    f.render_widget(host_paragraph, settings_chunks[0]);

    // Collection field (reusing EsIndex focus field)
    let collection_style = if app.focus == FocusField::EsIndex { focused_style } else { normal_style };
    let collection_value = app.qdrant_config.collection.clone().unwrap_or_default();
    let collection_text = format!("Collection: {}", collection_value);
    let collection_paragraph = Paragraph::new(collection_text).style(collection_style);
    f.render_widget(collection_paragraph, settings_chunks[1]);

    // API Key field
    let api_key_style = if app.focus == FocusField::QdrantApiKey { focused_style } else { normal_style };
    let api_key_value = app.qdrant_config.api_key.clone().map_or(String::new(), |k| "*".repeat(k.len()));
    let api_key_text = format!("API Key: {}", api_key_value);
    let api_key_paragraph = Paragraph::new(api_key_text).style(api_key_style);
    f.render_widget(api_key_paragraph, settings_chunks[2]);
}

/// Render the UI
pub fn ui<B: Backend>(f: &mut Frame, app: &mut RustoredApp) {
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),  // Title
                Constraint::Length(15), // S3 Settings & Restore Target (horizontal row)
                Constraint::Min(10),    // Snapshot List
            ]
            .as_ref(),
        )
        .split(f.size());

    // Render title
    let title = Paragraph::new("Rustored - S3 Snapshot Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Create horizontal layout for S3 Settings, Restore Target, and Restore Settings
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),  // S3 Settings
                Constraint::Percentage(20),  // Restore Target
                Constraint::Percentage(50),  // Restore Settings
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    // S3 Settings section
    let s3_settings_block = Block::default()
        .title("S3 Settings")
        .borders(Borders::ALL);
    f.render_widget(s3_settings_block, horizontal_chunks[0]);

    // Restore Target section
    let restore_target_block = Block::default()
        .title("Restore Target")
        .borders(Borders::ALL);
    f.render_widget(restore_target_block, horizontal_chunks[1]);

    // Create layout for restore target selection
    let restore_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Current selection
                Constraint::Length(1),  // Spacer
                Constraint::Length(1),  // Option 1
                Constraint::Length(1),  // Option 2
                Constraint::Length(1),  // Option 3
            ]
            .as_ref(),
        )
        .split(horizontal_chunks[1]);

    // Show current selection
    let current_target = match app.restore_target {
        RestoreTarget::Postgres => "PostgreSQL",
        RestoreTarget::Elasticsearch => "Elasticsearch",
        RestoreTarget::Qdrant => "Qdrant",
    };

    let current_selection = Paragraph::new(format!("Current selection: {}", current_target))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(current_selection, restore_chunks[0]);

    // Show options
    let option_style = Style::default().fg(Color::White);
    let selected_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

    let pg_style = if app.restore_target == RestoreTarget::Postgres { selected_style } else { option_style };
    let es_style = if app.restore_target == RestoreTarget::Elasticsearch { selected_style } else { option_style };
    let qdrant_style = if app.restore_target == RestoreTarget::Qdrant { selected_style } else { option_style };

    let pg_option = Paragraph::new("1. PostgreSQL")
        .style(pg_style);
    let es_option = Paragraph::new("2. Elasticsearch")
        .style(es_style);
    let qdrant_option = Paragraph::new("3. Qdrant")
        .style(qdrant_style);

    f.render_widget(pg_option, restore_chunks[2]);
    f.render_widget(es_option, restore_chunks[3]);
    f.render_widget(qdrant_option, restore_chunks[4]);

    // Create a block for the restore settings
    let restore_settings_block = Block::default()
        .title(match app.restore_target {
            RestoreTarget::Postgres => "PostgreSQL Settings",
            RestoreTarget::Elasticsearch => "Elasticsearch Settings",
            RestoreTarget::Qdrant => "Qdrant Settings",
        })
        .borders(Borders::ALL);
    f.render_widget(restore_settings_block, horizontal_chunks[2]);

    // Display settings for the selected restore target
    let settings_area = horizontal_chunks[2];
    match app.restore_target {
        RestoreTarget::Postgres => render_postgres_settings::<B>(f, app, settings_area),
        RestoreTarget::Elasticsearch => render_elasticsearch_settings::<B>(f, app, settings_area),
        RestoreTarget::Qdrant => render_qdrant_settings::<B>(f, app, settings_area),
    };

    // Create layout for S3 settings fields
    let s3_settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1), // Bucket
                Constraint::Length(1), // Region
                Constraint::Length(1), // Prefix
                Constraint::Length(1), // Endpoint URL
                Constraint::Length(1), // Access Key ID
                Constraint::Length(1), // Secret Access Key
                Constraint::Length(1), // Path Style
                Constraint::Min(1),    // Flexible space
                Constraint::Length(1), // Help legend
            ]
            .as_ref(),
        )
        .split(horizontal_chunks[0]);

    // Render each S3 setting field
    let field_labels = [
        "Bucket: ",
        "Region: ",
        "Prefix: ",
        "Endpoint URL: ",
        "Access Key ID: ",
        "Secret Access Key: ",
        "Path Style: ",
    ];

    let field_values = [
        &app.s3_config.bucket,
        &app.s3_config.region,
        &app.s3_config.prefix,
        &app.s3_config.endpoint_url,
        &app.s3_config.access_key_id,
        &app.s3_config.secret_access_key,
        &app.s3_config.path_style.to_string(),
    ];

    let field_focus = [
        FocusField::Bucket,
        FocusField::Region,
        FocusField::Prefix,
        FocusField::EndpointUrl,
        FocusField::AccessKeyId,
        FocusField::SecretAccessKey,
        FocusField::PathStyle,
    ];

    for i in 0..field_labels.len() {
        let style = if app.focus == field_focus[i] {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let content = if app.input_mode == InputMode::Editing && app.focus == field_focus[i] {
            format!("{}[{}]", field_labels[i], app.input_buffer)
        } else {
            format!("{}{}", field_labels[i], field_values[i])
        };

        let paragraph = Paragraph::new(content)
            .style(style);

        f.render_widget(paragraph, s3_settings_chunks[i]);
    }

    // Only show S3 connection test option if required fields are set
    let has_required_fields = !app.s3_config.bucket.is_empty() &&
                            !app.s3_config.access_key_id.is_empty() &&
                            !app.s3_config.secret_access_key.is_empty();

    // Create help legend text
    let mut help_items = Vec::new();

    // Always show navigation help
    help_items.push(Span::styled("↑↓", Style::default().fg(Color::Yellow)));
    help_items.push(Span::raw(" Navigate "));

    // Show test connection option if fields are set
    if has_required_fields {
        help_items.push(Span::styled("[t]", Style::default().fg(Color::Yellow)));
        help_items.push(Span::raw(" Test Connection "));
    }

    // Create the help legend
    let help_text = Line::from(help_items);
    let help_legend = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    // Render the help legend at the bottom of the window
    f.render_widget(help_legend, s3_settings_chunks[8]);

    // Snapshot Browser section
    let snapshots_block = Block::default()
        .title("Snapshots")
        .borders(Borders::ALL);
    f.render_widget(snapshots_block, chunks[2]);

    // Render snapshot list
    if !app.snapshot_browser.snapshots.is_empty() {
        let snapshot_list_area = Rect {
            x: chunks[2].x + 1,
            y: chunks[2].y + 1,
            width: chunks[2].width - 2,
            height: chunks[2].height - 2,
        };

        // Create table rows from snapshots
        let visible_snapshots = if app.snapshot_browser.snapshots.len() > snapshot_list_area.height as usize {
            // Calculate which snapshots to display based on selected index
            let selected = app.snapshot_browser.selected_index;
            let list_height = snapshot_list_area.height as usize;
            let half_height = list_height / 2;
            let total_items = app.snapshot_browser.snapshots.len();

            let start_idx = if selected < half_height {
                0
            } else if selected >= total_items - half_height {
                total_items - list_height
            } else {
                selected - half_height
            };

            let end_idx = std::cmp::min(start_idx + list_height, total_items);
            &app.snapshot_browser.snapshots[start_idx..end_idx]
        } else {
            &app.snapshot_browser.snapshots
        };

        // Create rows for the table
        let rows: Vec<Row> = visible_snapshots
            .iter()
            .enumerate()
            .map(|(i, snapshot)| {
                // Convert AWS DateTime to chrono DateTime
                let timestamp = snapshot.last_modified;
                let dt = chrono::DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
                let formatted_date = dt.format("%Y-%m-%d %H:%M:%S").to_string();
                let size_mb = snapshot.size as f64 / 1024.0 / 1024.0;
                let formatted_size = format!("{:.2} MB", size_mb);

                // Use the full S3 path
                let full_path = &snapshot.key;

                // Apply style to the selected row
                let is_selected = i + (visible_snapshots.as_ptr() as usize - app.snapshot_browser.snapshots.as_ptr() as usize) / std::mem::size_of::<crate::ui::models::BackupMetadata>() == app.snapshot_browser.selected_index;
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    Cell::from(full_path.to_string()).style(style),
                    Cell::from(formatted_size).style(style),
                    Cell::from(formatted_date).style(style),
                ])
            })
            .collect();

        // Create header row
        let header_style = Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD);
        let header = Row::new(vec![
            Cell::from("S3 Path").style(header_style),
            Cell::from("Size").style(header_style),
            Cell::from("Last Modified").style(header_style),
        ]);

        // Create table with header and rows
        let mut table_rows = vec![header];
        table_rows.extend(rows);

        let table = Table::new(table_rows, &[
                Constraint::Percentage(50),  // Filename takes 50% of the width
                Constraint::Percentage(15),  // Size takes 15% of the width
                Constraint::Percentage(35),  // Date takes 35% of the width
            ])
            .column_spacing(1);

        f.render_widget(table, snapshot_list_area);
    } else {
        let no_snapshots_msg = Paragraph::new("No snapshots found")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        let msg_area = Rect {
            x: chunks[3].x + 1,
            y: chunks[3].y + chunks[3].height / 2,
            width: chunks[3].width - 2,
            height: 1,
        };

        f.render_widget(no_snapshots_msg, msg_area);
    }

    // Show help text at the bottom
    let help_text = match app.input_mode {
        InputMode::Normal => "Press 'q' to quit, Enter to edit, 'r' to refresh, Tab to navigate, 1-3 to select restore target, Ctrl+Z to suspend",
        InputMode::Editing => "Press Esc to cancel, Enter to save",
    };
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    let help_rect = Rect::new(
        chunks[2].x,
        chunks[2].y + chunks[2].height - 1,
        chunks[2].width,
        1,
    );
    f.render_widget(help_paragraph, help_rect);

    // Handle popups if needed
    if app.popup_state != PopupState::Hidden {
        // For now, we'll just show a simple popup
        let area = centered_rect(60, 5, f.size());
        f.render_widget(Clear, area);

        match &app.popup_state {
            PopupState::ConfirmRestore(snapshot) => {
                let popup = Paragraph::new(format!("Restore from {}? (y/n)", snapshot.key))
                    .block(Block::default().title("Confirm Restore").borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::Downloading(snapshot, progress, rate) => {
                // Create a progress bar
                let progress_bar_width = 50;
                let filled_width = (progress_bar_width as f32 * progress) as usize;
                let empty_width = progress_bar_width as usize - filled_width;

                let progress_bar = format!(
                    "[{}{}] {:.1}%",
                    "=".repeat(filled_width),
                    " ".repeat(empty_width),
                    *progress as f32 * 100.0
                );

                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::raw(format!("Downloading: {}", snapshot.key))]),
                    Line::from(vec![]),
                    Line::from(vec![Span::styled(
                        progress_bar,
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw(format!("Speed: {:.2} MB/s", rate / 1024.0 / 1024.0))]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Press Esc to cancel")]),
                ])
                .block(Block::default().title("Downloading").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::ConfirmCancel(snapshot, progress, _) => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::raw(format!("Cancel download of {}?", snapshot.key))]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw(format!("Progress: {:.1}%", *progress as f32 * 100.0))]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Press y to confirm, n to continue downloading")]),
                ])
                .block(Block::default().title("Confirm Cancel").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::Error(message) => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "Error",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw(message)]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Press Esc to dismiss")]),
                ])
                .block(Block::default().title("Error").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::Success(message) => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "Success",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw(message)]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Press Esc to dismiss")]),
                ])
                .block(Block::default().title("Success").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::TestingS3 => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "Testing S3 Connection...",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Please wait")]),
                ])
                .block(Block::default().title("S3 Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::TestS3Result(result) => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "S3 Connection Test Result",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw(result)]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Press Esc to dismiss")]),
                ])
                .block(Block::default().title("S3 Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::TestingPg => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "Testing PostgreSQL Connection...",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Please wait")]),
                ])
                .block(Block::default().title("PostgreSQL Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            PopupState::TestPgResult(result) => {
                let popup = Paragraph::new(vec![
                    Line::from(vec![Span::styled(
                        "PostgreSQL Connection Test Result",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw(result)]),
                    Line::from(vec![]),
                    Line::from(vec![Span::raw("Press Esc to dismiss")]),
                ])
                .block(Block::default().title("PostgreSQL Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
                f.render_widget(popup, area);
            },
            _ => {
                let popup = Paragraph::new("Popup content would go here")
                    .block(Block::default().title("Popup").borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(popup, area);
            }
        }
    }

    if let Some(error) = &app.s3_config.error_message {
        let error_block = Block::default()
            .title("Error")
            .borders(Borders::ALL);
        let error_paragraph = Paragraph::new(error.as_str())
            .block(error_block);
        f.render_widget(error_paragraph, chunks[3]);
    }

    // If in editing mode, show an indicator
    if app.input_mode == InputMode::Editing {
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

    // Render popups if needed
    popups::render_popups::<B>(f, app);
}
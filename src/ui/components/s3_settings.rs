use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use log::debug;
use crate::ui::models::{FocusField, InputMode};
use crate::ui::rustored::RustoredApp;

/// Render S3 settings section
pub fn render_s3_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    debug!("Starting to render S3 settings in area: {:?}", area);
    debug!("Rendering S3 settings with focus: {:?}", app.focus);
    debug!("Rendering S3 settings with input mode: {:?}", app.input_mode);
    // S3 Settings
    let s3_settings_block = Block::default()
        .title("S3 Settings")
        .borders(Borders::ALL);
    f.render_widget(s3_settings_block, area);

    // Create layout for S3 settings fields
    // As per TDD rule #10, navigation help text should be at the bottom
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
                Constraint::Length(1), // Help legend (at the bottom as per TDD rule #10)
            ]
            .as_ref(),
        )
        .split(area);
    
    debug!("Created S3 settings layout with navigation help at the bottom (TDD rule #10)");

    // Bucket
    let bucket_style = if app.focus == FocusField::Bucket {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let bucket_text = if app.focus == FocusField::Bucket && app.input_mode == InputMode::Editing {
        format!("Bucket: {}", app.input_buffer)
    } else {
        format!("Bucket: {}", app.s3_config.bucket)
    };

    let bucket = Paragraph::new(bucket_text)
        .style(bucket_style);
    f.render_widget(bucket, s3_settings_chunks[0]);

    // Region
    let region_style = if app.focus == FocusField::Region {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let region_text = if app.focus == FocusField::Region && app.input_mode == InputMode::Editing {
        format!("Region: {}", app.input_buffer)
    } else {
        format!("Region: {}", app.s3_config.region)
    };

    let region = Paragraph::new(region_text)
        .style(region_style);
    f.render_widget(region, s3_settings_chunks[1]);

    // Prefix
    let prefix_style = if app.focus == FocusField::Prefix {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let prefix_text = if app.focus == FocusField::Prefix && app.input_mode == InputMode::Editing {
        format!("Prefix: {}", app.input_buffer)
    } else {
        format!("Prefix: {}", app.s3_config.prefix)
    };

    let prefix = Paragraph::new(prefix_text)
        .style(prefix_style);
    f.render_widget(prefix, s3_settings_chunks[2]);

    // Endpoint URL
    let endpoint_url_style = if app.focus == FocusField::EndpointUrl {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let endpoint_url_text = if app.focus == FocusField::EndpointUrl && app.input_mode == InputMode::Editing {
        format!("Endpoint URL: {}", app.input_buffer)
    } else {
        format!("Endpoint URL: {}", app.s3_config.endpoint_url)
    };

    let endpoint_url = Paragraph::new(endpoint_url_text)
        .style(endpoint_url_style);
    f.render_widget(endpoint_url, s3_settings_chunks[3]);

    // Access Key ID
    let access_key_style = if app.focus == FocusField::AccessKeyId {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let access_key_text = if app.focus == FocusField::AccessKeyId && app.input_mode == InputMode::Editing {
        format!("Access Key ID: {}", app.input_buffer)
    } else {
        format!("Access Key ID: {}", app.s3_config.masked_access_key())
    };

    let access_key = Paragraph::new(access_key_text)
        .style(access_key_style);
    f.render_widget(access_key, s3_settings_chunks[4]);

    // Secret Access Key
    let secret_key_style = if app.focus == FocusField::SecretAccessKey {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let is_editing = app.focus == FocusField::SecretAccessKey && app.input_mode == InputMode::Editing;
    let secret_key_text = app.s3_config.get_secret_key_display(is_editing, &app.input_buffer);

    let secret_key = Paragraph::new(secret_key_text)
        .style(secret_key_style);
    f.render_widget(secret_key, s3_settings_chunks[5]);

    // Path Style
    let path_style_style = if app.focus == FocusField::PathStyle {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let path_style_text = format!("Path Style: {}", app.s3_config.path_style);
    let path_style = Paragraph::new(path_style_text)
        .style(path_style_style);
    f.render_widget(path_style, s3_settings_chunks[6]);


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

    // Create the help legend and place it at the bottom as per TDD rule #10
    let help_text = Line::from(help_items);
    let help_legend = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Left);
    
    debug!("Rendering navigation help text at the bottom of S3 settings (TDD rule #10)");
    f.render_widget(help_legend, s3_settings_chunks[8]);
    
    debug!("Finished rendering S3 settings");
}

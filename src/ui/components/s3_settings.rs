use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::models::{FocusField, InputMode};
use crate::ui::rustored::RustoredApp;

/// Render S3 settings section
pub fn render_s3_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    // S3 Settings
    let s3_settings_block = Block::default()
        .title("S3 Settings")
        .borders(Borders::ALL);
    f.render_widget(s3_settings_block, area);

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
            ]
            .as_ref(),
        )
        .split(area);

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
    let endpoint_style = if app.focus == FocusField::EndpointUrl {
        if app.input_mode == InputMode::Editing {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default()
    };

    let endpoint_text = if app.focus == FocusField::EndpointUrl && app.input_mode == InputMode::Editing {
        format!("Endpoint URL: {}", app.input_buffer)
    } else {
        format!("Endpoint URL: {}", app.s3_config.endpoint_url)
    };

    let endpoint = Paragraph::new(endpoint_text)
        .style(endpoint_style);
    f.render_widget(endpoint, s3_settings_chunks[3]);

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

    let secret_key_text = if app.focus == FocusField::SecretAccessKey && app.input_mode == InputMode::Editing {
        format!("Secret Access Key: {}", app.input_buffer)
    } else {
        format!("Secret Access Key: {}", app.s3_config.masked_secret_key())
    };

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
}

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
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

    // Create a layout for the settings fields
    // As per TDD rule #10, navigation help text should be at the bottom
    let s3_settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(1),    // Main content area (table)
                Constraint::Length(1), // Help legend (at the bottom as per TDD rule #10)
            ]
            .as_ref(),
        )
        .split(area);
    
    debug!("Created S3 settings layout with 2 sections: content and help text");

    // Prepare the table rows for S3 settings
    let mut rows = Vec::new();
    
    // Define the fields to display
    let mut fields = Vec::new();
    
    // Bucket field
    let bucket_value = if app.focus == FocusField::Bucket && app.input_mode == InputMode::Editing {
        app.input_buffer.clone()
    } else {
        app.s3_config.bucket.clone()
    };
    fields.push(("Bucket", bucket_value, FocusField::Bucket));
    
    // Region field
    let region_value = if app.focus == FocusField::Region && app.input_mode == InputMode::Editing {
        app.input_buffer.clone()
    } else {
        app.s3_config.region.clone()
    };
    fields.push(("Region", region_value, FocusField::Region));
    
    // Prefix field
    let prefix_value = if app.focus == FocusField::Prefix && app.input_mode == InputMode::Editing {
        app.input_buffer.clone()
    } else {
        app.s3_config.prefix.clone()
    };
    fields.push(("Prefix", prefix_value, FocusField::Prefix));
    
    // Endpoint URL field
    let endpoint_value = if app.focus == FocusField::EndpointUrl && app.input_mode == InputMode::Editing {
        app.input_buffer.clone()
    } else {
        app.s3_config.endpoint_url.clone()
    };
    fields.push(("Endpoint URL", endpoint_value, FocusField::EndpointUrl));
    
    // Access Key ID field
    let access_key_value = if app.focus == FocusField::AccessKeyId && app.input_mode == InputMode::Editing {
        app.input_buffer.clone()
    } else {
        app.s3_config.access_key_id.clone()
    };
    fields.push(("Access Key ID", access_key_value, FocusField::AccessKeyId));
    
    // Secret Access Key field (with masking as per TDD rule #12)
    let is_editing = app.focus == FocusField::SecretAccessKey && app.input_mode == InputMode::Editing;
    let secret_key_value = app.s3_config.get_secret_key_display(is_editing, &app.input_buffer);
    fields.push(("Secret Access Key", secret_key_value.replace("Secret Access Key: ", ""), FocusField::SecretAccessKey));
    
    // Path Style field
    fields.push(("Path Style", app.s3_config.path_style.to_string(), FocusField::PathStyle));
    
    debug!("Created S3 settings fields for table layout");
    
    // Create a row for each field
    for (label, value, field) in &fields {
        // Determine if this field is focused
        let is_focused = app.focus == *field;
        
        // Style for the label
        let label_style = Style::default().fg(Color::Blue);
        
        // Style for the value - highlight if focused
        let value_style = if is_focused {
            if app.input_mode == InputMode::Editing {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            }
        } else {
            Style::default().fg(Color::White)
        };
        
        // Create the row with styled cells
        let row = Row::new(vec![
            Cell::from(label.to_string()).style(label_style),
            Cell::from(value.clone()).style(value_style),
        ]);
        
        rows.push(row);
    }
    
    // Create and render the table
    let table = Table::new(
        rows,
        [Constraint::Percentage(30), Constraint::Percentage(70)]
    )
    .column_spacing(1)
    .style(Style::default())
    .header(Row::new(vec![
        Cell::from(Span::styled("Setting", Style::default().add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Value", Style::default().add_modifier(Modifier::BOLD)))
    ]));
    
    // Render the table inside the block's inner area
    f.render_widget(table, s3_settings_chunks[0]);
    
    debug!("Rendered S3 settings using table layout");

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
    f.render_widget(help_legend, s3_settings_chunks[1]);
    
    debug!("Finished rendering S3 settings");
}

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
    Frame,
};

use log::debug;
use crate::ui::models::{FocusField, InputMode};
use crate::ui::rustored::RustoredApp;

/// Render Qdrant settings component
/// 
/// This function is responsible for rendering the Qdrant settings UI component.
/// It displays all Qdrant connection parameters and highlights the currently focused field.
/// 
/// # Arguments
/// 
/// * `f` - A mutable reference to the frame for rendering
/// * `app` - A reference to the application state
/// * `area` - The area in which to render the component
pub fn render_qdrant_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    // Log the start of rendering Qdrant settings
    debug!("Starting to render Qdrant settings in area: {:?}", area);
    // Log the rendering of Qdrant settings
    debug!("Rendering Qdrant settings in area: {:?}", area);

    // Create a block for the Qdrant settings
    let block = Block::default()
        .title(" Qdrant Settings ")
        .borders(Borders::ALL)
        .style(Style::default());

    // Create a layout for the settings fields
    // As per TDD rule #10, navigation help text should be at the bottom
    let inner_area = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Settings fields (use all remaining space)
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Help text at the bottom (TDD rule #10)
        ])
        .split(inner_area);
    
    debug!("Created Qdrant settings layout with navigation help at the bottom (TDD rule #10)");

    // Help text will be rendered at the bottom as per TDD rule #10

    // Prepare the table rows for Qdrant settings
    let mut rows = Vec::new();

    // Define the fields to display
    let mut fields = Vec::new();
    
    // Add standard fields
    fields.push(("Host", app.qdrant_config.host.clone().unwrap_or_default(), FocusField::EsHost));
    fields.push(("Collection", app.qdrant_config.collection.clone().unwrap_or_default(), FocusField::EsIndex));
    
    // Handle API key field with masking as per TDD rule #12
    let api_key_value = if app.focus == FocusField::QdrantApiKey && app.input_mode == InputMode::Editing {
        // Show actual API key only when editing
        app.qdrant_config.api_key.clone().unwrap_or_default()
    } else {
        // Mask API key when not editing
        if app.qdrant_config.api_key.clone().unwrap_or_default().is_empty() {
            "".to_string()
        } else {
            "[hidden]".to_string() // Masked API key using [hidden] to match S3 settings
        }
    };
    debug!("Applied [hidden] masking for Qdrant API key (consistent with S3 settings)");
    fields.push(("API Key", api_key_value, FocusField::QdrantApiKey));
    
    debug!("Applied masking for Qdrant API key field (TDD rule #12)");

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
    // Create the table with rows and column widths
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

    // Render the block first to create the border
    f.render_widget(block, area);
    // Then render the table inside the block's inner area
    f.render_widget(table, chunks[0]);
    
    // Render the help text at the bottom as per TDD rule #10
    let mut help_items = Vec::new();
    
    // Always show navigation help (using the same format as S3 settings)
    help_items.push(Span::styled("↑↓", Style::default().fg(Color::Yellow)));
    help_items.push(Span::raw(" Navigate "));
    
    // Show test connection option (using [t] consistently across all components)
    help_items.push(Span::styled("[t]", Style::default().fg(Color::Yellow)));
    help_items.push(Span::raw(" Test Connection "));
    
    let help_text = Line::from(help_items);
    
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    
    debug!("Rendering navigation help text at the bottom of Qdrant settings (TDD rule #10)");
    f.render_widget(help, chunks[2]);
    
    debug!("Finished rendering Qdrant settings");
}

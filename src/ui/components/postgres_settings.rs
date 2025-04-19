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

/// Render PostgreSQL settings component
/// 
/// This function is responsible for rendering the PostgreSQL settings UI component.
/// It displays all PostgreSQL connection parameters and highlights the currently focused field.
/// 
/// # Arguments
/// 
/// * `f` - A mutable reference to the frame for rendering
/// * `app` - A reference to the application state
/// * `area` - The area in which to render the component
pub fn render_postgres_settings<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    log::debug!("Starting to render PostgreSQL settings in area: {:?}", area);
    log::debug!("Rendering PostgreSQL settings with focus: {:?}", app.focus);
    log::debug!("Rendering PostgreSQL settings with input mode: {:?}", app.input_mode);
    // Log the rendering of PostgreSQL settings
    debug!("Rendering PostgreSQL settings in area: {:?}", area);

    // Create a block for the PostgreSQL settings
    let block = Block::default()
        .title(" PostgreSQL Settings ")
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
    
    debug!("Created PostgreSQL settings layout with navigation help at the bottom (TDD rule #10)");

    // Prepare the table rows for PostgreSQL settings
    let mut rows = Vec::new();

    // Define the fields to display
    let mut fields = Vec::new();
    
    // Add standard fields
    fields.push(("Host", app.pg_config.host.clone().unwrap_or_default(), FocusField::PgHost));
    fields.push(("Port", app.pg_config.port.map_or_else(|| "".to_string(), |p| p.to_string()), FocusField::PgPort));
    fields.push(("Username", app.pg_config.username.clone().unwrap_or_default(), FocusField::PgUsername));
    
    // Handle password field with masking as per TDD rule #12
    let password_value = if app.focus == FocusField::PgPassword && app.input_mode == InputMode::Editing {
        // Show actual password only when editing
        app.pg_config.password.clone().unwrap_or_default()
    } else {
        // Mask password when not editing
        if app.pg_config.password.clone().unwrap_or_default().is_empty() {
            "".to_string()
        } else {
            "[hidden]".to_string() // Masked password using [hidden] to match S3 settings
        }
    };
    debug!("Applied [hidden] masking for PostgreSQL password (consistent with S3 settings)");
    fields.push(("Password", password_value, FocusField::PgPassword));
    
    // Add remaining fields
    fields.push(("Database", app.pg_config.db_name.clone().unwrap_or_default(), FocusField::PgDbName));
    fields.push(("Use SSL", if app.pg_config.use_ssl { "Yes" } else { "No" }.to_string(), FocusField::PgDbName));
    
    debug!("Applied password masking for PostgreSQL password field (TDD rule #12)");

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
    
    debug!("Rendering navigation help text at the bottom of PostgreSQL settings (TDD rule #10)");
    f.render_widget(help, chunks[2]);
    
    debug!("Finished rendering PostgreSQL settings");
}

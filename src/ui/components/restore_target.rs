use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use log::debug;
use crate::ui::models::{FocusField, RestoreTarget};
use crate::ui::rustored::RustoredApp;

/// Render restore target section
/// 
/// This function is responsible for rendering the restore target selection UI component.
/// It displays the available restore targets with numeric prefixes as per TDD rule #11.
/// 
/// # Arguments
/// 
/// * `f` - A mutable reference to the frame for rendering
/// * `app` - A reference to the application state
/// * `area` - The area in which to render the component
pub fn render_restore_target<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    debug!("Starting to render restore target selection in area: {:?}", area);
    // Restore Target
    let restore_target_block = Block::default()
        .title("Restore Target")
        .borders(Borders::ALL);
    f.render_widget(restore_target_block, area);

    // Create layout for restore target list and help text
    // As per TDD rule #10, navigation help text should be at the bottom
    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(3),    // Restore Target List
                Constraint::Length(1), // Help text at the bottom
            ]
            .as_ref(),
        )
        .split(area);

    // Create list items for restore targets with numeric prefixes as per TDD rule #11
    // Each target will be on its own line as requested
    let restore_target_names = vec!["1. PostgreSQL", "2. Elasticsearch", "3. Qdrant"];
    debug!("Created restore targets with numeric prefixes: {:?}", restore_target_names);
    
    let restore_target_index = match app.restore_target {
        RestoreTarget::Postgres => 0,
        RestoreTarget::Elasticsearch => 1,
        RestoreTarget::Qdrant => 2,
    };
    debug!("Current restore target index: {}", restore_target_index);

    // Create list items with appropriate styling
    let items: Vec<ListItem> = restore_target_names
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let style = if i == restore_target_index {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if app.focus == FocusField::RestoreTarget {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(Span::styled(name, style)))
        })
        .collect();
    
    // Create the list widget without an additional border
    let restore_target_list = List::new(items);
    
    debug!("Rendering restore target list with selected index: {}", restore_target_index);
    f.render_widget(restore_target_list, inner_area[0]);
    
    // Add help text at the bottom of the restore target section as per TDD rule #10
    let help_text = Line::from(vec![
        Span::styled("Press ", Style::default()),
        Span::styled("1-3", Style::default().fg(Color::Yellow)),
        Span::styled(" to select restore target type", Style::default()),
    ]);
    
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default())
        .alignment(ratatui::layout::Alignment::Left);
    
    debug!("Rendering help text at the bottom of restore target section (TDD rule #10)");
    f.render_widget(help_paragraph, inner_area[1]);
    
    debug!("Finished rendering restore target selection");
}

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
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

    // Create layout for restore target and connection fields
    let restore_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Restore Target Tabs
                Constraint::Min(4),    // Connection Fields
            ]
            .as_ref(),
        )
        .split(area);

    // Restore Target Tabs with numeric prefixes as per TDD rule #11
    let restore_targets = vec!["1. PostgreSQL", "2. Elasticsearch", "3. Qdrant"];
    debug!("Created restore targets with numeric prefixes: {:?}", restore_targets);
    
    let restore_target_index = match app.restore_target {
        RestoreTarget::Postgres => 0,
        RestoreTarget::Elasticsearch => 1,
        RestoreTarget::Qdrant => 2,
    };
    debug!("Current restore target index: {}", restore_target_index);

    let restore_target_style = if app.focus == FocusField::RestoreTarget {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let restore_target_tabs = Tabs::new(restore_targets)
        .block(Block::default().title(" Restore Target "))
        .select(restore_target_index)
        .style(Style::default())
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .style(restore_target_style);
    
    debug!("Rendering restore target tabs with selected index: {}", restore_target_index);
    f.render_widget(restore_target_tabs, restore_chunks[0]);
    
    debug!("Finished rendering restore target selection");
}

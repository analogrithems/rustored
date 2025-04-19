use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::ui::models::{FocusField, RestoreTarget};
use crate::ui::rustored::RustoredApp;

/// Render restore target section
pub fn render_restore_target<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
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

    // Restore Target Tabs
    let restore_targets = vec!["PostgreSQL", "Elasticsearch", "Qdrant"];
    let restore_target_index = match app.restore_target {
        RestoreTarget::Postgres => 0,
        RestoreTarget::Elasticsearch => 1,
        RestoreTarget::Qdrant => 2,
    };

    let restore_target_style = if app.focus == FocusField::RestoreTarget {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let restore_target_tabs = Tabs::new(restore_targets)
        .block(Block::default())
        .select(restore_target_index)
        .style(Style::default())
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .style(restore_target_style);
    f.render_widget(restore_target_tabs, restore_chunks[0]);
}

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use chrono::{DateTime, Utc};

use crate::ui::models::FocusField;
use crate::ui::rustored::RustoredApp;

/// Render snapshot list section
pub fn render_snapshot_list<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    // Snapshot List
    let snapshot_style = if app.focus == FocusField::SnapshotList {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let snapshot_block = Block::default()
        .title("Snapshots")
        .borders(Borders::ALL)
        .style(snapshot_style);

    let snapshot_items: Vec<ListItem> = app.snapshot_browser.snapshots
        .iter()
        .enumerate()
        .map(|(i, snapshot)| {
            // Convert AWS DateTime to chrono DateTime
            let timestamp = snapshot.last_modified;
            let dt: DateTime<Utc> = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
            let formatted_date = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            let size_mb = snapshot.size as f64 / 1024.0 / 1024.0;
            let content = format!("{} - {:.2} MB - {}", snapshot.key, size_mb, formatted_date);
            let style = if i == app.snapshot_browser.selected_index {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![Span::styled(content, style)]))
        })
        .collect();

    let snapshot_list = List::new(snapshot_items)
        .block(snapshot_block);
    f.render_widget(snapshot_list, area);
}

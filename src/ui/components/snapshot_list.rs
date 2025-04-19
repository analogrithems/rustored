use ratatui::{
    backend::Backend,
    layout::{Rect, Constraint},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Table, Row, Cell},
    Frame,
};
use chrono::{DateTime, Utc};
use log::debug;

use crate::ui::models::FocusField;
use crate::ui::rustored::RustoredApp;

/// Render snapshot list section
pub fn render_snapshot_list<B: Backend>(f: &mut Frame, app: &RustoredApp, area: Rect) {
    debug!("Starting to render snapshot list in area: {:?}", area);
    debug!("Current focus: {:?}, snapshot count: {}", app.focus, app.snapshot_browser.snapshots.len());
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

    // Create table rows from snapshots
    let rows: Vec<Row> = app.snapshot_browser.snapshots
        .iter()
        .enumerate()
        .map(|(i, snapshot)| {
            // Convert AWS DateTime to chrono DateTime
            let timestamp = snapshot.last_modified;
            let dt: DateTime<Utc> = DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_default();
            let formatted_date = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            let size_mb = snapshot.size as f64 / 1024.0 / 1024.0;
            let formatted_size = format!("{:.2} MB", size_mb);
            
            // Use the full S3 path as per TDD rule #8
            let full_path = &snapshot.key;
            debug!("Using full S3 path: {}", full_path);
            
            // Apply style to the selected row
            let style = if i == app.snapshot_browser.selected_index {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
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
    
    // Create header row with 'S3 Path' as per TDD rule #8
    let header_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("S3 Path").style(header_style),
        Cell::from("Size").style(header_style),
        Cell::from("Last Modified").style(header_style),
    ]);
    debug!("Created header row with S3 Path column");
    
    // Create table with header and rows
    let mut table_rows = vec![header];
    table_rows.extend(rows);
    
    let table = Table::new(table_rows, &[
            Constraint::Percentage(50),  // S3 Path takes 50% of the width
            Constraint::Percentage(15),  // Size takes 15% of the width
            Constraint::Percentage(35),  // Date takes 35% of the width
        ])
        .block(snapshot_block)
        .column_spacing(1);
        
    f.render_widget(table, area);
}

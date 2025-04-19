use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use log::debug;
use crate::ui::models::{RestoreTarget, PopupState};
use crate::ui::rustored::RustoredApp;
use crate::ui::components::{popups, postgres_settings, elasticsearch_settings, qdrant_settings, s3_settings, snapshot_list, restore_target};

/// Helper function to create a centered rect using up certain percentage of the available rect
/// 
/// This function calculates a rectangle that is centered within the provided rectangle
/// and takes up the specified percentage of width and height.
/// 
/// # Arguments
/// 
/// * `percent_x` - The percentage of width the centered rectangle should take up
/// * `percent_y` - The percentage of height the centered rectangle should take up
/// * `r` - The base rectangle within which to center the new rectangle
/// 
/// # Returns
/// 
/// A new `Rect` that is centered within `r` with the specified dimensions
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Log the centering operation
    debug!("Creating centered rect with {}% width, {}% height in area: {:?}", percent_x, percent_y, r);
    
    // Create a vertical layout to position the center row
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2), // Top space
                Constraint::Percentage(percent_y),             // Center row
                Constraint::Percentage((100 - percent_y) / 2), // Bottom space
            ]
            .as_ref(),
        )
        .split(r);

    // Create a horizontal layout in the center row to position the centered rect
    let centered = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2), // Left space
                Constraint::Percentage(percent_x),             // Center column
                Constraint::Percentage((100 - percent_x) / 2), // Right space
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1];
        
    debug!("Centered rect result: {:?}", centered);
    centered
}

/// Render the UI
/// 
/// This function is the main entry point for rendering the entire UI.
/// It coordinates the layout and rendering of all UI components based on the current application state.
/// 
/// # Arguments
/// 
/// * `f` - A mutable reference to the frame for rendering
/// * `app` - A mutable reference to the application state
pub fn ui<B: Backend>(f: &mut Frame, app: &mut RustoredApp) {
    // Log the start of UI rendering
    debug!("Starting UI rendering");
    
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(f.size());

    // Render title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Rustored ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("- S3 Snapshot Restore Tool"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));
    
    f.render_widget(title, chunks[0]);

    // Create vertical layout for the main content - split into top and bottom rows
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Top row (S3 settings, restore target selection, specific settings)
            Constraint::Min(0),     // Bottom row (snapshot list) - takes remaining space
        ])
        .split(chunks[1]);
    
    // Create horizontal layout for the top row
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // S3 settings
            Constraint::Percentage(30), // Restore target selection
            Constraint::Percentage(40), // Specific restore target settings
        ])
        .split(main_chunks[0]);

    // Render S3 settings on the left of the top row
    debug!("Rendering S3 settings on the left of the top row");
    s3_settings::render_s3_settings::<B>(f, app, top_row[0]);
    
    // Render restore target selection tabs in the middle of the top row
    debug!("Rendering restore target selection in the middle of the top row");
    restore_target::render_restore_target::<B>(f, app, top_row[1]);

    // Render specific restore target settings on the right of the top row
    debug!("Rendering specific restore target settings on the right of the top row");
    // Render the appropriate settings panel based on the selected restore target
    // Note: We've covered all possible variants of RestoreTarget enum, so no catch-all is needed
    match app.restore_target {
        RestoreTarget::Postgres => {
            debug!("Rendering PostgreSQL settings panel");
            postgres_settings::render_postgres_settings(f, app, top_row[2]);
        },
        RestoreTarget::Elasticsearch => {
            debug!("Rendering Elasticsearch settings panel");
            elasticsearch_settings::render_elasticsearch_settings(f, app, top_row[2]);
        },
        RestoreTarget::Qdrant => {
            debug!("Rendering Qdrant settings panel");
            qdrant_settings::render_qdrant_settings(f, app, top_row[2]);
        },
    };
    
    // For now, render snapshot list taking up the entire bottom row
    // Later, when a snapshot is selected, we'll split this row to show the restore window
    debug!("Rendering snapshot list in the bottom row");
    snapshot_list::render_snapshot_list::<B>(f, app, main_chunks[1]);

    // Render status bar
    let status = format!("Press 'q' to quit | Tab to switch focus | 1-3 to change restore target | Current focus: {:?}", app.focus);
    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center);
    
    f.render_widget(status_bar, chunks[2]);

    // Render popups if any are active
    if app.popup_state != PopupState::Hidden {
        debug!("Rendering popup: {:?}", app.popup_state);
        popups::render_popups::<B>(f, app);
    }
    
    debug!("Finished UI rendering");
}

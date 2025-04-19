use ratatui::layout::{Constraint, Direction, Layout, Rect};
use log::debug;

/// Helper function to create a centered rect
pub fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    debug!("Creating centered rect with percent_x: {}, height: {}, in area: {:?}", percent_x, height, r);
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let result = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1];
    
    debug!("Created centered rect: {:?}", result);
    result
}

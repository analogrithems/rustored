use ratatui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::layouts::centered_rect;
use crate::ui::models::PopupState;
use crate::ui::rustored::RustoredApp;

/// Render popups based on the current popup state
pub fn render_popups<B: Backend>(f: &mut Frame, app: &RustoredApp) {
    // Show popup if needed - render last to ensure they're on top
    match &app.popup_state {
        PopupState::ConfirmRestore(snapshot) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Restore snapshot: {}", snapshot.key))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press 'y' to confirm, 'n' to cancel")]),
            ])
            .block(Block::default().title("Confirm Restore").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Downloading(snapshot, progress, rate) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let _rate_mb = rate / 1024.0 / 1024.0;
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Downloading: {}", snapshot.key))]),
                Line::from(vec![Span::raw(format!("Progress: {:.1}%", progress * 100.0))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press Esc to cancel")]),
            ])
            .block(Block::default().title("Downloading").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::ConfirmCancel(snapshot, progress, rate) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let _rate_mb = rate / 1024.0 / 1024.0;
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Cancel download of: {}", snapshot.key))]),
                Line::from(vec![Span::raw(format!("Progress: {:.1}%", progress * 100.0))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Press 'y' to confirm cancel, 'n' to continue downloading")]),
            ])
            .block(Block::default().title("Confirm Cancel").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Error(message) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(message.as_str())
                .block(Block::default().title("Error").borders(Borders::ALL).style(Style::default().fg(Color::Red)))
                .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Success(message) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(message.as_str())
                .block(Block::default().title("Success").borders(Borders::ALL).style(Style::default().fg(Color::Green)))
                .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::TestingS3 => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw("Testing connection to S3...")]),
                Line::from(vec![]),
                Line::from(vec![Span::raw("Please wait")]),
            ])
                .block(Block::default().title("S3 Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
            f.render_widget(popup, area);
        },
        PopupState::TestS3Result(result) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(result.as_str())
                .block(Block::default().title("S3 Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::TestPgResult(result) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);
            let popup = Paragraph::new(result.as_str())
                .block(Block::default().title("PostgreSQL Connection Test").borders(Borders::ALL))
                .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Restoring(snapshot, progress) => {
            let area = centered_rect(60, 5, f.size());
            // Clear the area where the popup will be rendered
            f.render_widget(ratatui::widgets::Clear, area);

            // Create a progress bar
            let progress_value = progress; // Store in a local variable
            let _progress_percent = (progress_value * 100.0) as u16;
            let progress_bar_width = 50;
            let filled_width = (progress_bar_width as f32 * progress_value) as usize;
            let empty_width = progress_bar_width as usize - filled_width;

            let progress_bar = format!(
                "[{}{}] {:.1}%",
                "=".repeat(filled_width),
                " ".repeat(empty_width),
                progress_value * 100.0
            );

            let popup = Paragraph::new(vec![
                Line::from(vec![Span::raw(format!("Restoring: {}", snapshot.key))]),
                Line::from(vec![]),
                Line::from(vec![Span::raw(progress_bar)]),
            ])
            .block(Block::default().title("Restoring").borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(popup, area);
        }
        PopupState::Hidden => {}
    }
}

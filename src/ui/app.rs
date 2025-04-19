use log::debug;
use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::ui::rustored::RustoredApp;

/// Run the TUI application, delegating to RustoredApp
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut RustoredApp) -> Result<Option<String>> {
    // Initial load of snapshots
    if let Err(e) = app.snapshot_browser.load_snapshots().await {
        debug!("Failed to load snapshots: {}", e);
    }

    loop {
        // Draw UI
        terminal.draw(|f| crate::ui::renderer::ui::<B>(f, app))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Pass the key event to the app
                if let Some(snapshot_path) = app.handle_key_event::<B>(key).await? {
                    return Ok(Some(snapshot_path));
                }
            }
        }
    }
}

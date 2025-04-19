use std::io::stdout;
use log::debug;
use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use libc::SIGTSTP;
use libc::raise;
use ratatui::backend::Backend;
use ratatui::Terminal;

use crate::ui::models::InputMode;
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
                // Handle suspend on Ctrl+Z when in Normal mode
                if app.input_mode == InputMode::Normal && key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('z') {
                    // Exit TUI mode
                    disable_raw_mode()?;
                    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                    // Suspend process
                    unsafe { raise(SIGTSTP); }
                    // On resume, re-enter TUI
                    enable_raw_mode()?;
                    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                    continue;
                }
                
                // Pass the key event to the app
                if let Some(snapshot_path) = app.handle_key_event::<B>(key).await? {
                    return Ok(Some(snapshot_path));
                }
            }
        }
    }
}

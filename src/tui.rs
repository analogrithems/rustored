use std::{io, time::{Duration, Instant}};
use aws_sdk_s3::Client;
use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, Table, Row, Cell, Paragraph, Clear, Gauge}, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}};
use crate::config::{S3Config, DataStoreConfig};
use dirs;
use crate::restore::{download_snapshot, restore_to_datastore};
use anyhow::Result;

struct App {
    items: Vec<(String, u64, String)>, // (key, size, date)
    state: usize,
    show_help: bool,
    show_confirm: bool,
    show_download: bool,
    downloaded: u64,
    total: u64,
    show_restore: bool,
    show_debug: bool,
    raw_cmd: String,
}

impl App {
    fn new() -> Self { App { items: Vec::new(), state: 0, show_help: false, show_confirm: false, show_download: false, downloaded: 0, total: 0, show_restore: false, show_debug: false, raw_cmd: String::new() } }
    fn next(&mut self) { if !self.items.is_empty() { self.state = (self.state + 1) % self.items.len(); }}
    fn prev(&mut self) { if !self.items.is_empty() { if self.state == 0 { self.state = self.items.len() - 1 } else { self.state -= 1 } }}
}

pub async fn run_app(s3_client: Client, s3_cfg: S3Config, ds_cfg: DataStoreConfig) -> Result<()> {
    // fetch snapshot list
    let mut resp = s3_client.list_objects_v2().bucket(&s3_cfg.bucket);
    if !s3_cfg.prefix.is_empty() {
        resp = resp.prefix(&s3_cfg.prefix);
    }
    let out = resp.send().await?;
    let mut app = App::new();
    if let Some(contents) = out.contents { for obj in contents {
        let key = obj.key.unwrap_or_default();
        let size = obj.size as u64;
        let date = format!("{:?}", obj.last_modified.unwrap());
        app.items.push((key, size, date));
    }}
    // sort: newest first alphabetical
    app.items.sort_by(|a,b| b.0.cmp(&a.0));

    // terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // event loop
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    let mut confirmed = false;
    'main: loop {
        terminal.draw(|f| ui(f, &app))?;
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.prev(),
                    KeyCode::Enter => {
                        if app.show_confirm {
                            confirmed = true;
                            break 'main;
                        } else if !app.items.is_empty() {
                            app.show_confirm = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate { last_tick = Instant::now(); }
    }
    // on confirm: download loop inside TUI
    if confirmed {
        let (key, _size, _date) = &app.items[app.state];
        let dest = dirs::download_dir().unwrap_or(dirs::home_dir().unwrap()).join(key);
        app.show_download = true;
        download_snapshot(&s3_client, &s3_cfg.bucket, key, &dest).await?;
        app.show_download = false;
        // setup restore modal
        let cmd = format!("rustored restore --source {:?} --datastore-config", dest);
        app.raw_cmd = cmd;
        app.show_restore = true;
        // spawn restore task
        let rt = tokio::spawn(async move { restore_to_datastore(&dest, &ds_cfg).await });
        // modal event loop
        loop {
            terminal.draw(|f| ui(f, &app))?;
            if event::poll(Duration::from_millis(200))? {
                if let CEvent::Key(k) = event::read()? {
                    if k.code == KeyCode::Char('d') { app.show_debug = !app.show_debug; }
                }
            }
            if rt.is_finished() { break; }
        }
        // ignore result of restore task
        let _ = rt.await?;
        app.show_restore = false;
    }
    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui<B: ratatui::backend::Backend>(f: &mut ratatui::Frame<B>, app: &App) {
    let area = f.size();
    let chunks = Layout::default().direction(Direction::Vertical)
        .margin(1).constraints([Constraint::Min(3), Constraint::Length(3)]).split(area);
    // snapshot table
    let header = Row::new([Cell::from("Key"), Cell::from("Size"), Cell::from("Date")])
        .style(Style::default().fg(Color::Yellow)).bottom_margin(1);
    let rows: Vec<Row> = app.items.iter().map(|(k,s,d)| Row::new(vec![
        Cell::from(k.clone()),
        Cell::from(format!("{} B", s)),
        Cell::from(d.clone()),
    ])).collect();
    let table = Table::new(rows).header(header)
        .block(Block::default().borders(Borders::ALL).title("Snapshots"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::LightGreen))
        .widths(&[Constraint::Percentage(60), Constraint::Percentage(20), Constraint::Percentage(20)]);
    let mut state = ratatui::widgets::TableState::default(); state.select(Some(app.state));
    f.render_stateful_widget(table, chunks[0], &mut state);
    // legend
    let legend = Paragraph::new("q: quit  h: help  ↑/↓: navigate  Enter: select")
        .style(Style::default().fg(Color::White));
    f.render_widget(legend, chunks[1]);
    // help popup
    if app.show_help {
        let help = Paragraph::new("Help:\n q Quit\n h Toggle Help\n ↑/↓ Navigate\n Enter Confirm").block(Block::default().borders(Borders::ALL).title("Legend"));
        let popup = centered_rect(60, 40, area);
        f.render_widget(Clear, popup);
        f.render_widget(help, popup);
    }
    // confirm modal
    if app.show_confirm {
        let (k,s,d) = &app.items[app.state];
        let msg = format!("Confirm restore:\n{}\nSize: {} B\nDate: {}\n\nEnter to confirm, q to cancel", k, s, d);
        let confirm = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title("Confirm Restore"));
        let popup = centered_rect(50,30,area);
        f.render_widget(Clear,popup);
        f.render_widget(confirm,popup);
    }
    // download gauge
    if app.show_download {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Downloading"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(app.downloaded as f64 / app.total as f64)
            .label(format!("{}/{} B", app.downloaded, app.total));
        let popup = centered_rect(60, 20, area);
        f.render_widget(Clear, popup);
        f.render_widget(gauge, popup);
    }
    // restore modal
    if app.show_restore {
        let mut lines = vec!["Restoring...".to_string(), "Press 'd' for debug".to_string()];
        if app.show_debug { lines.push(app.raw_cmd.clone()); }
        let blk = Block::default().borders(Borders::ALL).title("Restore Progress");
        let p = Paragraph::new(lines.join("\n")).block(blk);
        let popup = centered_rect(60, 20, f.size());
        f.render_widget(Clear, popup);
        f.render_widget(p, popup);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let pw = r.width * percent_x / 100;
    let ph = r.height * percent_y / 100;
    Rect::new(
        r.x + (r.width - pw) / 2,
        r.y + (r.height - ph) / 2,
        pw, ph,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_navigation_wraps() {
        let mut app = App::new();
        // no items: state stays at 0
        app.next();
        assert_eq!(app.state, 0);
        app.prev();
        assert_eq!(app.state, 0);
        // two items: cycle forwards and backwards
        app.items = vec![("x".into(),0,"".into()), ("y".into(),0,"".into())];
        assert_eq!(app.state, 0);
        app.next();
        assert_eq!(app.state, 1);
        app.next();
        assert_eq!(app.state, 0);
        app.prev();
        assert_eq!(app.state, 1);
    }

    #[test]
    fn test_help_and_confirm_toggle() {
        let mut app = App::new();
        assert!(!app.show_help && !app.show_confirm);
        app.show_help = true;
        app.show_confirm = true;
        assert!(app.show_help && app.show_confirm);
    }
}

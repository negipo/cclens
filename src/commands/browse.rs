use crate::commands::export::render_session_markdown;
use crate::commands::query::{
    format_snippet, format_started_at, open_indexed_db, project_label,
};
use crate::db::Database;
use crate::models::QueryResult;
use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::HashMap;
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

#[derive(PartialEq)]
enum Mode {
    Normal,
    Search,
}

struct App {
    db: Database,
    project_paths: Vec<String>,
    base_sessions: Vec<QueryResult>,
    sessions: Vec<QueryResult>,
    selected: usize,
    mode: Mode,
    query_input: String,
    preview_cache: HashMap<String, String>,
    status: Option<String>,
}

impl App {
    fn full_export(&mut self, session_id: &str) -> String {
        if let Some(cached) = self.preview_cache.get(session_id) {
            return cached.clone();
        }
        let text = render_session_markdown(&self.db, session_id)
            .unwrap_or_else(|e| format!("(export failed: {})", e));
        self.preview_cache
            .insert(session_id.to_string(), text.clone());
        text
    }

    fn preview(&mut self) -> String {
        if self.sessions.is_empty() {
            return "no results".to_string();
        }
        let id = self.sessions[self.selected].session_id.clone();
        let full = self.full_export(&id);
        head_lines(&full, 20)
    }
}

fn pbcopy(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original(info);
    }));
}

pub fn run() -> Result<()> {
    if !std::io::stdout().is_terminal() {
        bail!("cclens browse は対話端末でのみ利用できます");
    }

    install_panic_hook();

    let (db, project_paths) = open_indexed_db()?;
    let project_refs: Vec<&str> = project_paths.iter().map(|s| s.as_str()).collect();
    let base_sessions = db.list_sessions(&project_refs, None, None, None, 120)?;
    drop(project_refs);

    let mut app = App {
        db,
        project_paths,
        sessions: base_sessions.clone(),
        base_sessions,
        selected: 0,
        mode: Mode::Normal,
        query_input: String::new(),
        preview_cache: HashMap::new(),
        status: None,
    };

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn event_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        let preview = app.preview();
        terminal.draw(|f| draw(f, app, &preview))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !app.sessions.is_empty()
                            && app.selected + 1 < app.sessions.len()
                        {
                            app.selected += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected > 0 {
                            app.selected -= 1;
                        }
                    }
                    KeyCode::Char('/') => {
                        app.mode = Mode::Search;
                        app.query_input.clear();
                        app.status = None;
                    }
                    KeyCode::Enter if !app.sessions.is_empty() => {
                        let id = app.sessions[app.selected].session_id.clone();
                        pbcopy(&id).ok();
                        app.status = Some("copied session_id".to_string());
                    }
                    KeyCode::Char('e') if !app.sessions.is_empty() => {
                        let id = app.sessions[app.selected].session_id.clone();
                        let full = app.full_export(&id);
                        pbcopy(&full).ok();
                        app.status = Some("copied export".to_string());
                    }
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Esc => {
                        app.mode = Mode::Normal;
                        app.sessions = app.base_sessions.clone();
                        app.selected = 0;
                    }
                    KeyCode::Enter => {
                        let refs: Vec<&str> =
                            app.project_paths.iter().map(|s| s.as_str()).collect();
                        let results =
                            app.db.browse_search(&app.query_input, &refs, 120)?;
                        app.sessions = results;
                        app.selected = 0;
                        app.mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        app.query_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.query_input.push(c);
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}

fn draw(f: &mut Frame, app: &App, preview: &str) {
    let bottom_len = 1u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Min(3),
            Constraint::Length(bottom_len),
        ])
        .split(f.area());

    let rows: Vec<Line> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let text = format!(
                "{}  {}  {}  {}",
                format_started_at(&r.started_at),
                project_label(r),
                r.session_id,
                format_snippet(r)
            );
            if i == app.selected {
                Line::from(Span::styled(
                    text,
                    Style::default().add_modifier(Modifier::REVERSED),
                ))
            } else {
                Line::from(text)
            }
        })
        .collect();
    let list = Paragraph::new(rows)
        .block(Block::default().borders(Borders::ALL).title("sessions"));
    f.render_widget(list, chunks[0]);

    let preview_widget = Paragraph::new(preview.to_string())
        .block(Block::default().borders(Borders::ALL).title("preview"));
    f.render_widget(preview_widget, chunks[1]);

    let bottom = match app.mode {
        Mode::Search => format!("/{}", app.query_input),
        Mode::Normal => app.status.clone().unwrap_or_default(),
    };
    f.render_widget(Paragraph::new(bottom), chunks[2]);
}

pub fn head_lines(text: &str, n: usize) -> String {
    text.lines().take(n).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_lines_truncates() {
        let s = "a\nb\nc\nd\ne";
        assert_eq!(head_lines(s, 3), "a\nb\nc");
    }

    #[test]
    fn test_head_lines_fewer_than_n() {
        let s = "a\nb";
        assert_eq!(head_lines(s, 20), "a\nb");
    }
}

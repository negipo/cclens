use crate::commands::export::render_session_markdown;
use crate::commands::query::{
    format_snippet, format_started_at, open_indexed_db, project_label,
};
use crate::db::Database;
use crate::models::QueryResult;
use anyhow::{bail, Result};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::collections::HashMap;
use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

#[derive(PartialEq)]
enum Mode {
    Normal,
    Search,
}

#[derive(PartialEq, Clone, Copy)]
enum Pane {
    Sessions,
    Preview,
}

struct App {
    db: Database,
    project_paths: Vec<String>,
    base_sessions: Vec<QueryResult>,
    sessions: Vec<QueryResult>,
    selected: usize,
    mode: Mode,
    focus: Pane,
    preview_scroll: u16,
    query_input: String,
    preview_cache: HashMap<String, String>,
    status: Option<String>,
    list_state: ListState,
    sessions_area: Rect,
    preview_area: Rect,
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
        self.full_export(&id)
    }

    fn select(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.selected = index;
            self.preview_scroll = 0;
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.sessions.is_empty() {
            return;
        }
        let next = (self.selected as i32 + delta)
            .clamp(0, self.sessions.len() as i32 - 1) as usize;
        if next != self.selected {
            self.selected = next;
            self.preview_scroll = 0;
        }
    }

    fn scroll_preview(&mut self, delta: i32) {
        let next = (self.preview_scroll as i32 + delta).max(0) as u16;
        self.preview_scroll = next;
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
        let _ = execute!(std::io::stdout(), DisableMouseCapture, LeaveAlternateScreen);
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
        focus: Pane::Sessions,
        preview_scroll: 0,
        query_input: String::new(),
        preview_cache: HashMap::new(),
        status: None,
        list_state: ListState::default(),
        sessions_area: Rect::default(),
        preview_area: Rect::default(),
    };

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn event_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        let preview = app.preview();
        terminal.draw(|f| draw(f, app, &preview))?;

        match event::read()? {
            Event::Key(key) => match app.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => {
                        app.focus = match app.focus {
                            Pane::Sessions => Pane::Preview,
                            Pane::Preview => Pane::Sessions,
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => match app.focus {
                        Pane::Sessions => app.move_selection(1),
                        Pane::Preview => app.scroll_preview(1),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match app.focus {
                        Pane::Sessions => app.move_selection(-1),
                        Pane::Preview => app.scroll_preview(-1),
                    },
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
                        app.preview_scroll = 0;
                    }
                    KeyCode::Enter => {
                        let refs: Vec<&str> =
                            app.project_paths.iter().map(|s| s.as_str()).collect();
                        let results =
                            app.db.browse_search(&app.query_input, &refs, 120)?;
                        app.sessions = results;
                        app.selected = 0;
                        app.preview_scroll = 0;
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
            },
            Event::Mouse(m) if app.mode == Mode::Normal => {
                let pos = Position::new(m.column, m.row);
                match m.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if app.sessions_area.contains(pos) {
                            app.focus = Pane::Sessions;
                            if let Some(index) = row_to_index(
                                m.row,
                                app.sessions_area,
                                app.list_state.offset(),
                                app.sessions.len(),
                            ) {
                                app.select(index);
                            }
                        } else if app.preview_area.contains(pos) {
                            app.focus = Pane::Preview;
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if app.sessions_area.contains(pos) {
                            app.move_selection(1);
                        } else if app.preview_area.contains(pos) {
                            app.scroll_preview(1);
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        if app.sessions_area.contains(pos) {
                            app.move_selection(-1);
                        } else if app.preview_area.contains(pos) {
                            app.scroll_preview(-1);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn row_to_index(row: u16, area: Rect, list_offset: usize, len: usize) -> Option<usize> {
    let first = area.y + 1;
    let last = area.y + area.height.saturating_sub(1);
    if row < first || row >= last {
        return None;
    }
    let index = list_offset + (row - first) as usize;
    if index < len {
        Some(index)
    } else {
        None
    }
}

fn clamp_scroll(scroll: u16, total_lines: usize, view_height: u16) -> u16 {
    let max = (total_lines as u16).saturating_sub(view_height);
    scroll.min(max)
}

fn draw(f: &mut Frame, app: &mut App, preview: &str) {
    let bottom_len = 1u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Min(3),
            Constraint::Length(bottom_len),
        ])
        .split(f.area());

    app.sessions_area = chunks[0];
    app.preview_area = chunks[1];

    let sessions_border = pane_border(app.focus == Pane::Sessions);
    let preview_border = pane_border(app.focus == Pane::Preview);

    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .map(|r| {
            let text = format!(
                "{}  {}  {}  {}",
                format_started_at(&r.started_at),
                project_label(r),
                r.session_id,
                format_snippet(r)
            );
            ListItem::new(text)
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(sessions_border)
                .title("sessions"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    if app.sessions.is_empty() {
        app.list_state.select(None);
    } else {
        app.list_state.select(Some(app.selected));
    }
    f.render_stateful_widget(list, chunks[0], &mut app.list_state);

    let view_height = chunks[1].height.saturating_sub(2);
    app.preview_scroll = clamp_scroll(app.preview_scroll, preview.lines().count(), view_height);
    let preview_widget = Paragraph::new(preview.to_string())
        .scroll((app.preview_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(preview_border)
                .title("preview"),
        );
    f.render_widget(preview_widget, chunks[1]);

    let bottom = match app.mode {
        Mode::Search => format!("/{}", app.query_input),
        Mode::Normal => app.status.clone().unwrap_or_default(),
    };
    f.render_widget(Paragraph::new(bottom), chunks[2]);
}

fn pane_border(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_scroll_within_bounds() {
        assert_eq!(clamp_scroll(2, 100, 10), 2);
    }

    #[test]
    fn test_clamp_scroll_caps_at_max() {
        assert_eq!(clamp_scroll(200, 30, 10), 20);
    }

    #[test]
    fn test_clamp_scroll_fits_in_view() {
        assert_eq!(clamp_scroll(5, 8, 10), 0);
    }

    #[test]
    fn test_row_to_index_maps_click_row() {
        let area = Rect::new(0, 0, 40, 10);
        assert_eq!(row_to_index(1, area, 0, 20), Some(0));
        assert_eq!(row_to_index(3, area, 0, 20), Some(2));
    }

    #[test]
    fn test_row_to_index_honors_offset() {
        let area = Rect::new(0, 0, 40, 10);
        assert_eq!(row_to_index(1, area, 5, 20), Some(5));
    }

    #[test]
    fn test_row_to_index_rejects_border_rows() {
        let area = Rect::new(0, 0, 40, 10);
        assert_eq!(row_to_index(0, area, 0, 20), None);
        assert_eq!(row_to_index(9, area, 0, 20), None);
    }

    #[test]
    fn test_row_to_index_rejects_beyond_len() {
        let area = Rect::new(0, 0, 40, 10);
        assert_eq!(row_to_index(5, area, 0, 2), None);
    }
}

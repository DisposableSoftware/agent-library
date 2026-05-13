use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // document list
            Constraint::Length(4), // help line + status line
        ])
        .split(f.area());

    draw_document_list(f, app, chunks[0]);
    draw_status_bar(f, app, chunks[1]);
}

fn draw_document_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    // 2 borders + 2 highlight symbol + 2 icon + 20 for "  [TaskType]"
    let overhead: usize = 26;
    let name_width = (area.width as usize).saturating_sub(overhead).max(20);

    let items: Vec<ListItem> = app
        .documents
        .iter()
        .map(|d| {
            let enabled_icon = if d.enabled { "✓" } else { "✗" };
            let enabled_color = if d.enabled { Color::Green } else { Color::Red };
            let line = Line::from(vec![
                Span::styled(
                    format!("{enabled_icon} "),
                    Style::default().fg(enabled_color),
                ),
                Span::raw(format!("{:<width$}", truncate(&d.name, name_width), width = name_width)),
                Span::styled(
                    format!("  [{}]", d.task_type),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    if d.is_skill { " ★" } else { "" },
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    state.select(if app.documents.is_empty() { None } else { Some(app.cursor) });

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Agent Document Library "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    const HELP: &str =
        "j/k ↑↓ move  space/e toggle  s skill  o/Enter edit  d delete  r refresh  R rebuild-root  q quit";

    let text = Text::from(vec![
        Line::from(HELP),
        Line::from(Span::styled(
            app.status_msg.as_deref().unwrap_or(""),
            Style::default().fg(Color::Cyan),
        )),
    ]);

    let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

/// Truncate `s` to at most `max` Unicode scalars, slicing on a char boundary
/// so we never panic on multibyte input.
fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

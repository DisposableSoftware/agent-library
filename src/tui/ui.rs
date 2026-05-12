use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // pattern list
            Constraint::Length(3), // status / help bar
        ])
        .split(f.area());

    draw_pattern_list(f, app, chunks[0]);
    draw_status_bar(f, app, chunks[1]);
}

fn draw_pattern_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .patterns
        .iter()
        .map(|p| {
            let enabled_icon = if p.enabled { "✓" } else { "✗" };
            let enabled_color = if p.enabled { Color::Green } else { Color::Red };
            let line = Line::from(vec![
                Span::styled(
                    format!("{enabled_icon} "),
                    Style::default().fg(enabled_color),
                ),
                Span::raw(format!("{:<30}", truncate(&p.name, 30))),
                Span::styled(
                    format!("  [{}]", p.task_type),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    state.select(if app.patterns.is_empty() { None } else { Some(app.cursor) });

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Agent Pattern Library "),
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
    let msg = app
        .status_msg
        .as_deref()
        .unwrap_or("j/k ↑↓ move  space/e toggle  o/Enter edit  r refresh  q quit");

    let paragraph = Paragraph::new(msg).block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

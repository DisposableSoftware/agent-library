use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use super::app::{App, Mode};
use super::theme::Theme;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.themes[app.theme_idx].theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // document list
            Constraint::Length(4), // help line + status line
        ])
        .split(f.area());

    draw_document_list(f, app, chunks[0], theme);
    draw_status_bar(f, app, chunks[1], theme);

    if app.mode == Mode::ThemePicker {
        draw_theme_picker(f, app, theme);
    }
}

fn draw_document_list(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let max_task_type_len = app
        .documents
        .iter()
        .map(|d| d.task_type.chars().count())
        .max()
        .unwrap_or(0);
    // 2 borders + 2 highlight + 2 enabled + 2 skill + 4 for "  []" + task type
    let overhead = 12 + max_task_type_len;
    let name_width = (area.width as usize).saturating_sub(overhead).max(10);

    let items: Vec<ListItem> = app
        .documents
        .iter()
        .map(|d| {
            let enabled_icon = if d.enabled { "✓" } else { "✗" };
            let enabled_color = if d.enabled { theme.base0b } else { theme.base08 };
            let line = Line::from(vec![
                Span::styled(
                    format!("{enabled_icon} "),
                    Style::default().fg(enabled_color),
                ),
                Span::styled(
                    if d.is_skill { "★ " } else { "  " },
                    Style::default().fg(theme.base0a),
                ),
                Span::styled(
                    format!("{:<width$}", truncate(&d.name, name_width), width = name_width),
                    Style::default().fg(theme.base05),
                ),
                Span::styled(
                    format!("  [{}]", d.task_type),
                    Style::default().fg(theme.base03),
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
                .border_style(Style::default().fg(theme.base03))
                .title(Span::styled(
                    " Agent Document Library ",
                    Style::default().fg(theme.base0d),
                ))
                .style(Style::default().bg(theme.base00)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.base02)
                .fg(theme.base05)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let help = if app.mode == Mode::ThemePicker {
        "j/k ↑↓ move  Enter select  Esc cancel"
    } else {
        "j/k ↑↓ move  space/e toggle  s skill  o/Enter edit  d delete  r refresh  R rebuild-root  T theme  q quit"
    };

    let text = Text::from(vec![
        Line::from(Span::styled(help, Style::default().fg(theme.base04))),
        Line::from(Span::styled(
            app.status_msg.as_deref().unwrap_or(""),
            Style::default().fg(theme.base0c),
        )),
    ]);

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.base03))
            .style(Style::default().bg(theme.base00)),
    );
    f.render_widget(paragraph, area);
}

fn draw_theme_picker(f: &mut Frame, app: &App, theme: &Theme) {
    let area = centered_rect(50, 60, f.area());

    let items: Vec<ListItem> = app
        .themes
        .iter()
        .map(|t| ListItem::new(Span::styled(&t.name, Style::default().fg(theme.base05))))
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.picker_cursor));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.base0e))
                .title(Span::styled(
                    " Select Theme ",
                    Style::default().fg(theme.base0e).add_modifier(Modifier::BOLD),
                ))
                .style(Style::default().bg(theme.base01)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.base02)
                .fg(theme.base07)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_widget(Clear, area);
    f.render_stateful_widget(list, area, &mut state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

/// Truncate `s` to at most `max` Unicode scalars, slicing on a char boundary
/// so we never panic on multibyte input.
fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

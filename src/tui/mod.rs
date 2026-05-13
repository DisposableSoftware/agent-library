pub mod theme;
mod app;
mod events;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::error::AppError;
use crate::store::DocumentStore;
use app::{App, Mode};
use events::{AppEvent, next_event};

/// RAII guard for raw mode + alternate screen. Drop restores the terminal,
/// so even a panic inside the TUI run loop leaves a usable shell behind.
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(TerminalGuard)
    }

    fn suspend(&self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn resume(&self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub fn run_tui(store: DocumentStore, initial_theme: Option<String>) -> Result<(), AppError> {
    let guard = TerminalGuard::enter().map_err(AppError::Io)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(AppError::Io)?;

    let result = run_loop(&mut terminal, store, initial_theme, &guard);

    let _ = terminal.show_cursor();
    // guard's Drop restores the terminal on its way out.
    drop(guard);
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    store: DocumentStore,
    initial_theme: Option<String>,
    guard: &TerminalGuard,
) -> Result<(), AppError> {
    let mut app = App::new(store, initial_theme.as_deref())?;

    loop {
        terminal.draw(|f| ui::draw(f, &app)).map_err(AppError::Io)?;

        let Some(event) = next_event()? else {
            continue;
        };

        match event {
            AppEvent::Quit => {
                if app.mode == Mode::ThemePicker {
                    app.close_theme_picker();
                } else {
                    app.quit();
                }
            }
            AppEvent::MoveUp => {
                if app.mode == Mode::ThemePicker {
                    app.picker_up();
                } else {
                    app.move_cursor_up();
                }
            }
            AppEvent::MoveDown => {
                if app.mode == Mode::ThemePicker {
                    app.picker_down();
                } else {
                    app.move_cursor_down();
                }
            }
            AppEvent::Toggle => {
                if app.mode == Mode::Normal && let Err(e) = app.toggle_enabled() {
                    app.status_msg = Some(format!("Error: {e}"));
                }
            }
            AppEvent::OpenEditor => {
                if app.mode == Mode::ThemePicker {
                    if let Err(e) = app.apply_selected_theme() {
                        app.status_msg = Some(format!("Error saving prefs: {e}"));
                    }
                } else if app.mode == Mode::Normal {
                    guard.suspend().map_err(AppError::Io)?;
                    let edit_result = app.open_in_editor();
                    guard.resume().map_err(AppError::Io)?;
                    terminal.clear().map_err(AppError::Io)?;

                    if let Err(e) = edit_result {
                        app.status_msg = Some(format!("Editor error: {e}"));
                    }
                }
            }
            AppEvent::Refresh => {
                if app.mode == Mode::Normal {
                    if let Err(e) = app.refresh() {
                        app.status_msg = Some(format!("Refresh error: {e}"));
                    } else {
                        app.status_msg = Some("Refreshed.".to_owned());
                    }
                }
            }
            AppEvent::RebuildRoot => {
                if app.mode == Mode::Normal && let Err(e) = app.rebuild_root() {
                    app.status_msg = Some(format!("Error: {e}"));
                }
            }
            AppEvent::ToggleSkill => {
                if app.mode == Mode::Normal && let Err(e) = app.toggle_skill() {
                    app.status_msg = Some(format!("Error: {e}"));
                }
            }
            AppEvent::DeleteRequest => {
                if app.mode == Mode::Normal {
                    app.request_delete();
                }
            }
            AppEvent::Confirm => {
                if app.mode == Mode::ThemePicker {
                    if let Err(e) = app.apply_selected_theme() {
                        app.status_msg = Some(format!("Error saving prefs: {e}"));
                    }
                } else if app.mode == Mode::ConfirmDelete
                    && let Err(e) = app.confirm_delete()
                {
                    app.status_msg = Some(format!("Error: {e}"));
                    app.mode = Mode::Normal;
                }
            }
            AppEvent::Cancel => {
                if app.mode == Mode::ThemePicker {
                    app.close_theme_picker();
                } else if app.mode == Mode::ConfirmDelete {
                    app.cancel_delete();
                }
            }
            AppEvent::OpenThemePicker => {
                if app.mode == Mode::Normal {
                    app.open_theme_picker();
                }
            }
        }

        if app.mode == Mode::Quitting {
            break;
        }
    }

    Ok(())
}

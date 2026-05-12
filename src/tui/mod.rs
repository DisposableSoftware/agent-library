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
use crate::store::PatternStore;
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

pub fn run_tui(store: PatternStore) -> Result<(), AppError> {
    let guard = TerminalGuard::enter().map_err(AppError::Io)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).map_err(AppError::Io)?;

    let result = run_loop(&mut terminal, store, &guard);

    let _ = terminal.show_cursor();
    // guard's Drop restores the terminal on its way out.
    drop(guard);
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    store: PatternStore,
    guard: &TerminalGuard,
) -> Result<(), AppError> {
    let mut app = App::new(store)?;

    loop {
        terminal.draw(|f| ui::draw(f, &app)).map_err(AppError::Io)?;

        let Some(event) = next_event()? else {
            continue;
        };

        match event {
            AppEvent::Quit => {
                app.quit();
            }
            AppEvent::MoveUp => app.move_cursor_up(),
            AppEvent::MoveDown => app.move_cursor_down(),
            AppEvent::Toggle => {
                if let Err(e) = app.toggle_enabled() {
                    app.status_msg = Some(format!("Error: {e}"));
                }
            }
            AppEvent::OpenEditor => {
                guard.suspend().map_err(AppError::Io)?;
                let edit_result = app.open_in_editor();
                guard.resume().map_err(AppError::Io)?;
                terminal.clear().map_err(AppError::Io)?;

                if let Err(e) = edit_result {
                    app.status_msg = Some(format!("Editor error: {e}"));
                }
            }
            AppEvent::Refresh => {
                if let Err(e) = app.refresh() {
                    app.status_msg = Some(format!("Refresh error: {e}"));
                } else {
                    app.status_msg = Some("Refreshed.".to_owned());
                }
            }
        }

        if app.mode == Mode::Quitting {
            break;
        }
    }

    Ok(())
}

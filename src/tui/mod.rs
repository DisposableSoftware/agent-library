mod app;
mod events;
mod ui;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::error::AppError;
use crate::store::PatternStore;
use app::{App, Mode};
use events::{AppEvent, next_event};

pub fn run_tui(store: PatternStore) -> Result<(), AppError> {
    enable_raw_mode().map_err(AppError::Io)?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(AppError::Io)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(AppError::Io)?;

    let result = run_loop(&mut terminal, store);

    // Always restore terminal, even on error
    disable_raw_mode().map_err(AppError::Io)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(AppError::Io)?;
    terminal.show_cursor().map_err(AppError::Io)?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    store: PatternStore,
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
                // Restore terminal before handing off to the editor
                disable_raw_mode().map_err(AppError::Io)?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(AppError::Io)?;

                let edit_result = app.open_in_editor();

                // Re-enter TUI mode
                enable_raw_mode().map_err(AppError::Io)?;
                execute!(terminal.backend_mut(), EnterAlternateScreen).map_err(AppError::Io)?;
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

use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::error::AppError;

#[derive(Debug)]
pub enum AppEvent {
    Quit,
    MoveUp,
    MoveDown,
    Toggle,
    OpenEditor,
    Refresh,
    DeleteRequest,
    Confirm,
    Cancel,
    ToggleSkill,
    RebuildRoot,
    OpenThemePicker,
    OpenHelp,
}

/// Block until the next key event and translate it to an AppEvent.
/// Returns `None` for unrecognised keys (caller should continue the loop).
pub fn next_event() -> Result<Option<AppEvent>, AppError> {
    let event = event::read().map_err(AppError::Io)?;

    match event {
        Event::Key(key) => {
            let ev = match key.code {
                KeyCode::Char('q') => Some(AppEvent::Quit),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(AppEvent::Quit)
                }
                KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::MoveUp),
                KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::MoveDown),
                KeyCode::Char(' ') | KeyCode::Char('e') => Some(AppEvent::Toggle),
                KeyCode::Enter | KeyCode::Char('o') => Some(AppEvent::OpenEditor),
                KeyCode::Char('r') => Some(AppEvent::Refresh),
                KeyCode::Char('R') => Some(AppEvent::RebuildRoot),
                KeyCode::Char('d') => Some(AppEvent::DeleteRequest),
                KeyCode::Char('s') => Some(AppEvent::ToggleSkill),
                KeyCode::Char('T') => Some(AppEvent::OpenThemePicker),
                KeyCode::Char('?') => Some(AppEvent::OpenHelp),
                KeyCode::Char('y') => Some(AppEvent::Confirm),
                KeyCode::Char('n') | KeyCode::Esc => Some(AppEvent::Cancel),
                _ => None,
            };
            Ok(ev)
        }
        _ => Ok(None),
    }
}

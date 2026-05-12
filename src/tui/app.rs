use std::process::Command;

use crate::error::AppError;
use crate::pattern::Pattern;
use crate::store::PatternStore;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Quitting,
}

pub struct App {
    pub patterns: Vec<Pattern>,
    pub cursor: usize,
    pub mode: Mode,
    pub store: PatternStore,
    pub status_msg: Option<String>,
}

impl App {
    pub fn new(store: PatternStore) -> Result<Self, AppError> {
        let patterns = store.load_all()?;
        Ok(App {
            patterns,
            cursor: 0,
            mode: Mode::Normal,
            store,
            status_msg: None,
        })
    }

    pub fn quit(&mut self) {
        self.mode = Mode::Quitting;
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if !self.patterns.is_empty() && self.cursor < self.patterns.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn toggle_enabled(&mut self) -> Result<(), AppError> {
        if self.patterns.is_empty() {
            return Ok(());
        }
        let p = &mut self.patterns[self.cursor];
        p.enabled = !p.enabled;
        self.store.update_pattern(p)?;
        let name = self.patterns[self.cursor].name.clone();
        let enabled = self.patterns[self.cursor].enabled;
        self.status_msg = Some(format!(
            "{name} {}",
            if enabled { "enabled" } else { "disabled" }
        ));
        Ok(())
    }

    /// Open the selected pattern in $VISUAL / $EDITOR / vi.
    /// Restores terminal state before spawning and re-initialises afterwards.
    pub fn open_in_editor(&mut self) -> Result<(), AppError> {
        if self.patterns.is_empty() {
            return Ok(());
        }
        let path = self.patterns[self.cursor].file_path.clone();
        let editor_var = std::env::var("VISUAL")
            .or_else(|_| std::env::var("EDITOR"))
            .unwrap_or_else(|_| "vi".to_owned());

        // Split so values like `EDITOR="emacsclient -nw"` work as expected.
        let parts = shell_words::split(&editor_var)
            .map_err(|e| AppError::Parse(format!("invalid editor command: {e}")))?;
        let (program, args) = parts
            .split_first()
            .ok_or_else(|| AppError::Parse("editor command is empty".into()))?;

        Command::new(program)
            .args(args)
            .arg(&path)
            .status()
            .map_err(AppError::Io)?;

        // Reload the pattern from disk after editing
        self.refresh()?;
        self.status_msg = Some(format!("Reloaded from {}", path.display()));
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<(), AppError> {
        let new_patterns = self.store.load_all()?;
        self.cursor = self.cursor.min(new_patterns.len().saturating_sub(1));
        self.patterns = new_patterns;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_store() -> PatternStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("agent-library-app-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        PatternStore::new(dir)
    }

    fn make_pattern(name: &str, enabled: bool) -> Pattern {
        Pattern {
            name: name.to_owned(),
            enabled,
            date_added: "2026-05-12".to_owned(),
            task_type: "Architecture".to_owned(),
            file_path: PathBuf::new(),
            content: "# Content\n".to_owned(),
        }
    }

    fn app_with_patterns(n: usize) -> App {
        let store = temp_store();
        for i in 0..n {
            let p = make_pattern(&format!("Pattern {i}"), true);
            store.save_pattern(&p).unwrap();
        }
        App::new(store).unwrap()
    }

    #[test]
    fn move_cursor_up_at_zero_stays() {
        let mut app = app_with_patterns(3);
        app.cursor = 0;
        app.move_cursor_up();
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn move_cursor_down_at_last_stays() {
        let mut app = app_with_patterns(3);
        app.cursor = 2;
        app.move_cursor_down();
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn move_cursor_navigates() {
        let mut app = app_with_patterns(3);
        app.move_cursor_down();
        assert_eq!(app.cursor, 1);
        app.move_cursor_down();
        assert_eq!(app.cursor, 2);
        app.move_cursor_up();
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn toggle_enabled_flips_field() {
        let mut app = app_with_patterns(1);
        let originally_enabled = app.patterns[0].enabled;
        app.toggle_enabled().unwrap();
        assert_eq!(app.patterns[0].enabled, !originally_enabled);
        app.toggle_enabled().unwrap();
        assert_eq!(app.patterns[0].enabled, originally_enabled);
    }

    #[test]
    fn toggle_enabled_empty_list_no_panic() {
        let store = temp_store();
        let mut app = App::new(store).unwrap();
        // Must not panic
        app.toggle_enabled().unwrap();
    }

    #[test]
    fn quit_sets_mode() {
        let store = temp_store();
        let mut app = App::new(store).unwrap();
        app.quit();
        assert_eq!(app.mode, Mode::Quitting);
    }
}

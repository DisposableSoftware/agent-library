use std::process::Command;

use crate::document::Document;
use crate::error::AppError;
use crate::store::DocumentStore;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Quitting,
}

pub struct App {
    pub documents: Vec<Document>,
    pub cursor: usize,
    pub mode: Mode,
    pub store: DocumentStore,
    pub status_msg: Option<String>,
}

impl App {
    pub fn new(store: DocumentStore) -> Result<Self, AppError> {
        let documents = store.load_all()?;
        Ok(App {
            documents,
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
        if !self.documents.is_empty() && self.cursor < self.documents.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn toggle_enabled(&mut self) -> Result<(), AppError> {
        if self.documents.is_empty() {
            return Ok(());
        }
        let d = &mut self.documents[self.cursor];
        d.enabled = !d.enabled;
        self.store.update_document(d)?;
        let name = self.documents[self.cursor].name.clone();
        let enabled = self.documents[self.cursor].enabled;
        self.status_msg = Some(format!(
            "{name} {}",
            if enabled { "enabled" } else { "disabled" }
        ));
        Ok(())
    }

    /// Open the selected document in $VISUAL / $EDITOR / vi.
    /// Restores terminal state before spawning and re-initialises afterwards.
    pub fn open_in_editor(&mut self) -> Result<(), AppError> {
        if self.documents.is_empty() {
            return Ok(());
        }
        let path = self.documents[self.cursor].file_path.clone();
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

        // Reload the document from disk after editing
        self.refresh()?;
        self.status_msg = Some(format!("Reloaded from {}", path.display()));
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<(), AppError> {
        let new_documents = self.store.load_all()?;
        self.cursor = self.cursor.min(new_documents.len().saturating_sub(1));
        self.documents = new_documents;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_store() -> DocumentStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("agent-library-app-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        DocumentStore::new(dir)
    }

    fn make_document(name: &str, enabled: bool) -> Document {
        Document {
            name: name.to_owned(),
            enabled,
            date_added: "2026-05-12".to_owned(),
            task_type: "Architecture".to_owned(),
            file_path: PathBuf::new(),
            content: "# Content\n".to_owned(),
        }
    }

    fn app_with_documents(n: usize) -> App {
        let store = temp_store();
        for i in 0..n {
            let d = make_document(&format!("Document {i}"), true);
            store.save_document(&d).unwrap();
        }
        App::new(store).unwrap()
    }

    #[test]
    fn move_cursor_up_at_zero_stays() {
        let mut app = app_with_documents(3);
        app.cursor = 0;
        app.move_cursor_up();
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn move_cursor_down_at_last_stays() {
        let mut app = app_with_documents(3);
        app.cursor = 2;
        app.move_cursor_down();
        assert_eq!(app.cursor, 2);
    }

    #[test]
    fn move_cursor_navigates() {
        let mut app = app_with_documents(3);
        app.move_cursor_down();
        assert_eq!(app.cursor, 1);
        app.move_cursor_down();
        assert_eq!(app.cursor, 2);
        app.move_cursor_up();
        assert_eq!(app.cursor, 1);
    }

    #[test]
    fn toggle_enabled_flips_field() {
        let mut app = app_with_documents(1);
        let originally_enabled = app.documents[0].enabled;
        app.toggle_enabled().unwrap();
        assert_eq!(app.documents[0].enabled, !originally_enabled);
        app.toggle_enabled().unwrap();
        assert_eq!(app.documents[0].enabled, originally_enabled);
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

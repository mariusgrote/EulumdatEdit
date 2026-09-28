//! Application state: documents are independent from the windows that show them.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Mutex;

use eulumdat_core::Eulumdat;

/// The open document and its on-disk association.
#[derive(Debug, Default)]
pub struct OpenDoc {
    pub model: Option<Eulumdat>,
    /// Absolute path the document was opened from / last saved to.
    pub path: Option<String>,
    /// Whether the in-memory model differs from the last saved state.
    pub dirty: bool,
    /// When set, enforces the legacy EULUMDAT text-length limits (8.3-era
    /// filename, etc.). Off by default; modern files are unrestricted.
    pub strict_validation: bool,
}

/// The ordered tabs shown by one native window.
#[derive(Debug, Default)]
pub(crate) struct WindowTabs {
    pub tabs: Vec<String>,
    pub active: Option<String>,
}

/// All document and window relationships guarded by one lock so moving a tab
/// can never leave it in two windows, or in none.
#[derive(Debug, Default)]
pub struct Workspace {
    docs: HashMap<String, OpenDoc>,
    windows: HashMap<String, WindowTabs>,
}

/// State needed to undo a provisional detach if native window creation fails.
pub struct DetachRollback {
    tab_id: String,
    source: String,
    target: String,
    source_index: usize,
    source_active: Option<String>,
    source_active_after_detach: Option<String>,
}

impl Workspace {
    pub fn ensure_window(&mut self, label: &str) {
        self.windows.entry(label.to_string()).or_default();
    }

    pub fn window(&self, label: &str) -> Option<&WindowTabs> {
        self.windows.get(label)
    }

    pub fn doc(&self, id: &str) -> Option<&OpenDoc> {
        self.docs.get(id)
    }

    pub fn active_id(&self, label: &str) -> Result<&str, String> {
        self.window(label)
            .and_then(|window| window.active.as_deref())
            .ok_or_else(|| "No document open".to_string())
    }

    pub fn active_doc_mut(&mut self, label: &str) -> Result<&mut OpenDoc, String> {
        let id = self.active_id(label)?.to_string();
        self.docs
            .get_mut(&id)
            .ok_or_else(|| "Active document is missing".to_string())
    }

    pub fn doc_in_window_mut(&mut self, label: &str, id: &str) -> Result<&mut OpenDoc, String> {
        if !self
            .window(label)
            .is_some_and(|window| window.tabs.iter().any(|tab| tab == id))
        {
            return Err("Tab does not belong to this window".to_string());
        }
        self.docs
            .get_mut(id)
            .ok_or_else(|| "Document is missing".to_string())
    }

    pub fn insert(&mut self, label: &str, id: String, doc: OpenDoc) -> Result<(), String> {
        if self.docs.contains_key(&id) {
            return Err("Document id is already open".to_string());
        }
        if let Some(path) = doc.path.as_deref() {
            if self.find_path(path).is_some() {
                return Err("This file is already open in another tab".to_string());
            }
        }
        self.docs.insert(id.clone(), doc);
        let window = self.windows.entry(label.to_string()).or_default();
        window.tabs.push(id.clone());
        window.active = Some(id);
        Ok(())
    }

    pub fn find_path(&self, path: &str) -> Option<(String, String)> {
        self.docs
            .iter()
            .find(|(_, doc)| doc.path.as_deref() == Some(path))
            .and_then(|(id, _)| {
                self.windows.iter().find_map(|(label, window)| {
                    window
                        .tabs
                        .contains(id)
                        .then(|| (label.clone(), id.clone()))
                })
            })
    }

    pub fn focus_path(&mut self, path: &str) -> Option<String> {
        let (label, id) = self.find_path(path)?;
        self.windows.get_mut(&label)?.active = Some(id);
        Some(label)
    }

    pub fn path_in_use_by_other(&self, path: &str, id: &str) -> bool {
        self.docs
            .iter()
            .any(|(other, doc)| other != id && doc.path.as_deref() == Some(path))
    }

    /// Checks global path ownership before writing, then changes the active
    /// document's association only after the write succeeds.
    pub fn save_as(
        &mut self,
        label: &str,
        path: &str,
        write: impl FnOnce(&Eulumdat) -> Result<Eulumdat, String>,
    ) -> Result<(Eulumdat, bool), String> {
        let id = self.active_id(label)?.to_string();
        if self.path_in_use_by_other(path, &id) {
            return Err("This file is already open in another tab".to_string());
        }
        let doc = self.active_doc_mut(label)?;
        let model = doc.model.as_ref().ok_or("No document open")?;
        let saved = write(model)?;
        doc.model = Some(saved.clone());
        doc.path = Some(path.to_string());
        doc.dirty = false;
        Ok((saved, doc.strict_validation))
    }

    fn select_neighbor(window: &mut WindowTabs, removed: &str, index: usize) {
        if window.active.as_deref() == Some(removed) {
            window.active = window
                .tabs
                .get(index)
                .or_else(|| index.checked_sub(1).and_then(|i| window.tabs.get(i)))
                .cloned();
        }
    }

    pub fn close(&mut self, label: &str, id: Option<&str>) -> Result<(), String> {
        let id = match id {
            Some(id) => id.to_string(),
            None => self.active_id(label)?.to_string(),
        };
        let window = self
            .windows
            .get_mut(label)
            .ok_or_else(|| "Window is missing".to_string())?;
        let index = window
            .tabs
            .iter()
            .position(|tab| tab == &id)
            .ok_or_else(|| "Tab does not belong to this window".to_string())?;
        window.tabs.remove(index);
        Self::select_neighbor(window, &id, index);
        self.docs.remove(&id);
        Ok(())
    }

    pub fn activate(&mut self, label: &str, id: &str) -> Result<(), String> {
        let window = self
            .windows
            .get_mut(label)
            .ok_or_else(|| "Window is missing".to_string())?;
        if !window.tabs.iter().any(|tab| tab == id) {
            return Err("Tab does not belong to this window".to_string());
        }
        window.active = Some(id.to_string());
        Ok(())
    }

    pub fn move_tab(
        &mut self,
        id: &str,
        source: &str,
        target: &str,
        target_index: Option<usize>,
    ) -> Result<(), String> {
        let source_window = self
            .windows
            .get_mut(source)
            .ok_or_else(|| "Source window is missing".to_string())?;
        let source_index = source_window
            .tabs
            .iter()
            .position(|tab| tab == id)
            .ok_or_else(|| "Tab does not belong to the source window".to_string())?;
        let target_index = target_index.map(|index| {
            if source == target && index > source_index {
                index - 1
            } else {
                index
            }
        });
        source_window.tabs.remove(source_index);
        Self::select_neighbor(source_window, id, source_index);
        let target_window = self.windows.entry(target.to_string()).or_default();
        let index = target_index
            .unwrap_or(target_window.tabs.len())
            .min(target_window.tabs.len());
        target_window.tabs.insert(index, id.to_string());
        target_window.active = Some(id.to_string());
        Ok(())
    }

    pub fn begin_detach(
        &mut self,
        id: &str,
        source: &str,
        target: &str,
    ) -> Result<DetachRollback, String> {
        if self.windows.contains_key(target) {
            return Err("Target window already exists".to_string());
        }
        let window = self
            .windows
            .get(source)
            .ok_or_else(|| "Source window is missing".to_string())?;
        let source_index = window
            .tabs
            .iter()
            .position(|tab| tab == id)
            .ok_or_else(|| "Tab does not belong to the source window".to_string())?;
        let source_active = window.active.clone();
        self.move_tab(id, source, target, None)?;
        let source_active_after_detach =
            self.window(source).and_then(|window| window.active.clone());
        Ok(DetachRollback {
            tab_id: id.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            source_index,
            source_active,
            source_active_after_detach,
        })
    }

    pub fn rollback_detach(&mut self, rollback: DetachRollback) {
        if let Some(target) = self.windows.get_mut(&rollback.target) {
            target.tabs.retain(|id| id != &rollback.tab_id);
        }
        self.windows.remove(&rollback.target);
        if let Some(source) = self.windows.get_mut(&rollback.source) {
            let index = rollback.source_index.min(source.tabs.len());
            source.tabs.insert(index, rollback.tab_id.clone());
            if source.active == rollback.source_active_after_detach
                && rollback
                    .source_active
                    .as_ref()
                    .is_some_and(|id| source.tabs.contains(id))
            {
                source.active = rollback.source_active;
            }
            if !source
                .active
                .as_ref()
                .is_some_and(|id| source.tabs.contains(id))
            {
                source.active = Some(rollback.tab_id);
            }
        }
    }

    pub fn remove_window(&mut self, label: &str) {
        if let Some(window) = self.windows.remove(label) {
            for id in window.tabs {
                self.docs.remove(&id);
            }
        }
    }

    pub fn other_documents_dirty(&self, label: &str) -> bool {
        let active = self
            .window(label)
            .and_then(|window| window.active.as_deref());
        self.docs
            .iter()
            .any(|(id, doc)| Some(id.as_str()) != active && doc.dirty)
    }

    pub fn path_is_open(&self, path: &str) -> bool {
        self.docs
            .values()
            .any(|doc| doc.path.as_deref() == Some(path))
    }
}

/// Shared, mutex-guarded application state.
#[derive(Debug, Default)]
pub struct AppState {
    pub workspace: Mutex<Workspace>,
    /// Active native tab previews, keyed by their source document window.
    pub tab_previews: Mutex<HashMap<String, String>>,
    /// Source of unique document/tab ids.
    pub next_doc_id: AtomicU32,
    /// Source of unique labels for windows opened after `main`.
    pub next_window_id: AtomicU32,
    /// A file the OS asked us to open (file association / `open with`) before
    /// the frontend was ready to receive it. Drained by `take_pending_open`.
    pub pending_open: Mutex<Option<String>>,
    /// Set once the frontend has drained `pending_open`. After this, OS open
    /// requests are delivered live via the `open-file` event instead.
    pub frontend_ready: AtomicBool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn doc(path: Option<&str>) -> OpenDoc {
        OpenDoc {
            path: path.map(str::to_string),
            ..OpenDoc::default()
        }
    }

    fn assert_invariants(workspace: &Workspace) {
        let mut owned = HashSet::new();
        for window in workspace.windows.values() {
            if let Some(active) = &window.active {
                assert!(window.tabs.contains(active));
            } else {
                assert!(window.tabs.is_empty());
            }
            for id in &window.tabs {
                assert!(workspace.docs.contains_key(id));
                assert!(owned.insert(id));
            }
        }
        assert_eq!(owned.len(), workspace.docs.len());
        let paths: Vec<_> = workspace
            .docs
            .values()
            .filter_map(|doc| doc.path.as_deref())
            .collect();
        assert_eq!(paths.len(), paths.iter().collect::<HashSet<_>>().len());
    }

    #[test]
    fn open_focus_close_and_destroy_keep_ownership() {
        let mut workspace = Workspace::default();
        workspace
            .insert("left", "a".into(), doc(Some("/a.ldt")))
            .unwrap();
        workspace.insert("left", "b".into(), doc(None)).unwrap();
        workspace.insert("left", "c".into(), doc(None)).unwrap();
        workspace
            .insert("right", "d".into(), doc(Some("/d.ldt")))
            .unwrap();
        assert_invariants(&workspace);

        assert_eq!(workspace.focus_path("/a.ldt").as_deref(), Some("left"));
        assert_eq!(workspace.active_id("left").unwrap(), "a");
        assert!(workspace
            .insert("right", "duplicate".into(), doc(Some("/a.ldt")))
            .is_err());
        assert!(workspace.path_in_use_by_other("/d.ldt", "a"));
        assert_invariants(&workspace);

        workspace.close("left", Some("b")).unwrap();
        assert_eq!(workspace.active_id("left").unwrap(), "a");
        workspace.close("left", None).unwrap();
        assert_eq!(workspace.active_id("left").unwrap(), "c");
        workspace.close("left", None).unwrap();
        assert!(workspace.active_id("left").is_err());
        assert!(workspace.close("left", Some("d")).is_err());
        workspace.remove_window("left");
        assert!(workspace.doc("c").is_none());
        assert!(workspace.doc("d").is_some());
        assert_invariants(&workspace);
    }

    #[test]
    fn move_reorder_and_failed_detach_restore_selection_and_order() {
        let mut workspace = Workspace::default();
        for id in ["a", "b", "c"] {
            workspace.insert("source", id.into(), doc(None)).unwrap();
        }
        workspace.insert("target", "d".into(), doc(None)).unwrap();
        workspace.activate("source", "b").unwrap();
        workspace
            .move_tab("b", "source", "target", Some(0))
            .unwrap();
        assert_eq!(workspace.window("source").unwrap().tabs, ["a", "c"]);
        assert_eq!(workspace.active_id("source").unwrap(), "c");
        assert_eq!(workspace.window("target").unwrap().tabs, ["b", "d"]);
        assert_eq!(workspace.active_id("target").unwrap(), "b");
        workspace
            .move_tab("a", "source", "source", Some(2))
            .unwrap();
        assert_eq!(workspace.window("source").unwrap().tabs, ["c", "a"]);
        workspace.close("source", None).unwrap();
        assert_eq!(workspace.active_id("source").unwrap(), "c");
        assert_invariants(&workspace);

        let rollback = workspace.begin_detach("b", "target", "detached").unwrap();
        assert_eq!(workspace.active_id("target").unwrap(), "d");
        assert_eq!(workspace.active_id("detached").unwrap(), "b");
        workspace.rollback_detach(rollback);
        assert_eq!(workspace.window("target").unwrap().tabs, ["b", "d"]);
        assert_eq!(workspace.active_id("target").unwrap(), "b");
        assert!(workspace.window("detached").is_none());
        assert_invariants(&workspace);

        let _committed = workspace.begin_detach("b", "target", "detached").unwrap();
        workspace.remove_window("target");
        assert!(workspace.doc("b").is_some());
        assert!(workspace.doc("d").is_none());
        assert_invariants(&workspace);
    }

    #[test]
    fn failed_detach_preserves_later_source_changes() {
        let mut workspace = Workspace::default();
        for id in ["a", "b", "c"] {
            workspace.insert("source", id.into(), doc(None)).unwrap();
        }
        workspace.activate("source", "b").unwrap();
        let rollback = workspace.begin_detach("b", "source", "detached").unwrap();
        workspace.activate("source", "a").unwrap();
        workspace.rollback_detach(rollback);
        assert_eq!(workspace.active_id("source").unwrap(), "a");
        assert_eq!(workspace.window("source").unwrap().tabs, ["a", "b", "c"]);
        assert_invariants(&workspace);

        let rollback = workspace.begin_detach("b", "source", "detached").unwrap();
        workspace.close("source", Some("a")).unwrap();
        workspace.rollback_detach(rollback);
        assert_eq!(workspace.active_id("source").unwrap(), "c");
        assert_invariants(&workspace);

        workspace.close("source", Some("c")).unwrap();
        let rollback = workspace.begin_detach("b", "source", "detached").unwrap();
        workspace.rollback_detach(rollback);
        assert_eq!(workspace.active_id("source").unwrap(), "b");
        assert_invariants(&workspace);
    }

    #[test]
    fn rejected_cross_window_actions_leave_state_intact() {
        let mut workspace = Workspace::default();
        workspace.insert("source", "a".into(), doc(None)).unwrap();
        workspace.insert("target", "b".into(), doc(None)).unwrap();
        assert!(workspace.doc_in_window_mut("target", "a").is_err());
        assert!(workspace.activate("target", "a").is_err());
        assert!(workspace.move_tab("a", "missing", "target", None).is_err());
        assert!(workspace.begin_detach("a", "source", "target").is_err());
        assert_invariants(&workspace);
    }

    #[test]
    fn save_as_checks_path_before_write_and_commits_only_on_success() {
        let mut workspace = Workspace::default();
        let mut unsaved = doc(None);
        unsaved.model = Some(Eulumdat::default());
        unsaved.dirty = true;
        workspace.insert("source", "a".into(), unsaved).unwrap();
        workspace
            .insert("target", "b".into(), doc(Some("/used.ldt")))
            .unwrap();
        assert!(workspace
            .save_as("source", "/used.ldt", |_| panic!(
                "must not write duplicate"
            ))
            .is_err());
        assert!(workspace
            .save_as("source", "/new.ldt", |_| Err("write failed".into()))
            .is_err());
        assert!(workspace.doc("a").unwrap().path.is_none());
        assert!(workspace.doc("a").unwrap().dirty);
        workspace
            .save_as("source", "/new.ldt", |model| Ok(model.clone()))
            .unwrap();
        assert_eq!(
            workspace.doc("a").unwrap().path.as_deref(),
            Some("/new.ldt")
        );
        assert!(!workspace.doc("a").unwrap().dirty);
        assert_invariants(&workspace);
    }
}

//! filemon – filesystem monitoring library.
//!
//! # Example
//! ```no_run
//! use filemon::{MonitorConfig, Action, EventFilter};
//! use std::path::PathBuf;
//!
//! let config = MonitorConfig {
//!     targets: vec![PathBuf::from("/tmp/watch_me")],
//!     action: Action::Copy,
//!     dest: Some(PathBuf::from("/tmp/backup")),
//!     events: EventFilter { on_modify: true, ..Default::default() },
//!     recursive: true,
//!     debounce_ms: 300,
//! };
//! filemon::start_monitor(config).unwrap();
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{
    Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

// ── Public types ─────────────────────────────────────────────────────────────

/// What to do when an event fires on a watched path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Copy the changed file into `dest`.
    Copy,
    /// Move (rename) the changed file into `dest`.
    Move,
    /// Delete the changed file.
    Delete,
}

/// Which filesystem event kinds should trigger the action.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Trigger on file / directory creation.
    pub on_create: bool,
    /// Trigger on file content or metadata modification.
    pub on_modify: bool,
    /// Trigger on rename or move.
    pub on_rename: bool,
    /// Trigger on deletion.
    pub on_delete: bool,
}

impl EventFilter {
    /// Returns `true` when at least one event kind is enabled.
    pub fn any_enabled(&self) -> bool {
        self.on_create || self.on_modify || self.on_rename || self.on_delete
    }

    /// Returns `true` when this event kind should trigger an action.
    pub fn matches(&self, kind: &EventKind) -> bool {
        match kind {
            EventKind::Create(_) => self.on_create,
            EventKind::Modify(_) => self.on_modify,
            EventKind::Remove(_) => self.on_delete,
            EventKind::Access(_) => false,
            EventKind::Other => false,
            // Rename / move shows up as EventKind::Modify(ModifyKind::Name(_))
            // on some platforms and EventKind::Rename on others; both are
            // matched above via on_rename when we check explicitly below.
            _ => false,
        }
    }

    /// Variant-aware match that handles rename across platforms.
    pub fn matches_event(&self, kind: &EventKind) -> bool {
        use notify::event::{ModifyKind, RenameMode};
        match kind {
            EventKind::Create(_) => self.on_create,
            EventKind::Remove(_) => self.on_delete,
            EventKind::Modify(ModifyKind::Name(RenameMode::From))
            | EventKind::Modify(ModifyKind::Name(RenameMode::To))
            | EventKind::Modify(ModifyKind::Name(RenameMode::Both))
            | EventKind::Modify(ModifyKind::Name(RenameMode::Any)) => self.on_rename,
            EventKind::Modify(_) => self.on_modify,
            _ => false,
        }
    }
}

/// Full configuration passed to [`start_monitor`].
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Paths (files or directories) to watch.
    pub targets: Vec<PathBuf>,
    /// Action to perform when an event fires.
    pub action: Action,
    /// Destination directory for `Copy` and `Move` actions.
    /// Ignored for `Delete`.
    pub dest: Option<PathBuf>,
    /// Which event kinds trigger the action.
    pub events: EventFilter,
    /// Watch directories recursively (default `true`).
    pub recursive: bool,
    /// Debounce delay in milliseconds before an event is delivered.
    pub debounce_ms: u64,
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum FilemonError {
    Notify(notify::Error),
    Io(std::io::Error),
    Config(String),
}

impl std::fmt::Display for FilemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilemonError::Notify(e) => write!(f, "watcher error: {e}"),
            FilemonError::Io(e) => write!(f, "io error: {e}"),
            FilemonError::Config(s) => write!(f, "config error: {s}"),
        }
    }
}

impl std::error::Error for FilemonError {}

impl From<notify::Error> for FilemonError {
    fn from(e: notify::Error) -> Self {
        FilemonError::Notify(e)
    }
}
impl From<std::io::Error> for FilemonError {
    fn from(e: std::io::Error) -> Self {
        FilemonError::Io(e)
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Start monitoring and **block** until the process is interrupted (Ctrl-C).
///
/// Each watched event triggers the configured action.  For `Copy` and `Move`
/// a `dest` directory must be supplied in the config.
pub fn start_monitor(config: MonitorConfig) -> Result<(), FilemonError> {
    validate_config(&config)?;

    if let Some(dest) = &config.dest {
        fs::create_dir_all(dest)?;
    }

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    let debounce = Duration::from_millis(config.debounce_ms);
    let notify_cfg = NotifyConfig::default().with_poll_interval(debounce);

    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, notify_cfg)?;

    let mode = if config.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    for target in &config.targets {
        watcher.watch(target, mode)?;
        eprintln!("[filemon] watching {}", target.display());
    }

    for event_result in rx {
        match event_result {
            Err(e) => eprintln!("[filemon] watcher error: {e}"),
            Ok(event) => {
                if !config.events.matches_event(&event.kind) {
                    continue;
                }
                for path in &event.paths {
                    if let Err(e) = apply_action(path, &config) {
                        eprintln!(
                            "[filemon] action failed for {}: {e}",
                            path.display()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn validate_config(cfg: &MonitorConfig) -> Result<(), FilemonError> {
    if cfg.targets.is_empty() {
        return Err(FilemonError::Config("no targets specified".into()));
    }
    if !cfg.events.any_enabled() {
        return Err(FilemonError::Config(
            "no events enabled: use at least one of --on-create / --on-modify / --on-rename / --on-delete".into(),
        ));
    }
    if matches!(cfg.action, Action::Copy | Action::Move) && cfg.dest.is_none() {
        return Err(FilemonError::Config(
            "--dest is required for copy/move actions".into(),
        ));
    }
    Ok(())
}

fn apply_action(src: &Path, cfg: &MonitorConfig) -> Result<(), FilemonError> {
    match &cfg.action {
        Action::Copy => {
            let dest_dir = cfg.dest.as_ref().unwrap();
            let dest_path = dest_file_path(src, dest_dir);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            if src.is_file() {
                fs::copy(src, &dest_path)?;
                eprintln!(
                    "[filemon] copied {} → {}",
                    src.display(),
                    dest_path.display()
                );
            }
        }
        Action::Move => {
            let dest_dir = cfg.dest.as_ref().unwrap();
            let dest_path = dest_file_path(src, dest_dir);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            if src.exists() {
                fs::rename(src, &dest_path)?;
                eprintln!(
                    "[filemon] moved {} → {}",
                    src.display(),
                    dest_path.display()
                );
            }
        }
        Action::Delete => {
            if src.is_file() {
                fs::remove_file(src)?;
                eprintln!("[filemon] deleted {}", src.display());
            } else if src.is_dir() {
                fs::remove_dir_all(src)?;
                eprintln!("[filemon] deleted dir {}", src.display());
            }
        }
    }
    Ok(())
}

/// Build the destination path: `dest_dir / filename`.
fn dest_file_path(src: &Path, dest_dir: &Path) -> PathBuf {
    let filename = src.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown"));
    dest_dir.join(filename)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn modify_only() -> EventFilter {
        EventFilter { on_modify: true, ..Default::default() }
    }

    fn all_events() -> EventFilter {
        EventFilter {
            on_create: true,
            on_modify: true,
            on_rename: true,
            on_delete: true,
        }
    }

    fn basic_config(action: Action, dest: Option<PathBuf>, targets: Vec<PathBuf>) -> MonitorConfig {
        MonitorConfig {
            targets,
            action,
            dest,
            events: modify_only(),
            recursive: true,
            debounce_ms: 100,
        }
    }

    // ── validate_config ───────────────────────────────────────────────────────

    #[test]
    fn validate_rejects_empty_targets() {
        let cfg = basic_config(Action::Delete, None, vec![]);
        let err = validate_config(&cfg).unwrap_err();
        assert!(matches!(err, FilemonError::Config(_)));
    }

    #[test]
    fn validate_rejects_no_events() {
        let cfg = MonitorConfig {
            targets: vec![PathBuf::from(".")],
            action: Action::Delete,
            dest: None,
            events: EventFilter::default(), // all false
            recursive: true,
            debounce_ms: 100,
        };
        let err = validate_config(&cfg).unwrap_err();
        assert!(matches!(err, FilemonError::Config(_)));
    }

    #[test]
    fn validate_rejects_copy_without_dest() {
        let cfg = basic_config(Action::Copy, None, vec![PathBuf::from(".")]);
        let err = validate_config(&cfg).unwrap_err();
        assert!(matches!(err, FilemonError::Config(_)));
    }

    #[test]
    fn validate_rejects_move_without_dest() {
        let cfg = basic_config(Action::Move, None, vec![PathBuf::from(".")]);
        let err = validate_config(&cfg).unwrap_err();
        assert!(matches!(err, FilemonError::Config(_)));
    }

    #[test]
    fn validate_allows_delete_without_dest() {
        let cfg = basic_config(Action::Delete, None, vec![PathBuf::from(".")]);
        assert!(validate_config(&cfg).is_ok());
    }

    #[test]
    fn validate_allows_copy_with_dest() {
        let dir = tempdir().unwrap();
        let cfg = basic_config(
            Action::Copy,
            Some(dir.path().to_path_buf()),
            vec![PathBuf::from(".")],
        );
        assert!(validate_config(&cfg).is_ok());
    }

    // ── EventFilter ───────────────────────────────────────────────────────────

    #[test]
    fn event_filter_any_enabled_false_when_all_off() {
        assert!(!EventFilter::default().any_enabled());
    }

    #[test]
    fn event_filter_any_enabled_true_when_one_on() {
        let f = EventFilter { on_rename: true, ..Default::default() };
        assert!(f.any_enabled());
    }

    #[test]
    fn event_filter_matches_create() {
        use notify::event::CreateKind;
        let f = all_events();
        assert!(f.matches_event(&EventKind::Create(CreateKind::File)));
        let f_off = EventFilter { on_modify: true, ..Default::default() };
        assert!(!f_off.matches_event(&EventKind::Create(CreateKind::File)));
    }

    #[test]
    fn event_filter_matches_modify() {
        use notify::event::{ModifyKind, DataChange};
        let f = all_events();
        assert!(f.matches_event(&EventKind::Modify(ModifyKind::Data(DataChange::Content))));
        let f_off = EventFilter { on_create: true, ..Default::default() };
        assert!(!f_off.matches_event(&EventKind::Modify(ModifyKind::Data(DataChange::Content))));
    }

    #[test]
    fn event_filter_matches_rename() {
        use notify::event::{ModifyKind, RenameMode};
        let f = all_events();
        assert!(f.matches_event(&EventKind::Modify(ModifyKind::Name(RenameMode::Any))));
        let f_off = EventFilter { on_modify: true, ..Default::default() };
        // rename kind should NOT match on_modify
        assert!(!f_off.matches_event(&EventKind::Modify(ModifyKind::Name(RenameMode::Any))));
    }

    #[test]
    fn event_filter_matches_delete() {
        use notify::event::RemoveKind;
        let f = all_events();
        assert!(f.matches_event(&EventKind::Remove(RemoveKind::File)));
        let f_off = EventFilter { on_modify: true, ..Default::default() };
        assert!(!f_off.matches_event(&EventKind::Remove(RemoveKind::File)));
    }

    #[test]
    fn event_filter_ignores_access() {
        use notify::event::AccessKind;
        let f = all_events();
        assert!(!f.matches_event(&EventKind::Access(AccessKind::Read)));
    }

    // ── dest_file_path ────────────────────────────────────────────────────────

    #[test]
    fn dest_file_path_appends_filename() {
        let result = dest_file_path(Path::new("/some/dir/file.txt"), Path::new("/dest"));
        assert_eq!(result, PathBuf::from("/dest/file.txt"));
    }

    #[test]
    fn dest_file_path_nested_src() {
        let result = dest_file_path(Path::new("/a/b/c/deep.bin"), Path::new("/out"));
        assert_eq!(result, PathBuf::from("/out/deep.bin"));
    }

    // ── apply_action ──────────────────────────────────────────────────────────

    #[test]
    fn apply_action_copy_creates_dest_file() {
        let src_dir = tempdir().unwrap();
        let dest_dir = tempdir().unwrap();
        let src_file = src_dir.path().join("hello.txt");
        fs::write(&src_file, b"hello").unwrap();

        let cfg = basic_config(
            Action::Copy,
            Some(dest_dir.path().to_path_buf()),
            vec![src_dir.path().to_path_buf()],
        );

        apply_action(&src_file, &cfg).unwrap();

        let dest = dest_dir.path().join("hello.txt");
        assert!(dest.exists(), "copied file should exist at dest");
        assert_eq!(fs::read(dest).unwrap(), b"hello");
        // source should still be there
        assert!(src_file.exists());
    }

    #[test]
    fn apply_action_copy_nonexistent_file_is_noop() {
        let dest_dir = tempdir().unwrap();
        let cfg = basic_config(
            Action::Copy,
            Some(dest_dir.path().to_path_buf()),
            vec![PathBuf::from(".")],
        );
        // Should not error even if the file doesn't exist (is_file() returns false)
        apply_action(Path::new("/nonexistent/ghost.txt"), &cfg).unwrap();
        assert_eq!(fs::read_dir(dest_dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn apply_action_move_relocates_file() {
        let src_dir = tempdir().unwrap();
        let dest_dir = tempdir().unwrap();
        let src_file = src_dir.path().join("move_me.txt");
        fs::write(&src_file, b"data").unwrap();

        let cfg = basic_config(
            Action::Move,
            Some(dest_dir.path().to_path_buf()),
            vec![src_dir.path().to_path_buf()],
        );

        apply_action(&src_file, &cfg).unwrap();

        let dest = dest_dir.path().join("move_me.txt");
        assert!(dest.exists(), "moved file should exist at dest");
        assert!(!src_file.exists(), "source file should be gone after move");
    }

    #[test]
    fn apply_action_delete_removes_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("delete_me.txt");
        fs::write(&file, b"bye").unwrap();
        assert!(file.exists());

        let cfg = basic_config(Action::Delete, None, vec![dir.path().to_path_buf()]);
        apply_action(&file, &cfg).unwrap();

        assert!(!file.exists(), "file should be deleted");
    }

    #[test]
    fn apply_action_delete_nonexistent_is_noop() {
        let cfg = basic_config(Action::Delete, None, vec![PathBuf::from(".")]);
        apply_action(Path::new("/nonexistent/ghost.txt"), &cfg).unwrap();
    }

    #[test]
    fn apply_action_copy_creates_dest_dir_if_missing() {
        let src_dir = tempdir().unwrap();
        let dest_base = tempdir().unwrap();
        // Dest dir does not exist yet
        let dest_dir = dest_base.path().join("new_subdir");
        assert!(!dest_dir.exists());

        let src_file = src_dir.path().join("data.bin");
        fs::write(&src_file, b"xyz").unwrap();

        let cfg = basic_config(
            Action::Copy,
            Some(dest_dir.clone()),
            vec![src_dir.path().to_path_buf()],
        );
        apply_action(&src_file, &cfg).unwrap();

        assert!(dest_dir.join("data.bin").exists());
    }
}

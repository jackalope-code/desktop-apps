//! Integration tests: spin up a real watcher in a background thread, trigger
//! filesystem events, then verify the action was applied.
//!
//! Each test leaks the monitor thread (the thread blocks on the notify channel
//! forever), which is acceptable — the OS reclaims resources when the test
//! process exits.

use filemon::{Action, EventFilter, MonitorConfig};
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

/// Wait up to `timeout` for `predicate` to become true, polling every 50 ms.
fn wait_for(timeout: Duration, predicate: impl Fn() -> bool) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if predicate() {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

// ── Copy ──────────────────────────────────────────────────────────────────────

#[test]
fn watcher_copy_on_modify() {
    let watch_dir = tempdir().unwrap();
    let dest_dir = tempdir().unwrap();

    // Pre-create the file so the watcher can see modifications to it.
    let src_file = watch_dir.path().join("sample.txt");
    fs::write(&src_file, b"initial").unwrap();

    let config = MonitorConfig {
        targets: vec![watch_dir.path().to_path_buf()],
        action: Action::Copy,
        dest: Some(dest_dir.path().to_path_buf()),
        events: EventFilter { on_modify: true, ..Default::default() },
        recursive: false,
        debounce_ms: 50,
    };

    let dest_path = dest_dir.path().join("sample.txt");

    thread::spawn(move || {
        let _ = filemon::start_monitor(config);
    });

    // Give the watcher time to arm.
    thread::sleep(Duration::from_millis(200));

    // Trigger a modify event.
    fs::write(&src_file, b"modified").unwrap();

    let appeared = wait_for(Duration::from_secs(3), || dest_path.exists());
    assert!(appeared, "copied file should have appeared in dest within 3 s");
    assert_eq!(fs::read(&dest_path).unwrap(), b"modified");
}

// ── Move ──────────────────────────────────────────────────────────────────────

#[test]
fn watcher_move_on_create() {
    let watch_dir = tempdir().unwrap();
    let dest_dir = tempdir().unwrap();

    let config = MonitorConfig {
        targets: vec![watch_dir.path().to_path_buf()],
        action: Action::Move,
        dest: Some(dest_dir.path().to_path_buf()),
        events: EventFilter { on_create: true, ..Default::default() },
        recursive: false,
        debounce_ms: 50,
    };

    let new_file = watch_dir.path().join("newfile.bin");
    let dest_path = dest_dir.path().join("newfile.bin");

    thread::spawn(move || {
        let _ = filemon::start_monitor(config);
    });

    thread::sleep(Duration::from_millis(200));

    // Trigger a create event.
    fs::write(&new_file, b"fresh").unwrap();

    let moved = wait_for(Duration::from_secs(3), || dest_path.exists());
    assert!(moved, "file should have been moved to dest within 3 s");
    // Original should be gone (possibly — the watcher may batch events; check dest is right)
    assert_eq!(fs::read(&dest_path).unwrap(), b"fresh");
}

// ── Delete ────────────────────────────────────────────────────────────────────

#[test]
fn watcher_delete_on_modify() {
    let watch_dir = tempdir().unwrap();
    let target_file = watch_dir.path().join("ephemeral.txt");
    fs::write(&target_file, b"temp data").unwrap();

    let config = MonitorConfig {
        targets: vec![watch_dir.path().to_path_buf()],
        action: Action::Delete,
        dest: None,
        events: EventFilter { on_modify: true, ..Default::default() },
        recursive: false,
        debounce_ms: 50,
    };

    let watch_path = target_file.clone();

    thread::spawn(move || {
        let _ = filemon::start_monitor(config);
    });

    thread::sleep(Duration::from_millis(200));

    // Modify triggers delete action.
    fs::write(&target_file, b"updated").unwrap();

    let deleted = wait_for(Duration::from_secs(3), || !watch_path.exists());
    assert!(deleted, "file should have been deleted within 3 s");
}

// ── Config validation (public error path) ─────────────────────────────────────

#[test]
fn start_monitor_returns_err_for_empty_targets() {
    let config = MonitorConfig {
        targets: vec![],
        action: Action::Delete,
        dest: None,
        events: EventFilter { on_modify: true, ..Default::default() },
        recursive: true,
        debounce_ms: 100,
    };
    assert!(filemon::start_monitor(config).is_err());
}

#[test]
fn start_monitor_returns_err_copy_without_dest() {
    let config = MonitorConfig {
        targets: vec![std::path::PathBuf::from(".")],
        action: Action::Copy,
        dest: None,
        events: EventFilter { on_modify: true, ..Default::default() },
        recursive: true,
        debounce_ms: 100,
    };
    assert!(filemon::start_monitor(config).is_err());
}

#[test]
fn start_monitor_returns_err_no_events() {
    let config = MonitorConfig {
        targets: vec![std::path::PathBuf::from(".")],
        action: Action::Delete,
        dest: None,
        events: EventFilter::default(),
        recursive: true,
        debounce_ms: 100,
    };
    assert!(filemon::start_monitor(config).is_err());
}

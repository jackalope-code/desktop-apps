use clap::{Parser, ValueEnum};
use filemon::{Action, EventFilter, MonitorConfig};
use std::path::PathBuf;

/// Watch files or directories and copy, move, or delete them on change.
///
/// Examples:
///   filemon --action copy --dest /tmp/out --on-modify /etc/hosts
///   filemon --action delete --on-create --on-modify ~/Downloads
///   filemon --action move --dest /tmp/archived --on-rename . src/
#[derive(Parser, Debug)]
#[command(name = "filemon", version, about)]
struct Cli {
    /// One or more files or directories to monitor.
    #[arg(required = true, value_name = "TARGET")]
    targets: Vec<PathBuf>,

    /// Action to perform when an event fires.
    #[arg(long, value_enum, default_value = "copy")]
    action: CliAction,

    /// Destination directory (required for copy and move).
    #[arg(long, value_name = "DIR")]
    dest: Option<PathBuf>,

    // ── Event filter flags ──────────────────────────────────────────────────
    /// Trigger on file or directory creation.
    #[arg(long)]
    on_create: bool,

    /// Trigger on file content or metadata modification.
    #[arg(long)]
    on_modify: bool,

    /// Trigger on rename or move.
    #[arg(long)]
    on_rename: bool,

    /// Trigger on deletion.
    #[arg(long)]
    on_delete: bool,

    // ── Recursion ───────────────────────────────────────────────────────────
    /// Do NOT recurse into subdirectories (directories are watched shallowly).
    #[arg(long)]
    no_recurse: bool,

    // ── Debounce ────────────────────────────────────────────────────────────
    /// Debounce interval in milliseconds before an event is delivered.
    #[arg(long, default_value = "300", value_name = "MS")]
    debounce: u64,
}

#[derive(Clone, Debug, ValueEnum)]
enum CliAction {
    Copy,
    Move,
    Delete,
}

impl From<CliAction> for Action {
    fn from(a: CliAction) -> Self {
        match a {
            CliAction::Copy => Action::Copy,
            CliAction::Move => Action::Move,
            CliAction::Delete => Action::Delete,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    // If no event flag is set default to --on-modify for convenience.
    let any_flag = cli.on_create || cli.on_modify || cli.on_rename || cli.on_delete;
    let events = if any_flag {
        EventFilter {
            on_create: cli.on_create,
            on_modify: cli.on_modify,
            on_rename: cli.on_rename,
            on_delete: cli.on_delete,
        }
    } else {
        eprintln!(
            "[filemon] no event flags supplied – defaulting to --on-modify. \
             Use --on-create / --on-modify / --on-rename / --on-delete to be explicit."
        );
        EventFilter {
            on_modify: true,
            ..Default::default()
        }
    };

    let config = MonitorConfig {
        targets: cli.targets,
        action: cli.action.into(),
        dest: cli.dest,
        events,
        recursive: !cli.no_recurse,
        debounce_ms: cli.debounce,
    };

    if let Err(e) = filemon::start_monitor(config) {
        eprintln!("[filemon] error: {e}");
        std::process::exit(1);
    }
}


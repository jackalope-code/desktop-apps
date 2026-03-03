//! fast-secret-scanner – library API.
//!
//! Scans files, directories, and git commit history for secrets using
//! configurable regex rules.

pub mod patterns;
pub mod scanner;
pub mod types;

pub use patterns::{default_rules, user_rule, Rule};
pub use scanner::{apply_gitignore, is_git_ignored, load_fssignore, scan_directory, scan_file, scan_git_history};
pub use types::{Finding, Location, ScanConfig, Severity};

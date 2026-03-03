use serde::Serialize;
use std::path::PathBuf;

// ── Severity ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    /// Informational / all env vars.
    Warning,
    /// Possibly sensitive pattern.
    Medium,
    /// Likely credential / token.
    High,
    /// Confirmed private key / known secret format.
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Warning  => write!(f, "WARNING"),
            Severity::Medium   => write!(f, "MEDIUM"),
            Severity::High     => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ── Location ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum Location {
    File {
        path: String,
        line_no: usize,
    },
    Commit {
        hash: String,
        short_hash: String,
        file: String,
        commit_message: String,
    },
}

// ── Finding ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule_name: String,
    pub severity: Severity,
    pub location: Location,
    /// The portion of the line that matched the regex (redacted for high/critical).
    pub matched_text: String,
    /// Full source line.
    pub line_content: String,
    /// True when the file containing this secret is covered by the repo's
    /// .gitignore rules (so the secret cannot be accidentally pushed).
    /// Always false for commit-history findings.
    #[serde(default)]
    pub git_ignored: bool,
}

// ── ScanConfig ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// All active rules (built-in + user-supplied).
    pub rules: Vec<crate::patterns::Rule>,
    /// Paths/names to ignore (files or directories).
    pub ignore: Vec<PathBuf>,
    /// Whether to scan git commit history in addition to working tree.
    pub scan_history: bool,
    /// Redact matched values in output (default: true). Pass false only when
    /// `--unredact` is explicitly requested by the user.
    pub redact: bool,
    /// Patterns from `.fssignore` – relative path prefixes to skip entirely.
    /// Lines starting with `#` and blank lines are ignored.  Trailing `/`
    /// denotes a directory prefix; otherwise an exact relative-path suffix match.
    pub fssignore: Vec<String>,
}

impl ScanConfig {
    pub fn is_ignored(&self, path: &std::path::Path) -> bool {
        for ign in &self.ignore {
            // Match by full path component or filename
            if path == ign {
                return true;
            }
            if path.starts_with(ign) {
                return true;
            }
            // Match by name segment
            for component in path.components() {
                if std::path::Path::new(component.as_os_str()) == ign.as_path() {
                    return true;
                }
            }
        }
        false
    }
}

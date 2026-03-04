use clap::Parser;
use colored::Colorize;
use fast_secret_scanner::{
    patterns::{default_rules, user_rule},
    scanner::{apply_gitignore, load_fssignore, scan_directory, scan_git_history},
    types::{Finding, Location, ScanConfig, Severity},
};
use std::path::PathBuf;

/// Fast Secret Scanner – find credentials and secrets in source code and git history.
///
/// Examples:
///   fss --git-dir .
///   fss --gh-repo owner/repo --ignore vendor node_modules
///   fss --git-dir . --no-history --json
///   fss --git-dir . --pattern "mytoken_[A-Za-z0-9]{32}"
#[derive(Parser, Debug)]
#[command(name = "fss", version, about, max_term_width = 100)]
struct Cli {
    /// Scan a GitHub repository (e.g. owner/repo). Clones it to a temp directory.
    #[arg(long, value_name = "OWNER/REPO", conflicts_with = "git_dir")]
    gh_repo: Option<String>,

    /// Scan a local git repository directory.
    #[arg(long, value_name = "PATH", conflicts_with = "gh_repo")]
    git_dir: Option<PathBuf>,

    /// Skip git commit history scan (working tree only).
    #[arg(long)]
    no_history: bool,

    /// Do not warn about hardcoded home-directory paths found in source files.
    #[arg(long)]
    keep_user_dir: bool,

    /// Paths or directory names to ignore (can be specified multiple times).
    #[arg(long, value_name = "PATH", num_args = 1.., action = clap::ArgAction::Append)]
    ignore: Vec<PathBuf>,

    /// Additional custom regex patterns to scan for (can be specified multiple times).
    #[arg(long, value_name = "REGEX", num_args = 1.., action = clap::ArgAction::Append)]
    pattern: Vec<String>,

    /// Output findings as JSON instead of colored text.
    #[arg(long)]
    json: bool,

    /// Minimum severity to report: warning | medium | high | critical  [default: warning]
    #[arg(long, value_name = "LEVEL", default_value = "warning")]
    min_severity: String,

    /// Show the full matched secret value in output instead of redacting it.
    #[arg(long)]
    unredact: bool,

    /// GitHub token for cloning private repositories (GITHUB_TOKEN env var also works).
    #[arg(long, value_name = "TOKEN", env = "GITHUB_TOKEN")]
    token: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // ── Build rules ────────────────────────────────────────────────────────
    let mut rules = default_rules();
    if cli.keep_user_dir {
        rules.retain(|r| r.name != "user-dir-path");
    }
    for (i, pat) in cli.pattern.iter().enumerate() {
        rules.push(user_rule(i, pat)?);
    }

    // ── Always add common ignores ──────────────────────────────────────────
    let mut ignore = cli.ignore.clone();
    for auto in ["node_modules", ".git", "target", "dist", "__pycache__", ".next"] {
        ignore.push(PathBuf::from(auto));
    }

    let min_sev = parse_severity(&cli.min_severity)?;

    // ── Resolve repo path ──────────────────────────────────────────────────
    let (repo_path, _temp_dir) = if let Some(ref slug) = cli.gh_repo {
        let td = clone_github_repo(slug, cli.token.as_deref())?;
        let path = td.path().to_path_buf();
        (path, Some(td))
    } else if let Some(ref dir) = cli.git_dir {
        let canonical = dir.canonicalize()
            .unwrap_or_else(|_| dir.clone());
        (canonical, None)
    } else {
        eprintln!("{}", "Error: provide --git-dir or --gh-repo".red().bold());
        std::process::exit(1);
    };

    if !cli.json {
        eprintln!("{} {}", "Scanning".cyan().bold(), repo_path.display());
    }

    let cfg = ScanConfig {
        rules,
        ignore,
        scan_history: !cli.no_history,
        redact: !cli.unredact,
        fssignore: load_fssignore(&repo_path),
    };

    let mut findings: Vec<Finding> = Vec::new();

    // ── Working tree ───────────────────────────────────────────────────────
    scan_directory(&repo_path, &cfg, &mut findings)?;

    // ── Commit history ─────────────────────────────────────────────────────
    if cfg.scan_history {
        if !cli.json {
            eprintln!("{}", "Scanning git commit history…".cyan());
        }
        match scan_git_history(&repo_path, &cfg, &mut findings) {
            Ok(()) => {}
            Err(e) => eprintln!("{} {}", "Warning: git history scan failed:".yellow(), e),
        }
    }

    // ── Filter by severity ─────────────────────────────────────────────────
    findings.retain(|f| f.severity >= min_sev);

    // Deduplicate: same rule + same matched_text + same file
    findings.dedup_by(|a, b| {
        a.rule_name == b.rule_name
            && a.matched_text == b.matched_text
            && loc_key(&a.location) == loc_key(&b.location)
    });

    // ── Check .gitignore coverage (file findings only) ─────────────────────
    // Commit-history findings are *never* considered covered — the secret is
    // already baked into the git object store and .gitignore cannot protect it.
    apply_gitignore(&repo_path, &mut findings);

    // Partition: actively-exposed secrets vs. safely .gitignore'd
    let (covered, active): (Vec<Finding>, Vec<Finding>) =
        findings.into_iter().partition(|f| f.git_ignored);

    // ── Output ─────────────────────────────────────────────────────────────
    if cli.json {
        #[derive(serde::Serialize)]
        struct JsonOutput {
            active_findings: Vec<Finding>,
            covered_by_gitignore: Vec<Finding>,
        }
        println!("{}", serde_json::to_string_pretty(&JsonOutput {
            active_findings: active.clone(),
            covered_by_gitignore: covered.clone(),
        })?);
    } else {
        if !covered.is_empty() {
            print_covered_findings(&covered);
        }
        if !active.is_empty() {
            print_findings(&active);
        }
    }

    // Exit 0 when there are no uncovered secrets (covered-only = success).
    if active.is_empty() {
        if !cli.json {
            if covered.is_empty() {
                println!("{}", "No secrets found.".green().bold());
            } else {
                println!(
                    "\n{} {} secret(s) found but all covered by .gitignore – safe to commit.",
                    "✓".green().bold(),
                    covered.len()
                );
            }
        }
        Ok(())
    } else {
        if !cli.json {
            let covered_note = if covered.is_empty() {
                String::new()
            } else {
                format!("  ({} additional finding(s) covered by .gitignore)", covered.len())
            };
            println!(
                "\n{} {} active finding(s) require attention.{}",
                "⚠".yellow().bold(),
                active.len(),
                covered_note.dimmed(),
            );
        }
        std::process::exit(1);
    }
}

// ── .gitignore coverage check ────────────────────────────────────────────────
// (logic lives in fast_secret_scanner::scanner::apply_gitignore)

// ── Clone helper ──────────────────────────────────────────────────────────────

fn clone_github_repo(slug: &str, token: Option<&str>) -> anyhow::Result<tempfile::TempDir> {
    use git2::{RemoteCallbacks, FetchOptions, Config};

    let url = if slug.starts_with("https://") || slug.starts_with("git@") {
        slug.to_string()
    } else {
        format!("https://github.com/{slug}.git")
    };

    eprintln!("{} {url}", "Cloning".cyan().bold());

    let td = tempfile::tempdir()?;

    let mut builder = git2::build::RepoBuilder::new();

    let token_owned = token.map(|t| t.to_string());
    let git_config = Config::open_default().ok();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        // 1. Explicit token takes priority.
        if let Some(ref tok) = token_owned {
            return git2::Cred::userpass_plaintext("x-access-token", tok);
        }
        // 2. SSH agent (for git@ URLs).
        if allowed.contains(git2::CredentialType::SSH_KEY) {
            let user = username.unwrap_or("git");
            if let Ok(c) = git2::Cred::ssh_key_from_agent(user) {
                return Ok(c);
            }
        }
        // 3. System credential helper (Windows Credential Manager, macOS keychain, etc.)
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            if let Some(ref cfg) = git_config {
                if let Ok(c) = git2::Cred::credential_helper(cfg, url, username) {
                    return Ok(c);
                }
            }
        }
        // 4. Default (Kerberos / NTLM / negotiate).
        if allowed.contains(git2::CredentialType::DEFAULT) {
            if let Ok(c) = git2::Cred::default() {
                return Ok(c);
            }
        }
        Err(git2::Error::from_str(
            "authentication required – pass --token or set GITHUB_TOKEN",
        ))
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    builder.fetch_options(fetch_opts);

    builder
        .clone(&url, td.path())
        .map_err(|e| anyhow::anyhow!("failed to clone '{}': {}", url, e))?;

    Ok(td)
}

// ── Severity parse helper ────────────────────────────────────────────────────

fn parse_severity(s: &str) -> anyhow::Result<Severity> {
    match s.to_ascii_lowercase().as_str() {
        "warning"  | "warn" | "w" => Ok(Severity::Warning),
        "medium"   | "med"  | "m" => Ok(Severity::Medium),
        "high"     | "h"          => Ok(Severity::High),
        "critical" | "crit" | "c" => Ok(Severity::Critical),
        other => anyhow::bail!("unknown severity '{}'; use warning/medium/high/critical", other),
    }
}

// ── Location key for dedup ────────────────────────────────────────────────────

fn loc_key(loc: &Location) -> String {
    match loc {
        Location::File { path, line_no } => format!("file:{path}:{line_no}"),
        Location::Commit { hash, file, .. } => format!("commit:{hash}:{file}"),
    }
}

// ── Colored output ────────────────────────────────────────────────────────────

fn severity_color(s: &Severity) -> colored::ColoredString {
    match s {
        Severity::Critical => format!("[{s}]").red().bold(),
        Severity::High     => format!("[{s}]").yellow().bold(),
        Severity::Medium   => format!("[{s}]").magenta(),
        Severity::Warning  => format!("[{s}]").cyan(),
    }
}

/// Print findings that are covered by .gitignore — success, but informational.
fn print_covered_findings(findings: &[Finding]) {
    for f in findings {
        let rule = f.rule_name.bold();
        match &f.location {
            Location::File { path, line_no } => {
                println!(
                    "{} {} {rule}  {}:{}",
                    "✓".green().bold(),
                    "[COVERED]".green().bold(),
                    path.blue(),
                    line_no.to_string().dimmed(),
                );
            }
            Location::Commit { short_hash, file, .. } => {
                println!(
                    "{} {} {rule}  commit {} {}",
                    "✓".green().bold(),
                    "[COVERED]".green().bold(),
                    short_hash.yellow(),
                    file.blue(),
                );
            }
        }
        println!("      match:   {}", f.matched_text.yellow());
        println!("      line:    {}", f.line_content.dimmed());
        println!();
    }
}

fn print_findings(findings: &[Finding]) {
    for f in findings {
        let sev = severity_color(&f.severity);
        let rule = f.rule_name.bold();

        match &f.location {
            Location::File { path, line_no } => {
                println!(
                    "{sev} {rule}  {}:{}",
                    path.blue(),
                    line_no.to_string().dimmed(),
                );
            }
            Location::Commit { short_hash, file, commit_message, .. } => {
                println!(
                    "{sev} {rule}  commit {} {}  {}",
                    short_hash.yellow(),
                    file.blue(),
                    commit_message.dimmed(),
                );
            }
        }
        println!(
            "      match:   {}",
            f.matched_text.yellow()
        );
        println!(
            "      line:    {}",
            f.line_content.dimmed()
        );
        println!();
    }
}


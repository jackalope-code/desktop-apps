use crate::types::{Finding, Location, ScanConfig};
use crate::patterns::Rule;
use anyhow::Context;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ── .gitignore coverage check ─────────────────────────────────────────────────────

/// Returns true if `abs_path` is covered by the repo's .gitignore rules.
/// Respects all gitignore layers (.gitignore, .git/info/exclude, global).
/// Always returns false for bare repos or paths outside the working tree.
pub fn is_git_ignored(repo: &git2::Repository, repo_path: &Path, abs_path: &str) -> bool {
    let workdir = match repo.workdir() {
        Some(w) => w,
        None => return false,
    };

    let abs = Path::new(abs_path);

    // Try to get a relative path. Try repo_path first (already canonicalized),
    // then workdir, then canonicalize both to handle Windows UNC prefixes.
    let rel: PathBuf = if let Ok(r) = abs.strip_prefix(repo_path) {
        r.to_path_buf()
    } else if let Ok(r) = abs.strip_prefix(workdir) {
        r.to_path_buf()
    } else {
        let canon_root = repo_path.canonicalize()
            .unwrap_or_else(|_| repo_path.to_path_buf());
        let canon_abs = abs.canonicalize()
            .unwrap_or_else(|_| abs.to_path_buf());
        match canon_abs.strip_prefix(&canon_root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => return false,
        }
    };

    repo.is_path_ignored(&rel).unwrap_or(false)
}

/// Apply .gitignore coverage to a set of findings.
/// File findings whose path is git-ignored get `git_ignored = true`.
/// Commit-history findings are never marked as covered (already in history).
pub fn apply_gitignore(repo_path: &Path, findings: &mut Vec<Finding>) {
    if let Ok(repo) = git2::Repository::open(repo_path) {
        for f in findings.iter_mut() {
            if let Location::File { path, .. } = &f.location {
                f.git_ignored = is_git_ignored(&repo, repo_path, path);
            }
        }
    }
}

// ── is-env-file helper ───────────────────────────────────────────────────────

fn is_env_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    // .env, .env.local, .env.production, env.example, etc.
    name == ".env"
        || name.starts_with(".env.")
        || name.ends_with(".env")
        || name == "env"
        || name.contains(".env")
}

// ── Single-line scanner ──────────────────────────────────────────────────────

fn scan_line(
    line: &str,
    line_no: usize,
    rules: &[Rule],
    env_file: bool,
    redact: bool,
    location_fn: impl Fn(usize) -> Location,
    findings: &mut Vec<Finding>,
) {
    for rule in rules {
        if rule.env_only && !env_file {
            continue;
        }
        if let Some(mat) = rule.regex.find(line) {
            let raw = mat.as_str();
            let (matched_text, line_content) = if redact {
                let token = if raw.len() > 8 {
                    format!("{}…[redacted]", &raw[..4])
                } else {
                    "[redacted]".into()
                };
                let redacted_line = format!(
                    "{}{}{}",
                    &line[..mat.start()],
                    "[redacted]",
                    &line[mat.end()..],
                );
                (token, redacted_line.trim_end().to_string())
            } else {
                (raw.to_string(), line.trim_end().to_string())
            };
            findings.push(Finding {
                rule_name: rule.name.clone(),
                severity: rule.severity.clone(),
                location: location_fn(line_no),
                matched_text,
                line_content,
                git_ignored: false,
            });
        }
    }
}

// ── File scanner ─────────────────────────────────────────────────────────────

pub fn scan_file(path: &Path, cfg: &ScanConfig, findings: &mut Vec<Finding>) -> anyhow::Result<()> {
    // Skip binary-looking files by extension
    if is_likely_binary(path) {
        return Ok(());
    }

    let env_file = is_env_file(path);

    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()), // skip non-UTF-8 / unreadable
    };

    let path_str = path.to_string_lossy().to_string();

    for (idx, line) in contents.lines().enumerate() {
        let line_no = idx + 1;
        let loc = |ln: usize| Location::File {
            path: path_str.clone(),
            line_no: ln,
        };
        scan_line(line, line_no, &cfg.rules, env_file, cfg.redact, loc, findings);
    }
    Ok(())
}

fn is_likely_binary(path: &Path) -> bool {
    let binary_exts = [
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp",
        "zip", "gz", "tar", "bz2", "xz", "7z", "rar",
        "exe", "dll", "so", "dylib", "bin", "obj", "o", "a",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "mp3", "mp4", "avi", "mov", "wav", "flac",
        "ttf", "otf", "woff", "woff2",
        "pyc", "pyd", "class",
        "lock",
    ];
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        return binary_exts.contains(&ext.to_lowercase().as_str());
    }
    false
}

// ── Directory scanner ─────────────────────────────────────────────────────────

pub fn scan_directory(root: &Path, cfg: &ScanConfig, findings: &mut Vec<Finding>) -> anyhow::Result<()> {
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Always skip ignored paths and the .git dir (history is handled
            // separately via scan_git_history).
            let p = e.path();
            if p.file_name().map_or(false, |n| n == ".git") {
                return false;
            }
            !cfg.is_ignored(p)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            scan_file(entry.path(), cfg, findings)?;
        }
    }
    Ok(())
}

// ── Git history scanner ───────────────────────────────────────────────────────

pub fn scan_git_history(repo_path: &Path, cfg: &ScanConfig, findings: &mut Vec<Finding>) -> anyhow::Result<()> {
    use git2::{DiffFormat, Repository, Sort};

    let repo = Repository::open(repo_path)
        .with_context(|| format!("failed to open git repo at {}", repo_path.display()))?;

    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME)?;

    // Push all local references so we scan every branch.
    revwalk.push_glob("refs/heads/*")?;
    // Fall back to HEAD if no branches found.
    let _ = revwalk.push_head();

    for oid_result in revwalk {
        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let diff = if commit.parent_count() == 0 {
            // Initial commit – diff against empty tree.
            repo.diff_tree_to_tree(None, Some(&tree), None)
        } else {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
        };

        let diff = match diff {
            Ok(d) => d,
            Err(_) => continue,
        };

        let short_hash = &oid.to_string()[..8];
        let commit_msg = commit.summary().unwrap_or("").to_string();

        // We collect findings in a local vec to pass to the closure.
        let mut local: Vec<Finding> = Vec::new();

        let rules = &cfg.rules;
        let ignored = &cfg.ignore;

        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
            // Only look at added lines (+).
            if line.origin() != '+' {
                return true;
            }
            let content = match std::str::from_utf8(line.content()) {
                Ok(s) => s,
                Err(_) => return true,
            };
            // Resolve the file path from the diff delta.
            let file_path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            // Check ignore list.
            let path_obj = std::path::PathBuf::from(&file_path);
            for ign in ignored {
                if path_obj.starts_with(ign) {
                    return true;
                }
                for comp in path_obj.components() {
                    if std::path::Path::new(comp.as_os_str()) == ign.as_path() {
                        return true;
                    }
                }
            }

            let env_file = is_env_file(Path::new(&file_path));
            let hash = oid.to_string();
            let sh = short_hash.to_string();
            let msg = commit_msg.clone();
            let fp = file_path.clone();

            scan_line(
                content.trim_end_matches('\n'),
                0, // line numbers not available in diff context
                rules,
                env_file,
                cfg.redact,
                move |_| Location::Commit {
                    hash: hash.clone(),
                    short_hash: sh.clone(),
                    file: fp.clone(),
                    commit_message: msg.clone(),
                },
                &mut local,
            );

            true
        })
        .ok(); // ignore diff errors

        findings.extend(local);
    }

    Ok(())
}

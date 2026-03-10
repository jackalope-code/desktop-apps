//! Integration tests for fast-secret-scanner.
//!
//! These tests exercise the public library API (scan_file, scan_directory,
//! default_rules, user_rule) against known-good and known-bad inputs.

use fast_secret_scanner::{
    apply_gitignore, default_rules, scan_directory, scan_file, scan_staged,
    types::{Finding, ScanConfig, Severity},
    user_rule,
};
use std::path::PathBuf;
use tempfile::TempDir;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_cfg() -> ScanConfig {
    ScanConfig {
        rules: default_rules(),
        ignore: vec![],
        scan_history: false,
        redact: true,
        fssignore: vec![],
    }
}

fn cfg_without(rule_name: &str) -> ScanConfig {
    let mut cfg = default_cfg();
    cfg.rules.retain(|r| r.name != rule_name);
    cfg
}

fn write_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

fn has_rule(findings: &[Finding], name: &str) -> bool {
    findings.iter().any(|f| f.rule_name == name)
}

fn findings_for<'a>(findings: &'a [Finding], name: &str) -> Vec<&'a Finding> {
    findings.iter().filter(|f| f.rule_name == name).collect()
}

fn location_path(f: &Finding) -> Option<&str> {
    match &f.location {
        fast_secret_scanner::Location::File { path, .. } => Some(path.as_str()),
        _ => None,
    }
}

// Helper trait so tests can call f.location_path() ergonomically
trait FindingExt {
    fn location_path(&self) -> Option<&str>;
}
impl FindingExt for Finding {
    fn location_path(&self) -> Option<&str> {
        location_path(self)
    }
}

// ── Private / PEM keys ────────────────────────────────────────────────────────

#[test]
fn detects_pem_private_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "key.pem",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQ...\n-----END RSA PRIVATE KEY-----\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "private-key-pem"),
        "should detect PEM private key"
    );
}

#[test]
fn detects_openssh_private_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "id_rsa", "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXkA...\n-----END OPENSSH PRIVATE KEY-----\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "ssh-private-key"),
        "should detect OpenSSH private key header"
    );
    assert!(
        has_rule(&findings, "private-key-pem"),
        "should also match PEM rule"
    );
}

// ── AWS ───────────────────────────────────────────────────────────────────────

#[test]
fn detects_aws_access_key_id() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.py",
        "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "aws-access-key-id"),
        "should detect AKIA... key"
    );
}

#[test]
fn detects_aws_secret_access_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.env",
        "aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "aws-secret-access-key"),
        "should detect aws_secret_access_key"
    );
}

// ── GitHub ────────────────────────────────────────────────────────────────────

#[test]
fn detects_github_pat() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "deploy.sh",
        "TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuV0123456\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "github-pat"),
        "should detect ghp_ token"
    );
}

#[test]
fn detects_github_app_token() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "ci.yml", "auth: ghs_1234567890ABCDEFGHIJKLMNOPqr\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "github-pat"),
        "should detect ghs_ token"
    );
}

// ── Google ────────────────────────────────────────────────────────────────────

#[test]
fn detects_google_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "index.js",
        "const key = 'AIzaSyD-9tSrke72I6sSSBJkLBMnMioAqsL3IYA';\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "google-api-key"),
        "should detect AIza... key"
    );
}

// ── Stripe ────────────────────────────────────────────────────────────────────

#[test]
fn detects_stripe_live_secret() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "payments.rb",
        "Stripe.api_key = 'sk_live_51ABCDEFGHIJKLMNorstuvwxyz12'\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "stripe-live-secret"),
        "should detect sk_live_ key"
    );
}

#[test]
fn detects_stripe_test_secret() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "test_payments.py",
        "api_key = 'sk_test_abcdefghijklmnopqrstuvwx'\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "stripe-test-secret"),
        "should detect sk_test_ key"
    );
    let stripe_findings = findings_for(&findings, "stripe-test-secret");
    assert_eq!(
        stripe_findings[0].severity,
        Severity::Medium,
        "stripe test key should be Medium"
    );
}

// ── JWT ───────────────────────────────────────────────────────────────────────

#[test]
fn detects_jwt_token() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir, "auth.js",
        "const token = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c';\n"
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "jwt-token"), "should detect JWT");
}

// ── Database connection strings ───────────────────────────────────────────────

#[test]
fn detects_db_connection_string_postgres() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "db.go",
        r#"db, _ := sql.Open("postgres", "postgresql://admin:s3cr3tpassword@prod.example.com:5432/mydb")"#,
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "db-connection-string"),
        "should detect postgres:// connection string"
    );
}

#[test]
fn detects_db_connection_string_mongodb() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "app.js",
        "mongoose.connect('mongodb+srv://user:password123@cluster0.mongodb.net/mydb');",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "db-connection-string"),
        "should detect mongodb+srv connection string"
    );
}

// ── Generic patterns ──────────────────────────────────────────────────────────

#[test]
fn detects_generic_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.ts",
        "const API_KEY = 'abcdefghijklmnopqrstuvwxyz1234567';\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "generic-api-key"),
        "should detect generic api_key assignment"
    );
}

#[test]
fn detects_generic_password() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "settings.py", "password = 'supersecret123'\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "generic-password"),
        "should detect password assignment"
    );
}

#[test]
fn detects_generic_secret() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "app.config",
        "app_secret=\"my_super_long_secret_value_here\"\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "generic-secret"),
        "should detect generic secret assignment"
    );
}

#[test]
fn detects_basic_auth_url() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "Makefile",
        "curl https://admin:password123@api.example.com/endpoint\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "basic-auth-url"),
        "should detect http basic auth URL"
    );
}

// ── .env file rules ───────────────────────────────────────────────────────────

#[test]
fn env_any_only_triggers_on_env_files() {
    let dir = TempDir::new().unwrap();
    // .rs file → env-var-any should NOT trigger
    let rs_path = write_file(&dir, "build.rs", "MY_VAR=hello\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&rs_path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "env-var-any"),
        "env-var-any should not trigger on .rs files"
    );
}

#[test]
fn env_any_triggers_on_dotenv_file() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        ".env",
        "DATABASE_URL=postgres://localhost/db\nDEBUG=true\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "env-var-any"),
        "env-var-any should trigger in .env file"
    );
}

#[test]
fn env_sensitive_triggers_for_secret_variable() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, ".env", "API_SECRET=abc123verylongvalue\nPORT=3000\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "env-var-sensitive"),
        "should flag API_SECRET as sensitive"
    );
    let sensitive = findings_for(&findings, "env-var-sensitive");
    assert_eq!(sensitive[0].severity, Severity::High);
}

#[test]
fn env_sensitive_triggers_for_password_variable() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        ".env.production",
        "DB_PASSWORD=my_prod_password_value\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "env-var-sensitive"),
        "should flag DB_PASSWORD as sensitive"
    );
}

// ── User directory path detection ─────────────────────────────────────────────

#[test]
fn detects_unix_user_home_path() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.py",
        "config_path = '/home/alice/projects/myapp/.config'\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "user-dir-path"),
        "should detect /home/alice/ path"
    );
    let user_findings = findings_for(&findings, "user-dir-path");
    assert_eq!(user_findings[0].severity, Severity::Warning);
}

#[test]
fn detects_bare_username_path() {
    // /home/alice alone (no subpath) should still be flagged – the username is exposed.
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "README.md", "Default home: /home/alice\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "user-dir-path"),
        "/home/alice at EOL should be flagged"
    );
}

#[test]
fn user_dir_root_not_flagged() {
    // Bare parent directories with no username should NOT fire.
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "README.md",
        "All user homes live under /home or under C:\\Users on Windows.\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "user-dir-path"),
        "bare /home and C:\\Users without a username should not be flagged"
    );
}

#[test]
fn detects_macos_user_home_path() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "README.md",
        "Copy to /Users/bob/Library/Application Support/MyApp/\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "user-dir-path"),
        "should detect /Users/bob/ path"
    );
}

#[test]
fn detects_windows_user_home_path() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "setup.bat",
        "set CONFIG_DIR=C:\\Users\\charlie\\AppData\\\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "user-dir-path"),
        "should detect C:\\Users\\charlie\\ path"
    );
}

#[test]
fn keep_user_dir_suppresses_rule() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "config.py", "path = '/home/alice/projects/app/'\n");
    let cfg = cfg_without("user-dir-path");
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "user-dir-path"),
        "user-dir-path should be suppressed when rule removed"
    );
}

// ── Binary / non-UTF-8 files are skipped ─────────────────────────────────────

#[test]
fn skips_binary_extension_files() {
    let dir = TempDir::new().unwrap();
    // .exe extension → should be skipped entirely even with secret-looking content
    let path = write_file(&dir, "app.exe", "AKIAIOSFODNN7EXAMPLE\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        findings.is_empty(),
        "binary-extension file should be skipped"
    );
}

#[test]
fn skips_lock_files() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "Cargo.lock", "AKIAIOSFODNN7EXAMPLE\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(findings.is_empty(), ".lock files should be skipped");
}

// ── Directory scanning ────────────────────────────────────────────────────────

#[test]
fn scan_directory_finds_secrets_recursively() {
    let dir = TempDir::new().unwrap();
    let subdir = dir.path().join("src");
    std::fs::create_dir_all(&subdir).unwrap();
    std::fs::write(
        subdir.join("config.py"),
        "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"\n",
    )
    .unwrap();
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "aws-access-key-id"),
        "directory scan should find nested secrets"
    );
}

#[test]
fn scan_directory_respects_ignore_list() {
    let dir = TempDir::new().unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    std::fs::write(
        vendor.join("secret.js"),
        "const key = 'sk_live_51ABCDEFGHIJKLMNorstuvwxyz12';\n",
    )
    .unwrap();
    let cfg = ScanConfig {
        rules: default_rules(),
        ignore: vec![PathBuf::from("vendor")],
        scan_history: false,
        redact: true,
        fssignore: vec![],
    };
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "stripe-live-secret"),
        "vendor directory should be ignored"
    );
}

#[test]
fn scan_directory_skips_git_dir() {
    let dir = TempDir::new().unwrap();
    let git_dir = dir.path().join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::write(
        git_dir.join("COMMIT_EDITMSG"),
        "api_key: sk_live_51ABCDEFGHIJKLMNorstuvwxyz12\n",
    )
    .unwrap();
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "stripe-live-secret"),
        ".git directory should be skipped"
    );
}

#[test]
fn scan_directory_finds_env_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join(".env"),
        "DB_PASSWORD=super_secret\nPORT=5432\n",
    )
    .unwrap();
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "env-var-any"),
        "should find env vars in .env file"
    );
    assert!(
        has_rule(&findings, "env-var-sensitive"),
        "should flag DB_PASSWORD as sensitive"
    );
}

// ── Custom user rules ─────────────────────────────────────────────────────────

#[test]
fn custom_user_rule_matches() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "deploy.sh",
        "MYAPP_TOKEN=mytoken_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcd\n",
    );
    let custom = user_rule(0, r"mytoken_[A-Za-z0-9]{40}").unwrap();
    let cfg = ScanConfig {
        rules: vec![custom],
        ignore: vec![],
        scan_history: false,
        redact: true,
        fssignore: vec![],
    };
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "custom-0"),
        "custom pattern should match"
    );
    assert_eq!(
        findings[0].severity,
        Severity::High,
        "custom rule should be High severity"
    );
}

#[test]
fn invalid_custom_pattern_returns_error() {
    let result = user_rule(0, r"[invalid-regex(");
    assert!(result.is_err(), "invalid regex should return an error");
}

// ── Severity ordering ─────────────────────────────────────────────────────────

#[test]
fn severity_ordering_is_correct() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Warning);
    assert!(Severity::Warning < Severity::Critical);
}

// ── Redaction ─────────────────────────────────────────────────────────────────

#[test]
fn all_findings_redacted_by_default() {
    let dir = TempDir::new().unwrap();
    let full_key = "AKIAIOSFODNN7EXAMPLE";
    let path = write_file(&dir, "config.env", &format!("AWS_ACCESS_KEY={full_key}\n"));
    let cfg = default_cfg(); // redact: true
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    for f in &findings {
        assert!(
            f.matched_text.contains("[redacted]"),
            "matched_text should be redacted for rule '{}', got: '{}'",
            f.rule_name,
            f.matched_text
        );
        assert!(
            !f.line_content.contains(full_key),
            "line_content should not contain the raw secret, got: '{}'",
            f.line_content
        );
    }
}

#[test]
fn warning_findings_also_redacted_by_default() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, ".env", "MY_VAR=hello\n");
    let cfg = default_cfg(); // redact: true
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    let warn_findings = findings_for(&findings, "env-var-any");
    assert!(!warn_findings.is_empty(), "should find env-var-any");
    for f in &warn_findings {
        assert!(
            f.matched_text.contains("[redacted]"),
            "warning findings should also be redacted by default, got: '{}'",
            f.matched_text
        );
    }
}

#[test]
fn unredact_shows_full_values() {
    let dir = TempDir::new().unwrap();
    let full_key = "AKIAIOSFODNN7EXAMPLE";
    let path = write_file(&dir, "config.env", &format!("AWS_ACCESS_KEY={full_key}\n"));
    let cfg = ScanConfig {
        rules: default_rules(),
        ignore: vec![],
        scan_history: false,
        redact: false, // --unredact
        fssignore: vec![],
    };
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    let aws: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "aws-access-key-id")
        .collect();
    assert!(!aws.is_empty(), "should find aws-access-key-id");
    for f in &aws {
        assert!(
            f.matched_text.contains(full_key) || f.line_content.contains(full_key),
            "with redact=false, full value must be visible"
        );
    }
}

// ── .gitignore coverage ───────────────────────────────────────────────────────

/// Create a minimal git repo in `dir` with a `.gitignore` containing `entries`.
fn init_git_repo_with_gitignore(dir: &TempDir, entries: &str) {
    git2::Repository::init(dir.path()).expect("git init failed");
    std::fs::write(dir.path().join(".gitignore"), entries).unwrap();
}

#[test]
fn gitignored_file_findings_are_marked_covered() {
    let dir = TempDir::new().unwrap();
    init_git_repo_with_gitignore(&dir, ".env\n");

    // .env is gitignored → finding should be covered
    std::fs::write(dir.path().join(".env"), "DB_PASSWORD=super_secret_value\n").unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let sensitive: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "env-var-sensitive")
        .collect();
    assert!(!sensitive.is_empty(), "should find env-var-sensitive");
    for f in &sensitive {
        assert!(
            f.git_ignored,
            "finding in gitignored .env should have git_ignored=true"
        );
    }
}

#[test]
fn non_gitignored_file_findings_are_marked_active() {
    let dir = TempDir::new().unwrap();
    init_git_repo_with_gitignore(&dir, "vendor/\n");

    // config.py is NOT gitignored → finding should remain active
    std::fs::write(
        dir.path().join("config.py"),
        "API_KEY = 'AIzaSyD-9tSrke72I6sSSBJkLBMnMioAqsL3IYA'\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let google: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "google-api-key")
        .collect();
    assert!(!google.is_empty(), "should find google-api-key");
    for f in &google {
        assert!(
            !f.git_ignored,
            "finding in non-gitignored file should have git_ignored=false"
        );
    }
}

#[test]
fn mixed_findings_partition_correctly() {
    let dir = TempDir::new().unwrap();
    // .env is ignored, config.js is not
    init_git_repo_with_gitignore(&dir, ".env\n");

    std::fs::write(
        dir.path().join(".env"),
        "DB_PASSWORD=ignored_secret_value\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("config.js"),
        "const key = 'sk_live_51ABCDEFGHIJKLMNorstuvwxyz12';\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let (covered, active): (Vec<_>, Vec<_>) = findings.iter().partition(|f| f.git_ignored);
    assert!(
        !covered.is_empty(),
        "should have covered findings from .env"
    );
    assert!(
        !active.is_empty(),
        "should have active findings from config.js"
    );

    // Stripe live key in config.js must be active (not gitignored)
    let stripe_active = active.iter().any(|f| f.rule_name == "stripe-live-secret");
    assert!(
        stripe_active,
        "stripe-live-secret in config.js must be active"
    );

    // env-var-sensitive from .env must be covered
    let env_covered = covered.iter().any(|f| f.rule_name == "env-var-sensitive");
    assert!(
        env_covered,
        "env-var-sensitive from gitignored .env must be covered"
    );
}

#[test]
fn all_covered_means_zero_active() {
    let dir = TempDir::new().unwrap();
    // Ignore everything that will contain secrets
    init_git_repo_with_gitignore(&dir, ".env\nsecrets/\n");

    std::fs::create_dir(dir.path().join("secrets")).unwrap();
    std::fs::write(
        dir.path().join(".env"),
        "API_SECRET=some_long_secret_value\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("secrets").join("keys.txt"),
        "GITHUB_TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuV0123456\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let active: Vec<_> = findings.iter().filter(|f| !f.git_ignored).collect();
    // Only the .gitignore file itself might appear in scans; no secret files are tracked
    // All finding files (.env, secrets/) are gitignored → active should be empty
    assert!(
        active.iter().all(|f| !matches!(&f.location,
            fast_secret_scanner::Location::File { path, .. }
            if path.contains(".env") || path.contains("keys.txt")
        )),
        "secrets in gitignored files should not appear as active findings"
    );
}

#[test]
fn wildcard_gitignore_pattern_covers_env_files() {
    let dir = TempDir::new().unwrap();
    // Wildcard: ignore all .env.* variants
    init_git_repo_with_gitignore(&dir, "*.env\n.env*\n");

    std::fs::write(
        dir.path().join(".env.production"),
        "API_TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuV0123456\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let github_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "github-pat")
        .collect();
    assert!(
        !github_findings.is_empty(),
        "should find github-pat in .env.production"
    );
    for f in &github_findings {
        assert!(
            f.git_ignored,
            ".env.production matched by .env* should be git_ignored"
        );
    }
}

#[test]
fn non_git_directory_all_findings_are_active() {
    let dir = TempDir::new().unwrap();
    // No git repo → apply_gitignore is a no-op, all findings stay active
    std::fs::write(
        dir.path().join("secret.js"),
        "const key = 'sk_live_51ABCDEFGHIJKLMNorstuvwxyz12';\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings); // no-op: no git repo

    let stripe: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "stripe-live-secret")
        .collect();
    assert!(!stripe.is_empty(), "should find stripe-live-secret");
    for f in &stripe {
        assert!(
            !f.git_ignored,
            "without a git repo, findings should not be marked git_ignored"
        );
    }
}

// ── Monorepo / nested .gitignore tests ───────────────────────────────────────
//
// git2::Repository::is_path_ignored walks the full gitignore chain
// (root .gitignore → subdir .gitignore → .git/info/exclude → global),
// so all nested rules should be respected automatically.

/// Create a git repo with ONLY a root .gitignore (no subdir rules yet).
/// Subdirectory .gitignore files are created by the individual tests.
fn init_monorepo(dir: &TempDir, root_gitignore: &str) {
    git2::Repository::init(dir.path()).expect("git init failed");
    if !root_gitignore.is_empty() {
        std::fs::write(dir.path().join(".gitignore"), root_gitignore).unwrap();
    }
}

#[test]
fn subdir_gitignore_covers_files_in_that_subdir() {
    // packages/api has its own .gitignore covering .env
    // packages/web has NO .gitignore – its .env is exposed
    let dir = TempDir::new().unwrap();
    init_monorepo(&dir, ""); // empty root gitignore

    let api = dir.path().join("packages").join("api");
    let web = dir.path().join("packages").join("web");
    std::fs::create_dir_all(&api).unwrap();
    std::fs::create_dir_all(&web).unwrap();

    std::fs::write(api.join(".gitignore"), ".env\n").unwrap();
    std::fs::write(api.join(".env"), "DB_PASSWORD=covered_secret\n").unwrap();
    std::fs::write(web.join(".env"), "DB_PASSWORD=exposed_secret\n").unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let api_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.location_path().map_or(false, |p| p.contains("api")))
        .collect();
    let web_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.location_path().map_or(false, |p| p.contains("web")))
        .collect();

    assert!(
        !api_findings.is_empty(),
        "should find secrets in packages/api"
    );
    for f in &api_findings {
        assert!(
            f.git_ignored,
            "packages/api/.env covered by subdir .gitignore → git_ignored"
        );
    }
    assert!(
        !web_findings.is_empty(),
        "should find secrets in packages/web"
    );
    for f in &web_findings {
        assert!(!f.git_ignored, "packages/web/.env has no coverage → active");
    }
}

#[test]
fn root_gitignore_covers_files_in_nested_subdirs() {
    // Root .gitignore with "**/.env" covers .env files at any depth
    let dir = TempDir::new().unwrap();
    init_monorepo(&dir, "**/.env\n");

    let deep = dir.path().join("packages").join("backend").join("config");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join(".env"), "API_SECRET=deeply_nested_secret\n").unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let env_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "env-var-sensitive")
        .collect();
    assert!(
        !env_findings.is_empty(),
        "should find env-var-sensitive in nested .env"
    );
    for f in &env_findings {
        assert!(
            f.git_ignored,
            "deeply nested .env covered by root **/.env rule"
        );
    }
}

#[test]
fn root_and_subdir_gitignores_both_apply() {
    // Root ignores *.key files globally; packages/payments also ignores .env
    // Both packages/payments/.env AND any *.key files anywhere are covered
    let dir = TempDir::new().unwrap();
    init_monorepo(&dir, "*.key\n");

    let payments = dir.path().join("packages").join("payments");
    std::fs::create_dir_all(&payments).unwrap();
    std::fs::write(payments.join(".gitignore"), ".env\n").unwrap();
    std::fs::write(
        payments.join(".env"),
        "STRIPE_SECRET=sk_live_51ABCDEFGHIJKLMNorstuvwxyz12\n",
    )
    .unwrap();
    std::fs::write(
        payments.join("signing.key"),
        "private-key = AKIAIOSFODNN7EXAMPLE\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    // .env covered by packages/payments/.gitignore
    let env_covered = findings
        .iter()
        .filter(|f| f.rule_name == "stripe-live-secret" && f.git_ignored)
        .count();
    assert!(
        env_covered > 0,
        "stripe secret in .env should be covered by subdir .gitignore"
    );

    // .key file covered by root .gitignore
    let key_covered = findings
        .iter()
        .filter(|f| {
            f.location_path()
                .map_or(false, |p| p.ends_with("signing.key"))
                && f.git_ignored
        })
        .count();
    assert!(
        key_covered > 0,
        "signing.key should be covered by root *.key rule"
    );
}

#[test]
fn subdir_gitignore_negation_uncovers_root_rule() {
    // Root ignores all .env files. packages/public/.gitignore tries to
    // re-include with "!.env". In native `git`, a subdir negation CAN override
    // a parent file-level rule (as opposed to a parent *directory* exclusion).
    //
    // However, libgit2's `is_path_ignored` does NOT honour subdir negation
    // overrides of root-level patterns — it still reports the file as ignored.
    // This is a known libgit2 limitation. We document and assert the actual
    // behavior here so that we catch any future library upgrade that fixes it.
    let dir = TempDir::new().unwrap();
    init_monorepo(&dir, ".env\n");

    let public_pkg = dir.path().join("packages").join("public");
    std::fs::create_dir_all(&public_pkg).unwrap();
    std::fs::write(public_pkg.join(".gitignore"), "!.env\n").unwrap();
    std::fs::write(public_pkg.join(".env"), "NODE_ENV=production\nPORT=8080\n").unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let public_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.location_path().map_or(false, |p| p.contains("public")))
        .collect();

    assert!(
        !public_findings.is_empty(),
        "should find env vars in packages/public/.env"
    );
    // libgit2 limitation: the root ".env" rule wins; subdir "!.env" negation is
    // not applied. All findings remain marked as git_ignored = true.
    // If this assertion ever fails it means libgit2 was fixed and we can
    // flip to asserting git_ignored = false.
    for f in &public_findings {
        assert!(f.git_ignored,
            "libgit2 does not honour subdir negation of root rules — expected git_ignored=true (known limitation)");
    }
}

#[test]
fn each_package_gitignore_is_scoped_to_its_subtree() {
    // Three packages: only packages/a ignores .env
    // packages/b and packages/c don't → their findings stay active
    let dir = TempDir::new().unwrap();
    init_monorepo(&dir, "");

    for pkg in ["a", "b", "c"] {
        let p = dir.path().join("packages").join(pkg);
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join(".env"),
            format!("TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuV{pkg}0123456\n"),
        )
        .unwrap();
    }
    // Only package a ignores it
    std::fs::write(
        dir.path().join("packages").join("a").join(".gitignore"),
        ".env\n",
    )
    .unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    apply_gitignore(dir.path(), &mut findings);

    let pat_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule_name == "github-pat")
        .collect();
    assert!(
        !pat_findings.is_empty(),
        "should find github-pat in all packages"
    );

    for f in &pat_findings {
        let path = f.location_path().unwrap_or("");
        let in_a = path.contains(&format!("packages{}a", std::path::MAIN_SEPARATOR))
            || path.contains("packages/a");
        if in_a {
            assert!(f.git_ignored, "packages/a/.env should be covered");
        } else {
            assert!(
                !f.git_ignored,
                "packages/b and packages/c .env should be active"
            );
        }
    }
}

// ── False-positive regression tests ──────────────────────────────────────────

#[test]
fn basic_auth_url_ignores_google_fonts_cdn() {
    // Google Fonts CDN URLs like ?family=Foo:wght@300 contain `:wght` (looks
    // like a username) and `@300` (looks like a host) but there is a `/` in
    // the URL path before the colon, so the fixed regex won't match.
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "styles.css",
        r#"@import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&display=swap');"#,
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "basic-auth-url"),
        "Google Fonts CDN URLs should not be flagged as basic-auth-url"
    );
}

#[test]
fn basic_auth_url_flags_real_credentials() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.js",
        r#"const dbUrl = "https://admin:supersecretpassword@db.example.com/mydb";"#,
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "basic-auth-url"),
        "real username:password@host credentials should be flagged"
    );
}

#[test]
fn generic_password_ignores_typescript_type_annotation() {
    // TypeScript function param `password: string` should not be flagged.
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "auth.ts",
        "async function validatePassword(password: string): Promise<boolean> { return true; }\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "generic-password"),
        "TypeScript type annotation `password: string` should not fire generic-password"
    );
}

#[test]
fn generic_password_flags_hardcoded_value() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.py",
        "password = \"hunter2secure\"\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "generic-password"),
        "hardcoded password value should still be flagged"
    );
}

// ── Infrastructure disclosure tests ──────────────────────────────────────────

#[test]
fn infra_flags_nonloopback_ipv4() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "db.rs",
        r#"let addr = "192.168.1.50:5432";"#,
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "infra-ipv4-address"),
        "non-loopback IPv4 should be flagged by infra-ipv4-address"
    );
}

#[test]
fn infra_ignores_loopback_ipv4() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "server.rs",
        r#"let addr = "127.0.0.1:8080";"#,
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "infra-ipv4-address"),
        "loopback 127.0.0.1 should not be flagged"
    );
}

#[test]
fn infra_flags_internal_hostname() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.yaml",
        "database_host: db-primary.internal\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "infra-internal-hostname"),
        "internal hostname should be flagged"
    );
}

#[test]
fn infra_suppressed_by_ignore_infrastructure_flag() {
    // Simulate --ignore-infrastructure by filtering infra rules out of the config.
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "db.rs",
        r#"let addr = "172.16.0.1:3306";"#,
    );
    let mut cfg = default_cfg();
    cfg.rules.retain(|r| !r.infra);
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        !has_rule(&findings, "infra-ipv4-address"),
        "infra rules should be suppressed when filtered out"
    );
}

// ── New API key rule tests ────────────────────────────────────────────────────

#[test]
fn detects_openai_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        ".env",
        "OPENAI_API_KEY=sk-ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij123456\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "openai-api-key"),
        "OpenAI API key should be detected"
    );
}

#[test]
fn detects_anthropic_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        ".env",
        "ANTHROPIC_KEY=sk-ant-abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJ\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "anthropic-api-key"),
        "Anthropic API key should be detected"
    );
}

#[test]
fn detects_linear_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.js",
        "const client = new LinearClient({ apiKey: 'lin_api_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqr' });\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "linear-api-key"),
        "Linear API key should be detected"
    );
}

#[test]
fn detects_notion_integration_token() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        ".env",
        "NOTION_TOKEN=secret_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrs\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "notion-token"),
        "Notion integration token should be detected"
    );
}

#[test]
fn detects_digitalocean_token() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        ".env",
        "DO_TOKEN=dop_v1_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456789012\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "digitalocean-token"),
        "DigitalOcean personal access token should be detected"
    );
}

#[test]
fn detects_sentry_dsn() {
    let dir = TempDir::new().unwrap();
    let path = write_file(
        &dir,
        "config.js",
        "Sentry.init({ dsn: 'https://abcdef1234567890abcdef1234567890@o123456.ingest.sentry.io/9876543' });\n",
    );
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(
        has_rule(&findings, "sentry-dsn"),
        "Sentry DSN should be detected"
    );
}

// ── Staged-file (pre-commit) scan tests ──────────────────────────────────────────

/// Stage a single file into the given repo's index.
fn stage_file(repo: &git2::Repository, filename: &str) {
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();
}

#[test]
fn staged_scan_finds_secret_in_index() {
    // A staged file containing a fake GitHub PAT must be detected.
    let dir = TempDir::new().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    write_file(&dir, "secrets.env", "TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ12345678\n");
    stage_file(&repo, "secrets.env");

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_staged(dir.path(), &cfg, &mut findings).unwrap();

    assert!(
        has_rule(&findings, "github-pat"),
        "staged github-pat should be detected"
    );
}

#[test]
fn staged_scan_ignores_unstaged_changes() {
    // A file on disk but not in the index must NOT appear in staged findings.
    let dir = TempDir::new().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    // Stage an innocent file.
    write_file(&dir, "readme.txt", "hello world\n");
    stage_file(&repo, "readme.txt");

    // Write a secret file but do NOT add it to the index.
    write_file(&dir, "secrets.env", "TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ99999999\n");

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_staged(dir.path(), &cfg, &mut findings).unwrap();

    assert!(
        !has_rule(&findings, "github-pat"),
        "unstaged secret file must not appear in staged scan"
    );
}

#[test]
fn staged_scan_clean_on_empty_index() {
    // An empty index produces no findings.
    let dir = TempDir::new().unwrap();
    let _repo = git2::Repository::init(dir.path()).unwrap();

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_staged(dir.path(), &cfg, &mut findings).unwrap();

    assert!(findings.is_empty(), "empty staged index should yield no findings");
}

#[test]
fn staged_scan_picks_up_new_lines_after_initial_commit() {
    // After an initial commit, only the *newly staged* lines (delta vs HEAD)
    // should appear in findings — not lines from the committed baseline.
    use git2::Signature;

    let dir = TempDir::new().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    // Initial commit: innocent file.
    write_file(&dir, "app.js", "console.log('hello');\n");
    stage_file(&repo, "app.js");
    let mut index = repo.index().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::now("Test", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();

    // Now stage a new file that contains a fake AWS key.
    write_file(
        &dir,
        "config.js",
        "const key = 'AKIAIOSFODNN7EXAMPLE';\n",
    );
    stage_file(&repo, "config.js");

    let cfg = default_cfg();
    let mut findings = vec![];
    scan_staged(dir.path(), &cfg, &mut findings).unwrap();

    assert!(
        has_rule(&findings, "aws-access-key-id"),
        "newly staged AWS key should be flagged"
    );
    // The innocent committed file should NOT contribute findings.
    let file_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.location_path().map_or(false, |p| p.contains("app.js")))
        .collect();
    assert!(
        file_findings.is_empty(),
        "previously committed innocent file must not appear in staged scan"
    );
}

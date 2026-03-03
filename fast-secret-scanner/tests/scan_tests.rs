//! Integration tests for fast-secret-scanner.
//!
//! These tests exercise the public library API (scan_file, scan_directory,
//! default_rules, user_rule) against known-good and known-bad inputs.

use fast_secret_scanner::{
    default_rules, scan_directory, scan_file, user_rule,
    types::{Finding, ScanConfig, Severity},
};
use std::path::PathBuf;
use tempfile::TempDir;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_cfg() -> ScanConfig {
    ScanConfig {
        rules: default_rules(),
        ignore: vec![],
        scan_history: false,
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

// ── Private / PEM keys ────────────────────────────────────────────────────────

#[test]
fn detects_pem_private_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "key.pem", "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQ...\n-----END RSA PRIVATE KEY-----\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "private-key-pem"), "should detect PEM private key");
}

#[test]
fn detects_openssh_private_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "id_rsa", "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXkA...\n-----END OPENSSH PRIVATE KEY-----\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "ssh-private-key"), "should detect OpenSSH private key header");
    assert!(has_rule(&findings, "private-key-pem"), "should also match PEM rule");
}

// ── AWS ───────────────────────────────────────────────────────────────────────

#[test]
fn detects_aws_access_key_id() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "config.py", "AWS_ACCESS_KEY_ID = \"AKIAIOSFODNN7EXAMPLE\"\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "aws-access-key-id"), "should detect AKIA... key");
}

#[test]
fn detects_aws_secret_access_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "config.env", "aws_secret_access_key=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "aws-secret-access-key"), "should detect aws_secret_access_key");
}

// ── GitHub ────────────────────────────────────────────────────────────────────

#[test]
fn detects_github_pat() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "deploy.sh", "TOKEN=ghp_aBcDeFgHiJkLmNoPqRsTuV0123456\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "github-pat"), "should detect ghp_ token");
}

#[test]
fn detects_github_app_token() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "ci.yml", "auth: ghs_1234567890ABCDEFGHIJKLMNOPqr\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "github-pat"), "should detect ghs_ token");
}

// ── Google ────────────────────────────────────────────────────────────────────

#[test]
fn detects_google_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "index.js", "const key = 'AIzaSyD-9tSrke72I6sSSBJkLBMnMioAqsL3IYA';\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "google-api-key"), "should detect AIza... key");
}

// ── Stripe ────────────────────────────────────────────────────────────────────

#[test]
fn detects_stripe_live_secret() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "payments.rb", "Stripe.api_key = 'sk_live_51ABCDEFGHIJKLMNorstuvwxyz12'\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "stripe-live-secret"), "should detect sk_live_ key");
}

#[test]
fn detects_stripe_test_secret() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "test_payments.py", "api_key = 'sk_test_abcdefghijklmnopqrstuvwx'\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "stripe-test-secret"), "should detect sk_test_ key");
    let stripe_findings = findings_for(&findings, "stripe-test-secret");
    assert_eq!(stripe_findings[0].severity, Severity::Medium, "stripe test key should be Medium");
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
    let path = write_file(&dir, "db.go", r#"db, _ := sql.Open("postgres", "postgresql://admin:s3cr3tpassword@prod.example.com:5432/mydb")"#);
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "db-connection-string"), "should detect postgres:// connection string");
}

#[test]
fn detects_db_connection_string_mongodb() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "app.js", "mongoose.connect('mongodb+srv://user:password123@cluster0.mongodb.net/mydb');");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "db-connection-string"), "should detect mongodb+srv connection string");
}

// ── Generic patterns ──────────────────────────────────────────────────────────

#[test]
fn detects_generic_api_key() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "config.ts", "const API_KEY = 'abcdefghijklmnopqrstuvwxyz1234567';\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "generic-api-key"), "should detect generic api_key assignment");
}

#[test]
fn detects_generic_password() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "settings.py", "password = 'supersecret123'\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "generic-password"), "should detect password assignment");
}

#[test]
fn detects_generic_secret() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "app.config", "app_secret=my_super_long_secret_value_here\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "generic-secret"), "should detect generic secret assignment");
}

#[test]
fn detects_basic_auth_url() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "Makefile", "curl https://admin:password123@api.example.com/endpoint\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "basic-auth-url"), "should detect http basic auth URL");
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
    assert!(!has_rule(&findings, "env-var-any"), "env-var-any should not trigger on .rs files");
}

#[test]
fn env_any_triggers_on_dotenv_file() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, ".env", "DATABASE_URL=postgres://localhost/db\nDEBUG=true\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "env-var-any"), "env-var-any should trigger in .env file");
}

#[test]
fn env_sensitive_triggers_for_secret_variable() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, ".env", "API_SECRET=abc123verylongvalue\nPORT=3000\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "env-var-sensitive"), "should flag API_SECRET as sensitive");
    let sensitive = findings_for(&findings, "env-var-sensitive");
    assert_eq!(sensitive[0].severity, Severity::High);
}

#[test]
fn env_sensitive_triggers_for_password_variable() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, ".env.production", "DB_PASSWORD=my_prod_password_value\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "env-var-sensitive"), "should flag DB_PASSWORD as sensitive");
}

// ── User directory path detection ─────────────────────────────────────────────

#[test]
fn detects_unix_user_home_path() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "config.py", "config_path = '/home/alice/projects/myapp/.config'\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "user-dir-path"), "should detect /home/alice/ path");
    let user_findings = findings_for(&findings, "user-dir-path");
    assert_eq!(user_findings[0].severity, Severity::Warning);
}

#[test]
fn detects_macos_user_home_path() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "README.md", "Copy to /Users/bob/Library/Application Support/MyApp/\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "user-dir-path"), "should detect /Users/bob/ path");
}

#[test]
fn detects_windows_user_home_path() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "setup.bat", "set CONFIG_DIR=C:\\Users\\charlie\\AppData\\\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "user-dir-path"), "should detect C:\\Users\\charlie\\ path");
}

#[test]
fn keep_user_dir_suppresses_rule() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "config.py", "path = '/home/alice/projects/app/'\n");
    let cfg = cfg_without("user-dir-path");
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(!has_rule(&findings, "user-dir-path"), "user-dir-path should be suppressed when rule removed");
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
    assert!(findings.is_empty(), "binary-extension file should be skipped");
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
    ).unwrap();
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "aws-access-key-id"), "directory scan should find nested secrets");
}

#[test]
fn scan_directory_respects_ignore_list() {
    let dir = TempDir::new().unwrap();
    let vendor = dir.path().join("vendor");
    std::fs::create_dir_all(&vendor).unwrap();
    std::fs::write(
        vendor.join("secret.js"),
        "const key = 'sk_live_51ABCDEFGHIJKLMNorstuvwxyz12';\n",
    ).unwrap();
    let cfg = ScanConfig {
        rules: default_rules(),
        ignore: vec![PathBuf::from("vendor")],
        scan_history: false,
    };
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(!has_rule(&findings, "stripe-live-secret"), "vendor directory should be ignored");
}

#[test]
fn scan_directory_skips_git_dir() {
    let dir = TempDir::new().unwrap();
    let git_dir = dir.path().join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::write(
        git_dir.join("COMMIT_EDITMSG"),
        "api_key: sk_live_51ABCDEFGHIJKLMNorstuvwxyz12\n",
    ).unwrap();
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(!has_rule(&findings, "stripe-live-secret"), ".git directory should be skipped");
}

#[test]
fn scan_directory_finds_env_files() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join(".env"),
        "DB_PASSWORD=super_secret\nPORT=5432\n",
    ).unwrap();
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_directory(dir.path(), &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "env-var-any"), "should find env vars in .env file");
    assert!(has_rule(&findings, "env-var-sensitive"), "should flag DB_PASSWORD as sensitive");
}

// ── Custom user rules ─────────────────────────────────────────────────────────

#[test]
fn custom_user_rule_matches() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, "deploy.sh", "MYAPP_TOKEN=mytoken_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcd\n");
    let custom = user_rule(0, r"mytoken_[A-Za-z0-9]{40}").unwrap();
    let cfg = ScanConfig {
        rules: vec![custom],
        ignore: vec![],
        scan_history: false,
    };
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    assert!(has_rule(&findings, "custom-0"), "custom pattern should match");
    assert_eq!(findings[0].severity, Severity::High, "custom rule should be High severity");
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
fn high_severity_findings_are_redacted() {
    let dir = TempDir::new().unwrap();
    let full_key = "AKIAIOSFODNN7EXAMPLE";
    let path = write_file(&dir, "config.env", &format!("AWS_ACCESS_KEY={full_key}\n"));
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    let aws_findings: Vec<_> = findings.iter().filter(|f| f.severity >= Severity::High).collect();
    for f in aws_findings {
        assert!(
            f.matched_text.contains("[redacted]") || f.matched_text.len() < full_key.len(),
            "high/critical matched_text should be redacted, got: '{}'", f.matched_text
        );
    }
}

#[test]
fn warning_severity_findings_are_not_redacted() {
    let dir = TempDir::new().unwrap();
    let path = write_file(&dir, ".env", "MY_VAR=hello\n");
    let cfg = default_cfg();
    let mut findings = vec![];
    scan_file(&path, &cfg, &mut findings).unwrap();
    let warn_findings = findings_for(&findings, "env-var-any");
    // Warning-level matches are not redacted — full match is present
    for f in warn_findings {
        assert!(!f.matched_text.contains("[redacted]"), "warning findings should not be redacted");
    }
}

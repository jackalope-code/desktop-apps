use crate::types::Severity;
use regex::Regex;
use serde::Serialize;

// ── Rule ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub severity: Severity,
    /// If true, rule only runs on .env / dotenv files.
    pub env_only: bool,
    #[serde(skip)]
    pub regex: Regex,
}

impl Rule {
    fn new(name: &str, description: &str, pattern: &str, severity: Severity) -> Self {
        Rule {
            name: name.into(),
            description: description.into(),
            severity,
            env_only: false,
            regex: Regex::new(pattern)
                .unwrap_or_else(|e| panic!("bad regex for rule '{name}': {e}")),
        }
    }

    fn env_rule(name: &str, description: &str, pattern: &str, severity: Severity) -> Self {
        let mut r = Self::new(name, description, pattern, severity);
        r.env_only = true;
        r
    }
}

// ── Built-in rules ────────────────────────────────────────────────────────────

pub fn default_rules() -> Vec<Rule> {
    vec![
        // ── Private / PEM keys ───────────────────────────────────────────────
        Rule::new("private-key-pem", "PEM-encoded private key block",
            r"-----BEGIN (RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY( BLOCK)?-----",
            Severity::Critical),
        Rule::new("ssh-private-key", "OpenSSH private key header",
            r"-----BEGIN OPENSSH PRIVATE KEY-----",
            Severity::Critical),

        // ── AWS ──────────────────────────────────────────────────────────────
        Rule::new("aws-access-key-id", "AWS access key ID (AKIA)",
            r"(?:^|[^A-Z0-9])(AKIA[0-9A-Z]{16})(?:[^A-Z0-9]|$)",
            Severity::Critical),
        Rule::new("aws-secret-access-key", "AWS secret access key (heuristic)",
            r#"(?i)aws[_\-\.]?secret[_\-\.]?(?:access[_\-\.]?)?key\s*[:=]\s*['"]?([A-Za-z0-9/+]{40})['"]?"#,
            Severity::Critical),

        // ── GitHub ───────────────────────────────────────────────────────────
        Rule::new("github-pat", "GitHub personal / OAuth / app access token",
            r"(ghp|gho|ghu|ghs|ghr|github_pat)_[A-Za-z0-9_]{20,255}",
            Severity::Critical),

        // ── Google ───────────────────────────────────────────────────────────
        Rule::new("google-api-key", "Google API key (AIza)",
            r"AIza[0-9A-Za-z\-_]{35}", Severity::Critical),
        Rule::new("google-oauth-client", "Google OAuth client ID",
            r"[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com",
            Severity::High),

        // ── Stripe ───────────────────────────────────────────────────────────
        Rule::new("stripe-live-secret", "Stripe live secret key",
            r"sk_live_[0-9a-zA-Z]{24,}", Severity::Critical),
        Rule::new("stripe-live-public", "Stripe live publishable key",
            r"pk_live_[0-9a-zA-Z]{24,}", Severity::High),
        Rule::new("stripe-test-secret", "Stripe test secret key",
            r"sk_test_[0-9a-zA-Z]{24,}", Severity::Medium),

        // ── Slack ────────────────────────────────────────────────────────────
        Rule::new("slack-token", "Slack API / bot / app token",
            r"xox[baprs]-[0-9A-Za-z\-]{10,}", Severity::Critical),
        Rule::new("slack-webhook", "Slack incoming webhook URL",
            r"https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]+",
            Severity::Critical),

        // ── SendGrid / Twilio ─────────────────────────────────────────────────
        Rule::new("sendgrid-api-key", "SendGrid API key",
            r"SG\.[A-Za-z0-9_\-]{22}\.[A-Za-z0-9_\-]{43}", Severity::Critical),
        Rule::new("twilio-api-key", "Twilio Auth token or API key SID",
            r"SK[0-9a-fA-F]{32}", Severity::Critical),

        // ── npm ──────────────────────────────────────────────────────────────
        Rule::new("npm-access-token", "npm automation / access token",
            r"npm_[A-Za-z0-9]{36}", Severity::Critical),

        // ── JWT ──────────────────────────────────────────────────────────────
        Rule::new("jwt-token", "JSON Web Token",
            r"eyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}",
            Severity::High),

        // ── Database connection strings ───────────────────────────────────────
        Rule::new("db-connection-string",
            "Database connection string with embedded credentials",
            r"(?i)(mongodb\+srv?|postgresql|postgres|mysql|mssql|redis|amqp)://[^:@\s]{1,}:[^@\s]{1,}@[^\s]+",
            Severity::Critical),

        // ── Generic patterns ─────────────────────────────────────────────────
        Rule::new("generic-api-key", "Generic API key assignment",
            r#"(?i)\bapi[_\-]?key\s*[:=]\s*['"]?([A-Za-z0-9_\-\.]{20,})"#,
            Severity::High),
        Rule::new("generic-secret", "Generic secret assignment",
            r#"(?i)\b(?:app[_\-]?|client[_\-]?)?secret(?:[_\-]?key)?\s*[:=]\s*['"]?([A-Za-z0-9_\-\./:+]{10,})"#,
            Severity::High),
        Rule::new("generic-password", "Hardcoded password assignment",
            r#"(?i)\bpassw(?:or)?d\s*[:=]\s*['"]?([^\s'"]{6,})"#,
            Severity::High),
        Rule::new("generic-token", "Generic token assignment",
            r#"(?i)\btoken\s*[:=]\s*['"]?([A-Za-z0-9_\-\.]{20,})"#,
            Severity::High),
        Rule::new("generic-auth-header",
            "Authorization header with bearer/basic credential",
            r"(?i)Authorization\s*[:=]\s*(?:Bearer|Basic)\s+([A-Za-z0-9+/=_\-\.]{10,})",
            Severity::High),
        Rule::new("private-key-assignment", "Private key value in assignment",
            r#"(?i)\bprivate[_\-]?key\s*[:=]\s*['"]?([A-Za-z0-9+/=_\-]{30,})"#,
            Severity::Critical),
        Rule::new("heroku-api-key", "Heroku API key (UUID format)",
            r#"(?i)heroku[^\s]*\s*[:=]\s*['"]?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}['"]?"#,
            Severity::Critical),
        Rule::new("basic-auth-url", "HTTP(S) URL with embedded username:password",
            r"https?://[^:@\s]{1,}:[^@\s]{3,}@[^\s]+", Severity::High),
        Rule::new("mailgun-api-key", "Mailgun API key",
            r#"(?i)mailgun[_\-\. ]?(?:api[_\-\. ]?)?key\s*[:=]\s*['"]?key-[0-9a-zA-Z]{32}['"]?"#,
            Severity::Critical),

        // ── Hardcoded user home-directory paths ──────────────────────────────
        Rule::new("user-dir-path",
            "Hardcoded user home-directory path (exposes local username / machine path)",
            r"(?:/(?:home|Users)/[A-Za-z0-9_.\-]+(?:/|$)|[A-Za-z]:\\[Uu]sers\\[A-Za-z0-9_.\-]+(?:\\|$))",
            Severity::Warning),

        // ── .env – ALL uppercase VAR=VALUE .> Warning ─────────────────────────
        Rule::env_rule("env-var-any",
            "Environment variable assignment (all .env vars reported)",
            r"^([A-Z][A-Z0-9_]{1,})\s*=\s*(.+)$", Severity::Warning),
        // Higher severity for sensitive variable names
        Rule::env_rule("env-var-sensitive",
            "Sensitive environment variable name",
            r#"(?i)^((?:[A-Za-z0-9_]*(?:secret|password|passwd|token|api[_\-]?key|auth|private[_\-]?key|credential|access[_\-]?key|client[_\-]?secret|signing[_\-]?key)[A-Za-z0-9_]*))\s*=\s*([^\s'"].*)$"#,
            Severity::High),
    ]
}

/// Build a single user-supplied rule from a raw regex string.
pub fn user_rule(index: usize, pattern: &str) -> anyhow::Result<Rule> {
    let regex = Regex::new(pattern)
        .map_err(|e| anyhow::anyhow!("invalid pattern '{}': {}", pattern, e))?;
    Ok(Rule {
        name: format!("custom-{}", index),
        description: format!("User-supplied pattern: {}", pattern),
        severity: Severity::High,
        env_only: false,
        regex,
    })
}

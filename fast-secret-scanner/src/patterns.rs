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
    /// If true, this rule is categorised as infrastructure disclosure
    /// (IP addresses, internal hostnames, public endpoints).
    pub infra: bool,
    #[serde(skip)]
    pub regex: Regex,
    /// When set, a line that also matches this pattern is silently skipped
    /// (suppresses false positives).
    #[serde(skip)]
    pub allowlist_regex: Option<Regex>,
}

impl Rule {
    fn new(name: &str, description: &str, pattern: &str, severity: Severity) -> Self {
        Rule {
            name: name.into(),
            description: description.into(),
            severity,
            env_only: false,
            infra: false,
            allowlist_regex: None,
            regex: Regex::new(pattern)
                .unwrap_or_else(|e| panic!("bad regex for rule '{name}': {e}")),
        }
    }

    fn env_rule(name: &str, description: &str, pattern: &str, severity: Severity) -> Self {
        let mut r = Self::new(name, description, pattern, severity);
        r.env_only = true;
        r
    }

    fn infra_rule(name: &str, description: &str, pattern: &str, severity: Severity) -> Self {
        let mut r = Self::new(name, description, pattern, severity);
        r.infra = true;
        r
    }

    /// Chain a per-rule allowlist: lines that match this pattern are skipped.
    pub fn with_allowlist(mut self, pattern: &str) -> Self {
        self.allowlist_regex = Some(
            Regex::new(pattern)
                .unwrap_or_else(|e| panic!("bad allowlist regex for rule '{}': {e}", self.name)),
        );
        self
    }
}

// ── Built-in rules ────────────────────────────────────────────────────────────

pub fn default_rules() -> Vec<Rule> {
    vec![
        // ── Private / PEM keys ───────────────────────────────────────────────
        Rule::new(
            "private-key-pem",
            "PEM-encoded private key block",
            r"-----BEGIN (RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY( BLOCK)?-----",
            Severity::Critical,
        ),
        Rule::new(
            "ssh-private-key",
            "OpenSSH private key header",
            r"-----BEGIN OPENSSH PRIVATE KEY-----",
            Severity::Critical,
        ),
        // ── AWS ──────────────────────────────────────────────────────────────
        Rule::new(
            "aws-access-key-id",
            "AWS access key ID (AKIA)",
            r"(?:^|[^A-Z0-9])(AKIA[0-9A-Z]{16})(?:[^A-Z0-9]|$)",
            Severity::Critical,
        ),
        Rule::new(
            "aws-secret-access-key",
            "AWS secret access key (heuristic)",
            r#"(?i)aws[_\-\.]?secret[_\-\.]?(?:access[_\-\.]?)?key\s*[:=]\s*['"]?([A-Za-z0-9/+]{40})['"]?"#,
            Severity::Critical,
        ),
        // ── GitHub ───────────────────────────────────────────────────────────
        Rule::new(
            "github-pat",
            "GitHub personal / OAuth / app access token",
            r"(ghp|gho|ghu|ghs|ghr|github_pat)_[A-Za-z0-9_]{20,255}",
            Severity::Critical,
        ),
        // ── Google ───────────────────────────────────────────────────────────
        Rule::new(
            "google-api-key",
            "Google API key (AIza)",
            r"AIza[0-9A-Za-z\-_]{35}",
            Severity::Critical,
        ),
        Rule::new(
            "google-oauth-client",
            "Google OAuth client ID",
            r"[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com",
            Severity::High,
        ),
        // ── Stripe ───────────────────────────────────────────────────────────
        Rule::new(
            "stripe-live-secret",
            "Stripe live secret key",
            r"sk_live_[0-9a-zA-Z]{24,}",
            Severity::Critical,
        ),
        Rule::new(
            "stripe-live-public",
            "Stripe live publishable key",
            r"pk_live_[0-9a-zA-Z]{24,}",
            Severity::High,
        ),
        Rule::new(
            "stripe-test-secret",
            "Stripe test secret key",
            r"sk_test_[0-9a-zA-Z]{24,}",
            Severity::Medium,
        ),
        // ── Slack ────────────────────────────────────────────────────────────
        Rule::new(
            "slack-token",
            "Slack API / bot / app token",
            r"xox[baprs]-[0-9A-Za-z\-]{10,}",
            Severity::Critical,
        ),
        Rule::new(
            "slack-webhook",
            "Slack incoming webhook URL",
            r"https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]+",
            Severity::Critical,
        ),
        // ── SendGrid / Twilio ─────────────────────────────────────────────────
        Rule::new(
            "sendgrid-api-key",
            "SendGrid API key",
            r"SG\.[A-Za-z0-9_\-]{22}\.[A-Za-z0-9_\-]{43}",
            Severity::Critical,
        ),
        Rule::new(
            "twilio-api-key",
            "Twilio Auth token or API key SID",
            r"SK[0-9a-fA-F]{32}",
            Severity::Critical,
        ),
        // ── npm ──────────────────────────────────────────────────────────────
        Rule::new(
            "npm-access-token",
            "npm automation / access token",
            r"npm_[A-Za-z0-9]{36}",
            Severity::Critical,
        ),
        // ── JWT ──────────────────────────────────────────────────────────────
        Rule::new(
            "jwt-token",
            "JSON Web Token",
            r"eyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}",
            Severity::High,
        ),
        // ── Database connection strings ───────────────────────────────────────
        Rule::new(
            "db-connection-string",
            "Database connection string with embedded credentials",
            r"(?i)(mongodb\+srv?|postgresql|postgres|mysql|mssql|redis|amqp)://[^:@\s]{1,}:[^@\s]{1,}@[^\s]+",
            Severity::Critical,
        ),
        // ── Generic patterns ─────────────────────────────────────────────────
        Rule::new(
            "generic-api-key",
            "Generic API key assignment",
            r#"(?i)\bapi[_\-]?key\s*[:=]\s*['"]?([A-Za-z0-9_\-\.]{20,})"#,
            Severity::High,
        ),
        Rule::new(
            "generic-secret",
            "Generic secret assignment",
            r#"\b(?:app[_\-]?|client[_\-]?)?secret(?:[_\-]?key)?\s*[:=]\s*['"]([a-zA-Z0-9_\-\./:+]{10,})['"]"#,
            Severity::High,
        ),
        Rule::new(
            "generic-password",
            "Hardcoded password assignment",
            r#"\bpassw(?:or)?d\s*[:=]\s*['"]?([a-zA-Z0-9_\-\./:+]{6,})['"]?"#,
            Severity::High,        ).with_allowlist(
            r#"(?i)\bpassw(?:or)?d\s*:\s*(?:string|boolean|number|void|any|null|undefined|object|never|int|float|double|bool|str|bytes|char|usize|u8|u16|u32|u64|i8|i16|i32|i64)\b"#        ),
        Rule::new(
            "generic-token",
            "Generic token assignment",
            r#"\btoken\s*[:=]\s*['"]([a-zA-Z0-9_\-]{20,})['"]"#,
            Severity::High,
        ),
        Rule::new(
            "generic-auth-header",
            "Authorization header with bearer/basic credential",
            r"(?i)Authorization\s*[:=]\s*(?:Bearer|Basic)\s+([A-Za-z0-9+/=_\-\.]{10,})",
            Severity::High,
        ),
        Rule::new(
            "private-key-assignment",
            "Private key value in assignment",
            r#"(?i)\bprivate[_\-]?key\s*[:=]\s*['"]?([A-Za-z0-9+/=_\-]{30,})"#,
            Severity::Critical,
        ),
        Rule::new(
            "heroku-api-key",
            "Heroku API key (UUID format)",
            r#"(?i)heroku[^\s]*\s*[:=]\s*['"]?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}['"]?"#,
            Severity::Critical,
        ),
        Rule::new(
            "basic-auth-url",
            "HTTP(S) URL with embedded username:password",
            r"https?://[^:@/\s]+:[^@\s]{3,}@(?:[a-zA-Z0-9\-]+\.)+[a-zA-Z]{2,}",
            Severity::High,
        ),
        Rule::new(
            "mailgun-api-key",
            "Mailgun API key",
            r#"(?i)mailgun[_\-\. ]?(?:api[_\-\. ]?)?key\s*[:=]\s*['"]?key-[0-9a-zA-Z]{32}['"]?"#,
            Severity::Critical,
        ),
        // ── Cloud / SaaS API keys ──────────────────────────────────────────────────
        Rule::new(
            "openai-api-key",
            "OpenAI API key",
            r"sk-[A-Za-z0-9]{32,}",
            Severity::Critical,
        ),
        Rule::new(
            "anthropic-api-key",
            "Anthropic API key",
            r"sk-ant-[A-Za-z0-9\-_]{32,}",
            Severity::Critical,
        ),
        Rule::new(
            "azure-subscription-key",
            "Azure Cognitive Services / subscription key header",
            r#"(?i)(?:ocp-apim-subscription-key|azure[_\-]?(?:api[_\-]?)?key)\s*[:=]\s*['"']?([0-9a-fA-F]{32})['"']?"#,
            Severity::Critical,
        ),
        Rule::new(
            "cloudflare-api-token",
            "Cloudflare API token assignment",
            r#"(?i)cloudflare[_\-\.]?(?:api[_\-\.]?)?(?:token|key)\s*[:=]\s*['"']?([A-Za-z0-9_\-]{37})['"']?"#,
            Severity::High,
        ),
        Rule::new(
            "supabase-service-key",
            "Supabase service role / anon JWT",
            r#"eyJhbGci[A-Za-z0-9_\-\.]+\.[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+"#,
            Severity::Critical,
        ),
        Rule::new(
            "vercel-token",
            "Vercel authentication token",
            r#"(?i)vercel[_\-]?(?:api[_\-]?)?token\s*[:=]\s*['"]?([A-Za-z0-9]{24,})"#,
            Severity::Critical,
        ),
        Rule::new(
            "linear-api-key",
            "Linear API key",
            r"lin_api_[A-Za-z0-9]{36,}",
            Severity::Critical,
        ),
        Rule::new(
            "notion-token",
            "Notion integration token",
            r"secret_[A-Za-z0-9]{36,}",
            Severity::Critical,
        ),
        Rule::new(
            "pagerduty-api-key",
            "PagerDuty API key",
            r#"(?i)pagerduty[_\-]?(?:api[_\-]?)?(?:key|token)\s*[:=]\s*['"']?([A-Za-z0-9+\-_]{20,})['"']?"#,
            Severity::Critical,
        ),
        Rule::new(
            "datadog-api-key",
            "Datadog API or application key",
            r#"(?i)(?:dd[_\-]?api[_\-]?key|datadog[_\-]?(?:api|app)[_\-]?key)\s*[:=]\s*['"']?([0-9a-fA-F]{32})['"']?"#,
            Severity::Critical,
        ),
        Rule::new(
            "digitalocean-token",
            "DigitalOcean personal access token",
            r"dop_v1_[A-Za-z0-9]{64}",
            Severity::Critical,
        ),
        Rule::new(
            "sentry-dsn",
            "Sentry DSN (contains ingest key)",
            r"https://[0-9a-fA-F]{32}@(?:o\d+\.)?(?:ingest\.)?sentry\.io/\d+",
            Severity::High,
        ),
        Rule::new(
            "okta-api-token",
            "Okta API token",
            r#"(?i)okta[_\-]?(?:api[_\-]?)?token\s*[:=]\s*['"']?([A-Za-z0-9\-_]{40,})['"']?"#,
            Severity::Critical,
        ),
        // ── Infrastructure disclosure ────────────────────────────────────────────
        Rule::infra_rule(
            "infra-ipv4-address",
            "Non-loopback IPv4 address (potential infrastructure disclosure)",
            r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b",
            Severity::Warning,
        ).with_allowlist(r"\b(?:127\.0\.0\.1|0\.0\.0\.0|255\.255\.255\.255)\b"),
        Rule::infra_rule(
            "infra-internal-hostname",
            "Internal hostname pattern (e.g. service.internal, service.corp)",
            r"\b[a-z0-9][a-z0-9\-]{1,61}[a-z0-9]\.(?:internal|corp|local|intranet|lan|priv)\b",
            Severity::Warning,
        ),
        Rule::infra_rule(
            "infra-public-endpoint",
            "Hardcoded internal service endpoint URL",
            r"https?://[a-z0-9][a-z0-9\-\.]{2,}\.(?:internal|corp|local|intranet|lan|priv|private)[/:]" ,
            Severity::Warning,
        ).with_allowlist(r"\blocalhost\b"),
        // ── Hardcoded user home-directory paths ───────────────────────────────────
        // Fires when a specific username is present in a path, revealing the
        // local account name.  Bare parent dirs (/home, C:\Users) are not flagged.
        Rule::new(
            "user-dir-path",
            "Hardcoded user home-directory path (reveals local username)",
            r"(?:/(?:home|Users)/[A-Za-z0-9_.\-]+(?:/|$)|[A-Za-z]:\\[Uu]sers\\[A-Za-z0-9_.\-]+(?:\\|$))",
            Severity::Warning,
        ),
        // ── .env – ALL uppercase VAR=VALUE .> Warning ─────────────────────────
        Rule::env_rule(
            "env-var-any",
            "Environment variable assignment (all .env vars reported)",
            r"^([A-Z][A-Z0-9_]{1,})\s*=\s*(.+)$",
            Severity::Warning,
        ),
        // Higher severity for sensitive variable names
        Rule::env_rule(
            "env-var-sensitive",
            "Sensitive environment variable name",
            r#"(?i)^((?:[A-Za-z0-9_]*(?:secret|password|passwd|token|api[_\-]?key|auth|private[_\-]?key|credential|access[_\-]?key|client[_\-]?secret|signing[_\-]?key)[A-Za-z0-9_]*))\s*=\s*([^\s'"].*)$"#,
            Severity::High,
        ),
    ]
}

/// Build a single user-supplied rule from a raw regex string.
pub fn user_rule(index: usize, pattern: &str) -> anyhow::Result<Rule> {
    let regex =
        Regex::new(pattern).map_err(|e| anyhow::anyhow!("invalid pattern '{}': {}", pattern, e))?;
    Ok(Rule {
        name: format!("custom-{}", index),
        description: format!("User-supplied pattern: {}", pattern),
        severity: Severity::High,
        env_only: false,
        infra: false,
        allowlist_regex: None,
        regex,
    })
}

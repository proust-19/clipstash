use lazy_static::lazy_static;
use regex::Regex;

/// Describes why content was flagged as sensitive
pub enum SensitiveMatch {
    ApiKey(&'static str),
    PrivateKey,
    CreditCard,
    JwtToken,
    Password,
}

impl SensitiveMatch {
    pub fn label(&self) -> &'static str {
        match self {
            SensitiveMatch::ApiKey(provider) => provider,
            SensitiveMatch::PrivateKey => "Private Key",
            SensitiveMatch::CreditCard => "Credit Card",
            SensitiveMatch::JwtToken => "JWT Token",
            SensitiveMatch::Password => "Possible Password",
        }
    }
}

lazy_static! {
    // API key patterns
    static ref RE_AWS_KEY: Regex = Regex::new(r"(?i)AKIA[0-9A-Z]{16}").unwrap();
    static ref RE_GITHUB_TOKEN: Regex = Regex::new(r"gh[pousr]_[A-Za-z0-9_]{36,}").unwrap();
    static ref RE_OPENAI_KEY: Regex = Regex::new(r"sk-[A-Za-z0-9_-]{20,}").unwrap();
    static ref RE_STRIPE_KEY: Regex = Regex::new(r"(?i)(sk_live_|pk_live_|sk_test_|pk_test_)[A-Za-z0-9]{20,}").unwrap();
    static ref RE_SLACK_TOKEN: Regex = Regex::new(r"xox[baprs]-[0-9A-Za-z\-]{10,}").unwrap();
    static ref RE_GENERIC_API: Regex = Regex::new(r"(?i)(api[_-]?key|api[_-]?secret|access[_-]?token)\s*[:=]\s*\S{10,}").unwrap();

    // Private keys
    static ref RE_PRIVATE_KEY: Regex = Regex::new(r"-----BEGIN\s+(RSA\s+|EC\s+|DSA\s+|OPENSSH\s+)?PRIVATE\s+KEY-----").unwrap();

    // Credit card numbers (Visa, MasterCard, Amex, Discover)
    static ref RE_CREDIT_CARD: Regex = Regex::new(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|6(?:011|5[0-9]{2})[0-9]{12})\b").unwrap();
    // With spaces/dashes
    static ref RE_CREDIT_CARD_SPACED: Regex = Regex::new(r"\b(?:4[0-9]{3}[\s-]?[0-9]{4}[\s-]?[0-9]{4}[\s-]?[0-9]{4}|5[1-5][0-9]{2}[\s-]?[0-9]{4}[\s-]?[0-9]{4}[\s-]?[0-9]{4})\b").unwrap();

    // JWT tokens
    static ref RE_JWT: Regex = Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap();

    // Password-like strings: short, no spaces, mixed chars
    static ref RE_PASSWORD: Regex = Regex::new(r"^[^\s]{8,40}$").unwrap();
    static ref RE_PASSWORD_COMPLEXITY: Regex = Regex::new(r"[!@#$%^&*()\+\-=\[\]{};:\\|,.<>/?]").unwrap();
}

/// Check if clipboard content looks like sensitive data.
/// Returns Some(SensitiveMatch) if sensitive, None if safe to store.
pub fn detect_sensitive(content: &str) -> Option<SensitiveMatch> {
    let trimmed = content.trim();

    // Skip very long or multi-line text (unlikely to be a password)
    // But still check for embedded keys/tokens in longer text

    // API Keys
    if RE_AWS_KEY.is_match(trimmed) {
        return Some(SensitiveMatch::ApiKey("AWS Key"));
    }
    if RE_GITHUB_TOKEN.is_match(trimmed) {
        return Some(SensitiveMatch::ApiKey("GitHub Token"));
    }
    if RE_OPENAI_KEY.is_match(trimmed) {
        return Some(SensitiveMatch::ApiKey("OpenAI Key"));
    }
    if RE_STRIPE_KEY.is_match(trimmed) {
        return Some(SensitiveMatch::ApiKey("Stripe Key"));
    }
    if RE_SLACK_TOKEN.is_match(trimmed) {
        return Some(SensitiveMatch::ApiKey("Slack Token"));
    }
    if RE_GENERIC_API.is_match(trimmed) {
        return Some(SensitiveMatch::ApiKey("API Key"));
    }

    // Private keys
    if RE_PRIVATE_KEY.is_match(trimmed) {
        return Some(SensitiveMatch::PrivateKey);
    }

    // JWT tokens
    if RE_JWT.is_match(trimmed) {
        return Some(SensitiveMatch::JwtToken);
    }

    // Credit cards (only check single-line content)
    if trimmed.lines().count() <= 2 {
        let digits_only: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if (RE_CREDIT_CARD.is_match(&digits_only) || RE_CREDIT_CARD_SPACED.is_match(trimmed))
            && luhn_check(&digits_only)
        {
            return Some(SensitiveMatch::CreditCard);
        }
    }

    // Password-like: single line, 8-40 chars, has special chars + digits + letters
    if trimmed.lines().count() == 1 && RE_PASSWORD.is_match(trimmed) {
        let has_letter = trimmed.chars().any(|c| c.is_alphabetic());
        let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
        let has_special = RE_PASSWORD_COMPLEXITY.is_match(trimmed);

        // Must have at least 2 of 3 complexity classes AND look random (not a normal word/URL)
        let complexity = [has_letter, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if complexity >= 3 && !looks_like_normal_text(trimmed) {
            return Some(SensitiveMatch::Password);
        }
    }

    None
}

/// Simple Luhn algorithm check for credit card validation
fn luhn_check(number: &str) -> bool {
    if number.len() < 13 || number.len() > 19 {
        return false;
    }
    let mut sum = 0u32;
    let mut double = false;
    for ch in number.chars().rev() {
        if let Some(d) = ch.to_digit(10) {
            let val = if double {
                let v = d * 2;
                if v > 9 { v - 9 } else { v }
            } else {
                d
            };
            sum += val;
            double = !double;
        } else {
            return false;
        }
    }
    sum % 10 == 0
}

/// Heuristic: does this look like normal text (URL, file path, command, English word)?
fn looks_like_normal_text(s: &str) -> bool {
    // URLs
    if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://") {
        return true;
    }
    // File paths
    if s.starts_with('/') || s.starts_with("~/") || s.contains(":\\") {
        return true;
    }
    // Email addresses
    if s.contains('@') && s.contains('.') && !s.contains(' ') {
        return true;
    }
    // Common shell commands
    if s.starts_with("sudo ")
        || s.starts_with("cd ")
        || s.starts_with("ls ")
        || s.starts_with("git ")
        || s.starts_with("cargo ")
        || s.starts_with("npm ")
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key() {
        assert!(detect_sensitive("AKIAIOSFODNN7EXAMPLE").is_some());
    }

    #[test]
    fn test_github_token() {
        assert!(detect_sensitive("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij123456").is_some());
    }

    #[test]
    fn test_jwt() {
        assert!(detect_sensitive("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abcdef123456").is_some());
    }

    #[test]
    fn test_private_key() {
        assert!(detect_sensitive("-----BEGIN RSA PRIVATE KEY-----\nMIIE...").is_some());
    }

    #[test]
    fn test_normal_text() {
        assert!(detect_sensitive("Hello world, this is normal text").is_none());
    }

    #[test]
    fn test_url() {
        assert!(detect_sensitive("https://github.com/user/repo").is_none());
    }

    #[test]
    fn test_password_like() {
        assert!(detect_sensitive("P@ssw0rd!2024").is_some());
    }
}

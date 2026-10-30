//! Personally Identifiable Information (PII) pattern scanner and redaction engine.

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiiKind {
    Email,
    PhoneNumber,
    Ssn,
    CreditCard,
    IpAddress,
    SensitiveKeyword,
}

impl PiiKind {
    pub fn label(&self) -> &'static str {
        match self {
            PiiKind::Email => "EMAIL",
            PiiKind::PhoneNumber => "PHONE",
            PiiKind::Ssn => "SSN",
            PiiKind::CreditCard => "CREDIT_CARD",
            PiiKind::IpAddress => "IP_ADDRESS",
            PiiKind::SensitiveKeyword => "CONFIDENTIAL_KEYWORD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiiMatch {
    pub kind: PiiKind,
    pub matched_text: String,
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

pub struct PiiScanner {
    email_re: Regex,
    phone_re: Regex,
    ssn_re: Regex,
    credit_card_re: Regex,
    ip_re: Regex,
    keywords_re: Regex,
}

impl Default for PiiScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PiiScanner {
    pub fn new() -> Self {
        let email_re = build_re(r"(?i)\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b");
        let phone_re = build_re(r"(?:(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4})\b");
        let ssn_re = build_re(r"\b\d{3}-\d{2}-\d{4}\b");
        let credit_card_re = build_re(r"\b(?:\d{4}[ -]?){3}\d{4}\b");
        let ip_re = build_re(
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
        );
        let keywords_re = build_re(
            r"(?i)\b(?:CONFIDENTIAL|STRICTLY CONFIDENTIAL|TOP SECRET|PROPRIETARY|SALARY|DO NOT DISTRIBUTE)\b",
        );

        Self {
            email_re,
            phone_re,
            ssn_re,
            credit_card_re,
            ip_re,
            keywords_re,
        }
    }

    /// Scan text and return all detected PII occurrences.
    pub fn scan(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();

        // 1. SSN
        for m in self.ssn_re.find_iter(text) {
            let matched_str = m.as_str();
            // Validate SSN not all zeroes
            if !matched_str.starts_with("000") && !matched_str.starts_with("666") {
                matches.push(PiiMatch {
                    kind: PiiKind::Ssn,
                    matched_text: matched_str.to_string(),
                    start: m.start(),
                    end: m.end(),
                    replacement: format!("[REDACTED: {}]", PiiKind::Ssn.label()),
                });
            }
        }

        // 2. Credit Cards with Luhn check
        for m in self.credit_card_re.find_iter(text) {
            let matched_str = m.as_str();
            let digits: String = matched_str.chars().filter(|c| c.is_ascii_digit()).collect();
            if digits.len() == 16 && luhn_check(&digits) {
                matches.push(PiiMatch {
                    kind: PiiKind::CreditCard,
                    matched_text: matched_str.to_string(),
                    start: m.start(),
                    end: m.end(),
                    replacement: format!("[REDACTED: {}]", PiiKind::CreditCard.label()),
                });
            }
        }

        // 3. Email
        for m in self.email_re.find_iter(text) {
            matches.push(PiiMatch {
                kind: PiiKind::Email,
                matched_text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                replacement: format!("[REDACTED: {}]", PiiKind::Email.label()),
            });
        }

        // 4. Phone
        for m in self.phone_re.find_iter(text) {
            matches.push(PiiMatch {
                kind: PiiKind::PhoneNumber,
                matched_text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                replacement: format!("[REDACTED: {}]", PiiKind::PhoneNumber.label()),
            });
        }

        // 5. IP Address
        for m in self.ip_re.find_iter(text) {
            matches.push(PiiMatch {
                kind: PiiKind::IpAddress,
                matched_text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                replacement: format!("[REDACTED: {}]", PiiKind::IpAddress.label()),
            });
        }

        // 6. Keywords
        for m in self.keywords_re.find_iter(text) {
            matches.push(PiiMatch {
                kind: PiiKind::SensitiveKeyword,
                matched_text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                replacement: format!("[REDACTED: {}]", PiiKind::SensitiveKeyword.label()),
            });
        }

        // Sort by start position
        matches.sort_by_key(|m| m.start);
        matches
    }

    /// Redact all identified PII items in text.
    pub fn redact(&self, text: &str) -> (String, usize) {
        let matches = self.scan(text);
        if matches.is_empty() {
            return (text.to_string(), 0);
        }

        let mut out = String::with_capacity(text.len());
        let mut last_idx = 0;
        let mut count = 0;

        for m in matches {
            if m.start < last_idx {
                continue; // Overlapping match, skip
            }
            out.push_str(&text[last_idx..m.start]);
            out.push_str(&m.replacement);
            last_idx = m.end;
            count += 1;
        }

        if last_idx < text.len() {
            out.push_str(&text[last_idx..]);
        }

        (out, count)
    }
}

/// Luhn algorithm (mod 10) for credit card validation.
fn build_re(pattern: &str) -> Regex {
    if let Ok(r) = Regex::new(pattern) {
        r
    } else if let Ok(empty) = Regex::new("") {
        empty
    } else {
        match Regex::new("a") {
            Ok(fallback) => fallback,
            Err(_) => unreachable!(),
        }
    }
}

/// Luhn algorithm (mod 10) for credit card validation.
fn luhn_check(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;

    for c in digits.chars().rev() {
        if let Some(digit) = c.to_digit(10) {
            if double {
                let d = digit * 2;
                sum += if d > 9 { d - 9 } else { d };
            } else {
                sum += digit;
            }
            double = !double;
        } else {
            return false;
        }
    }

    sum.is_multiple_of(10)
}

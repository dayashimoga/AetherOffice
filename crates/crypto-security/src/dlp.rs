//! Enterprise Data Loss Prevention (DLP) Policy Engine.
//!
//! Evaluates document content against configurable compliance policies,
//! detects sensitive data (SSN, credit cards, healthcare records, IP addresses),
//! and determines governance enforcement actions (Allow, Warn, Redact, Block).

use crate::pii::{PiiKind, PiiScanner};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Severity rating for DLP violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DlpSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DlpSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Action to enforce upon policy violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DlpAction {
    Allow,
    Warn,
    RedactAndAllow,
    Block,
    Quarantine,
}

impl DlpAction {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Block | Self::Quarantine)
    }

    pub fn requires_redaction(&self) -> bool {
        matches!(self, Self::RedactAndAllow)
    }
}

/// Pattern target for a DLP rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum DlpPattern {
    Pii(PiiKind),
    Keyword(String),
    RegexPattern(String),
}

/// Individual DLP rule defining sensitive data patterns, threshold, and enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DlpRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub patterns: Vec<DlpPattern>,
    pub threshold: usize,
    pub severity: DlpSeverity,
    pub action: DlpAction,
}

impl DlpRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        patterns: Vec<DlpPattern>,
        threshold: usize,
        severity: DlpSeverity,
        action: DlpAction,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            patterns,
            threshold: threshold.max(1),
            severity,
            action,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Enterprise DLP compliance policy containing prioritized rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DlpPolicy {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub rules: Vec<DlpRule>,
    pub default_action: DlpAction,
}

impl DlpPolicy {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version,
            rules: Vec::new(),
            default_action: DlpAction::Allow,
        }
    }

    pub fn add_rule(&mut self, rule: DlpRule) {
        self.rules.push(rule);
    }

    /// Default enterprise DLP policy guarding SSNs, payment card data, and confidential keywords.
    pub fn default_enterprise_policy() -> Self {
        let mut policy = Self::new("ent-dlp-std", "Standard Enterprise Governance", 1);

        policy.add_rule(
            DlpRule::new(
                "rule-pci-dss",
                "Payment Card Data Protection",
                vec![DlpPattern::Pii(PiiKind::CreditCard)],
                1,
                DlpSeverity::Critical,
                DlpAction::Block,
            )
            .with_description("Prohibits unencrypted transmission of credit card numbers"),
        );

        policy.add_rule(
            DlpRule::new(
                "rule-ssn-redact",
                "Social Security Number Sanitization",
                vec![DlpPattern::Pii(PiiKind::Ssn)],
                1,
                DlpSeverity::High,
                DlpAction::RedactAndAllow,
            )
            .with_description("Mandates redaction of SSNs prior to export"),
        );

        policy.add_rule(
            DlpRule::new(
                "rule-confidential-kw",
                "Confidential Document Keywords",
                vec![
                    DlpPattern::Keyword("RESTRICTED".to_string()),
                    DlpPattern::Keyword("STRICTLY PRIVATE".to_string()),
                    DlpPattern::Keyword("TOP SECRET".to_string()),
                ],
                1,
                DlpSeverity::Medium,
                DlpAction::Warn,
            )
            .with_description("Warns users when internal confidential markings are detected"),
        );

        policy
    }

    /// Strict HIPAA healthcare policy for medical record identifiers.
    pub fn strict_hipaa_policy() -> Self {
        let mut policy = Self::new("hipaa-phi-strict", "HIPAA PHI Strict Protection", 1);
        policy.default_action = DlpAction::Allow;

        policy.add_rule(
            DlpRule::new(
                "hipaa-ssn",
                "Patient SSN Protection",
                vec![DlpPattern::Pii(PiiKind::Ssn)],
                1,
                DlpSeverity::Critical,
                DlpAction::Block,
            )
            .with_description("Strict HIPAA block on patient SSNs"),
        );

        policy.add_rule(
            DlpRule::new(
                "hipaa-contact",
                "Patient Contact Information",
                vec![
                    DlpPattern::Pii(PiiKind::Email),
                    DlpPattern::Pii(PiiKind::PhoneNumber),
                ],
                2,
                DlpSeverity::High,
                DlpAction::RedactAndAllow,
            )
            .with_description("Auto-redacts patient contact identifiers"),
        );

        policy
    }
}

/// Match detected by a DLP rule evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DlpMatch {
    pub rule_id: String,
    pub pattern_label: String,
    pub matched_text: String,
    pub start: usize,
    pub end: usize,
    pub severity: DlpSeverity,
    pub replacement: String,
}

/// Comprehensive report of DLP policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DlpEvaluationResult {
    pub policy_id: String,
    pub policy_version: u32,
    pub action: DlpAction,
    pub highest_severity: Option<DlpSeverity>,
    pub matches: Vec<DlpMatch>,
    pub triggered_rule_ids: Vec<String>,
    pub is_violation: bool,
    pub summary: String,
}

/// High-performance DLP text and document scanner.
pub struct DlpEvaluator;

impl DlpEvaluator {
    /// Evaluates plaintext content against a DLP policy.
    pub fn evaluate_text(policy: &DlpPolicy, text: &str) -> DlpEvaluationResult {
        let scanner = PiiScanner::new();
        let mut all_matches = Vec::new();
        let mut triggered_rule_ids = Vec::new();
        let mut highest_severity: Option<DlpSeverity> = None;
        let mut decisive_action = policy.default_action;

        for rule in &policy.rules {
            let mut rule_matches = Vec::new();

            for pattern in &rule.patterns {
                match pattern {
                    DlpPattern::Pii(kind) => {
                        let pii_hits = scanner.scan(text);
                        for hit in pii_hits {
                            if hit.kind == *kind {
                                rule_matches.push(DlpMatch {
                                    rule_id: rule.id.clone(),
                                    pattern_label: kind.label().to_string(),
                                    matched_text: hit.matched_text,
                                    start: hit.start,
                                    end: hit.end,
                                    severity: rule.severity,
                                    replacement: hit.replacement,
                                });
                            }
                        }
                    }
                    DlpPattern::Keyword(kw) => {
                        let kw_lower = kw.to_lowercase();
                        let text_lower = text.to_lowercase();
                        let mut start = 0;
                        while let Some(pos) = text_lower[start..].find(&kw_lower) {
                            let match_start = start + pos;
                            let match_end = match_start + kw.len();
                            rule_matches.push(DlpMatch {
                                rule_id: rule.id.clone(),
                                pattern_label: format!("KEYWORD: {kw}"),
                                matched_text: text[match_start..match_end].to_string(),
                                start: match_start,
                                end: match_end,
                                severity: rule.severity,
                                replacement: "[CONFIDENTIAL]".to_string(),
                            });
                            start = match_end;
                        }
                    }
                    DlpPattern::RegexPattern(pattern_str) => {
                        if let Ok(re) = Regex::new(pattern_str) {
                            for mat in re.find_iter(text) {
                                rule_matches.push(DlpMatch {
                                    rule_id: rule.id.clone(),
                                    pattern_label: format!("REGEX: {pattern_str}"),
                                    matched_text: mat.as_str().to_string(),
                                    start: mat.start(),
                                    end: mat.end(),
                                    severity: rule.severity,
                                    replacement: "[REDACTED]".to_string(),
                                });
                            }
                        }
                    }
                }
            }

            // Check if match occurrences satisfy rule threshold
            if rule_matches.len() >= rule.threshold {
                triggered_rule_ids.push(rule.id.clone());

                if highest_severity.is_none() || Some(rule.severity) > highest_severity {
                    highest_severity = Some(rule.severity);
                }

                if rule.action > decisive_action {
                    decisive_action = rule.action;
                }

                all_matches.extend(rule_matches);
            }
        }

        let is_violation = !triggered_rule_ids.is_empty();
        let summary = if is_violation {
            format!(
                "DLP Policy '{}' triggered {} rule(s) (Action: {:?}, Matches: {})",
                policy.name,
                triggered_rule_ids.len(),
                decisive_action,
                all_matches.len()
            )
        } else {
            format!("DLP Policy '{}' passed with zero violations", policy.name)
        };

        DlpEvaluationResult {
            policy_id: policy.id.clone(),
            policy_version: policy.version,
            action: decisive_action,
            highest_severity,
            matches: all_matches,
            triggered_rule_ids,
            is_violation,
            summary,
        }
    }

    /// Evaluates text and returns redacted content if policy prescribes redaction.
    pub fn evaluate_and_redact(policy: &DlpPolicy, text: &str) -> (DlpEvaluationResult, String) {
        let result = Self::evaluate_text(policy, text);
        if result.action != DlpAction::RedactAndAllow && !result.action.requires_redaction() {
            return (result, text.to_string());
        }

        if result.matches.is_empty() {
            return (result, text.to_string());
        }

        // Sort matches by start position descending to perform in-place replacement without offset drift
        let mut sorted_matches = result.matches.clone();
        sorted_matches.sort_by_key(|a| std::cmp::Reverse(a.start));

        let mut redacted = text.to_string();
        for m in sorted_matches {
            if m.start < redacted.len() && m.end <= redacted.len() && m.start <= m.end {
                redacted.replace_range(m.start..m.end, &m.replacement);
            }
        }

        (result, redacted)
    }
}

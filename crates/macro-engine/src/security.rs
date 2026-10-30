use crate::ast::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    FileIo,
    Network,
    SystemCommand,
    Environment,
    Registry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    PromptRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: Severity,
    pub capability: Option<Capability>,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    pub risk_level: RiskLevel,
    pub findings: Vec<SecurityFinding>,
    pub required_capabilities: Vec<Capability>,
}

impl SecurityAuditReport {
    pub fn is_safe_to_execute(&self) -> bool {
        self.risk_level <= RiskLevel::Low && self.required_capabilities.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp_millis: u64,
    pub capability: Capability,
    pub action: String,
    pub allowed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    pub entries: Vec<AuditLogEntry>,
}

impl SecurityAuditLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, capability: Capability, action: &str, allowed: bool, details: &str) {
        self.entries.push(AuditLogEntry {
            timestamp_millis: self.entries.len() as u64 + 1, // deterministic synthetic clock
            capability,
            action: action.to_string(),
            allowed,
            details: details.to_string(),
        });
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

pub struct SecurityScanner;

impl SecurityScanner {
    pub fn scan_source(source: &str) -> SecurityAuditReport {
        let mut findings = Vec::new();
        let mut caps = HashSet::new();
        let lower = source.to_ascii_lowercase();

        // 1. Text-level fast pattern scanning for dangerous keywords and DLL imports
        if lower.contains("declare ") && (lower.contains(" lib ") || lower.contains(" ptrsafe ")) {
            findings.push(SecurityFinding {
                severity: Severity::Critical,
                capability: Some(Capability::SystemCommand),
                message: "Native Win32 DLL import via 'Declare Function/Sub Lib' detected"
                    .to_string(),
                location: None,
            });
            caps.insert(Capability::SystemCommand);
        }

        if lower.contains("wscript.shell") || lower.contains("shell.application") {
            findings.push(SecurityFinding {
                severity: Severity::Critical,
                capability: Some(Capability::SystemCommand),
                message: "Invocation of Windows OS shell automation COM object (WScript.Shell / Shell.Application)".to_string(),
                location: None,
            });
            caps.insert(Capability::SystemCommand);
        }

        if lower.contains("msxml2.xmlhttp")
            || lower.contains("winhttp.winhttprequest")
            || lower.contains("msxml2.serverxmlhttp")
        {
            findings.push(SecurityFinding {
                severity: Severity::High,
                capability: Some(Capability::Network),
                message: "HTTP network request COM component detected".to_string(),
                location: None,
            });
            caps.insert(Capability::Network);
        }

        if lower.contains("scripting.filesystemobject") || lower.contains("adodb.stream") {
            findings.push(SecurityFinding {
                severity: Severity::High,
                capability: Some(Capability::FileIo),
                message: "FileSystemObject or Stream local disk file access detected".to_string(),
                location: None,
            });
            caps.insert(Capability::FileIo);
        }

        if lower.contains("environ(") || lower.contains("environ$(") {
            findings.push(SecurityFinding {
                severity: Severity::Medium,
                capability: Some(Capability::Environment),
                message: "System environment variable access via 'Environ' detected".to_string(),
                location: None,
            });
            caps.insert(Capability::Environment);
        }

        let max_sev = findings
            .iter()
            .map(|f| f.severity)
            .max()
            .unwrap_or(Severity::Info);
        let risk_level = match max_sev {
            Severity::Critical => RiskLevel::Critical,
            Severity::High => RiskLevel::High,
            Severity::Medium => RiskLevel::Medium,
            Severity::Low => RiskLevel::Low,
            Severity::Info => RiskLevel::Safe,
        };

        let mut required_capabilities: Vec<Capability> = caps.into_iter().collect();
        required_capabilities.sort_by_key(|c| *c as u8);

        SecurityAuditReport {
            risk_level,
            findings,
            required_capabilities,
        }
    }

    pub fn scan_program(program: &VbaProgram) -> SecurityAuditReport {
        let mut findings = Vec::new();
        let mut caps = HashSet::new();

        for routine in &program.routines {
            for stmt in &routine.body {
                Self::inspect_statement(stmt, &routine.name, &mut findings, &mut caps);
            }
        }

        for stmt in &program.top_level_statements {
            Self::inspect_statement(stmt, "TopLevel", &mut findings, &mut caps);
        }

        let max_sev = findings
            .iter()
            .map(|f| f.severity)
            .max()
            .unwrap_or(Severity::Info);
        let risk_level = match max_sev {
            Severity::Critical => RiskLevel::Critical,
            Severity::High => RiskLevel::High,
            Severity::Medium => RiskLevel::Medium,
            Severity::Low => RiskLevel::Low,
            Severity::Info => RiskLevel::Safe,
        };

        let mut required_capabilities: Vec<Capability> = caps.into_iter().collect();
        required_capabilities.sort_by_key(|c| *c as u8);

        SecurityAuditReport {
            risk_level,
            findings,
            required_capabilities,
        }
    }

    fn inspect_statement(
        stmt: &VbaStatement,
        location: &str,
        findings: &mut Vec<SecurityFinding>,
        caps: &mut HashSet<Capability>,
    ) {
        match stmt {
            VbaStatement::Call { target, args } => {
                Self::inspect_expr(target, location, findings, caps);
                for a in args {
                    Self::inspect_expr(a, location, findings, caps);
                }
            }
            VbaStatement::Assign { target, value } | VbaStatement::Set { target, value } => {
                Self::inspect_expr(target, location, findings, caps);
                Self::inspect_expr(value, location, findings, caps);
            }
            VbaStatement::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                Self::inspect_expr(condition, location, findings, caps);
                for s in then_branch {
                    Self::inspect_statement(s, location, findings, caps);
                }
                for (cond, branch) in else_ifs {
                    Self::inspect_expr(cond, location, findings, caps);
                    for s in branch {
                        Self::inspect_statement(s, location, findings, caps);
                    }
                }
                if let Some(eb) = else_branch {
                    for s in eb {
                        Self::inspect_statement(s, location, findings, caps);
                    }
                }
            }
            VbaStatement::For {
                start,
                end,
                step,
                body,
                ..
            } => {
                Self::inspect_expr(start, location, findings, caps);
                Self::inspect_expr(end, location, findings, caps);
                if let Some(st) = step {
                    Self::inspect_expr(st, location, findings, caps);
                }
                for s in body {
                    Self::inspect_statement(s, location, findings, caps);
                }
            }
            VbaStatement::DoLoop {
                condition, body, ..
            } => {
                if let Some(c) = condition {
                    Self::inspect_expr(c, location, findings, caps);
                }
                for s in body {
                    Self::inspect_statement(s, location, findings, caps);
                }
            }
            VbaStatement::WhileWend { condition, body } => {
                Self::inspect_expr(condition, location, findings, caps);
                for s in body {
                    Self::inspect_statement(s, location, findings, caps);
                }
            }
            VbaStatement::With { object, body } => {
                Self::inspect_expr(object, location, findings, caps);
                for s in body {
                    Self::inspect_statement(s, location, findings, caps);
                }
            }
            VbaStatement::Expr(e) => {
                Self::inspect_expr(e, location, findings, caps);
            }
            _ => {}
        }
    }

    fn inspect_expr(
        expr: &VbaExpr,
        location: &str,
        findings: &mut Vec<SecurityFinding>,
        caps: &mut HashSet<Capability>,
    ) {
        match expr {
            VbaExpr::Variable(name) => {
                if name.eq_ignore_ascii_case("shell") {
                    findings.push(SecurityFinding {
                        severity: Severity::Critical,
                        capability: Some(Capability::SystemCommand),
                        message: "Direct Shell execution routine referenced".to_string(),
                        location: Some(location.to_string()),
                    });
                    caps.insert(Capability::SystemCommand);
                }
            }
            VbaExpr::IndexOrCall { callee, args } => {
                if let VbaExpr::Variable(name) = callee.as_ref() {
                    let lower = name.to_ascii_lowercase();
                    if lower == "shell" {
                        findings.push(SecurityFinding {
                            severity: Severity::Critical,
                            capability: Some(Capability::SystemCommand),
                            message: "Direct invocation of 'Shell' system execution".to_string(),
                            location: Some(location.to_string()),
                        });
                        caps.insert(Capability::SystemCommand);
                    } else if lower == "environ" {
                        findings.push(SecurityFinding {
                            severity: Severity::Medium,
                            capability: Some(Capability::Environment),
                            message: "Reading system environment variable via 'Environ'"
                                .to_string(),
                            location: Some(location.to_string()),
                        });
                        caps.insert(Capability::Environment);
                    } else if lower == "createobject" {
                        if let Some(VbaExpr::Literal(VbaLiteral::Str(prog_id))) = args.first() {
                            let p_lower = prog_id.to_ascii_lowercase();
                            if p_lower.contains("shell") {
                                findings.push(SecurityFinding {
                                    severity: Severity::Critical,
                                    capability: Some(Capability::SystemCommand),
                                    message: format!(
                                        "CreateObject with dangerous shell automation '{prog_id}'"
                                    ),
                                    location: Some(location.to_string()),
                                });
                                caps.insert(Capability::SystemCommand);
                            } else if p_lower.contains("http") || p_lower.contains("xmlhttp") {
                                findings.push(SecurityFinding {
                                    severity: Severity::High,
                                    capability: Some(Capability::Network),
                                    message: format!(
                                        "CreateObject with network client '{prog_id}'"
                                    ),
                                    location: Some(location.to_string()),
                                });
                                caps.insert(Capability::Network);
                            } else if p_lower.contains("filesystemobject")
                                || p_lower.contains("stream")
                            {
                                findings.push(SecurityFinding {
                                    severity: Severity::High,
                                    capability: Some(Capability::FileIo),
                                    message: format!(
                                        "CreateObject with file I/O target '{prog_id}'"
                                    ),
                                    location: Some(location.to_string()),
                                });
                                caps.insert(Capability::FileIo);
                            }
                        }
                    }
                }

                Self::inspect_expr(callee, location, findings, caps);
                for a in args {
                    Self::inspect_expr(a, location, findings, caps);
                }
            }
            VbaExpr::BinaryOp { left, right, .. } => {
                Self::inspect_expr(left, location, findings, caps);
                Self::inspect_expr(right, location, findings, caps);
            }
            VbaExpr::UnaryOp { expr, .. } => {
                Self::inspect_expr(expr, location, findings, caps);
            }
            VbaExpr::MemberAccess { object, .. } => {
                Self::inspect_expr(object, location, findings, caps);
            }
            _ => {}
        }
    }
}

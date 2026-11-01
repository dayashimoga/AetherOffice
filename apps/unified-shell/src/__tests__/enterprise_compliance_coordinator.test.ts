import { describe, it, expect, beforeEach } from "vitest";
import {
  EnterpriseComplianceCoordinator,
  type DlpPolicyDto,
} from "../state/EnterpriseComplianceCoordinator";

describe("EnterpriseComplianceCoordinator (Sub-Phase 53.1.1)", () => {
  let coordinator: EnterpriseComplianceCoordinator;

  beforeEach(async () => {
    coordinator = new EnterpriseComplianceCoordinator();
    await coordinator.init();
  });

  it("1. should initialize with default enterprise policy and verified audit log", () => {
    const state = coordinator.getState();
    expect(state.policy.id).toBe("ent-dlp-std");
    expect(state.policy.name).toBe("Standard Enterprise Governance");
    expect(state.isIntegrityValid).toBe(true);
    expect(state.totalVerifiedAuditEntries).toBeGreaterThanOrEqual(0);
    expect(state.errorMessage).toBeNull();
  });

  it("2. should evaluate clean text with zero violations and allow action", async () => {
    const text = "Q4 financial review demonstrates strong operating margins across all products.";
    const result = await coordinator.evaluateContent(text);

    expect(result.is_violation).toBe(false);
    expect(result.action).toBe("allow");
    expect(result.highest_severity).toBeNull();
    expect(result.matches.length).toBe(0);
  });

  it("3. should evaluate text containing credit card and trigger critical block action", async () => {
    const text = "Payment card on file: 4532 0150 0000 0007.";
    const result = await coordinator.evaluateContent(text);

    expect(result.is_violation).toBe(true);
    expect(result.action).toBe("block");
    expect(result.highest_severity).toBe("critical");
    expect(result.triggered_rule_ids).toContain("rule-pci-dss");
    expect(result.matches.length).toBeGreaterThan(0);
  });

  it("4. should evaluate text containing SSN and trigger high redact-and-allow action", async () => {
    const text = "Employee social security record: 123-45-6789.";
    const result = await coordinator.evaluateContent(text);

    expect(result.is_violation).toBe(true);
    expect(result.action).toBe("redact-and-allow");
    expect(result.highest_severity).toBe("high");
    expect(result.triggered_rule_ids).toContain("rule-ssn-redact");
  });

  it("5. should evaluate and redact content in-place with audit logging", async () => {
    const text = "Confidential report for SSN 123-45-6789.";
    const [evalRes, redacted] = await coordinator.evaluateAndRedactContent(text);

    expect(evalRes.is_violation).toBe(true);
    expect(evalRes.action).toBe("redact-and-allow");
    expect(redacted).not.toContain("123-45-6789");
    expect(redacted).toContain("[REDACTED: SSN]");

    const state = coordinator.getState();
    expect(state.lastSanitizedText).toBe(redacted);
  });

  it("6. should update enterprise DLP policy and log policy update event", async () => {
    const customPolicy: DlpPolicyDto = {
      id: "hipaa-phi-custom",
      name: "Custom Healthcare PHI Policy",
      version: 2,
      default_action: "allow",
      rules: [
        {
          id: "rule-phi-block",
          name: "PHI Strict Block",
          description: "Blocks unencrypted patient medical identifiers",
          patterns: [{ kind: "pii", value: "ssn" }],
          threshold: 1,
          severity: "critical",
          action: "block",
        },
      ],
    };

    await coordinator.setPolicy(customPolicy);
    const state = coordinator.getState();
    expect(state.policy.id).toBe("hipaa-phi-custom");
    expect(state.policy.version).toBe(2);

    const latestAudit = state.auditEntries[state.auditEntries.length - 1];
    expect(latestAudit).toBeDefined();
    expect(latestAudit.doc_id).toBe("hipaa-phi-custom");
  });

  it("7. should generate cryptographically verifiable sanitization certificate", async () => {
    const originalText = "Patient 123-45-6789 diagnosis record.";
    const sanitizedText = "Patient [REDACTED: SSN] diagnosis record.";
    const key = "audit-cert-signing-key-2026";

    const cert = await coordinator.generateCertificate(
      "doc-med-404",
      originalText,
      sanitizedText,
      [["SSN", 1]],
      key
    );

    expect(cert.certificate_id).toBeDefined();
    expect(cert.doc_id).toBe("doc-med-404");
    expect(cert.original_digest).toBeDefined();
    expect(cert.sanitized_digest).toBeDefined();
    expect(cert.signature).toBeDefined();

    const state = coordinator.getState();
    expect(state.certificates.has(cert.certificate_id)).toBe(true);
  });

  it("8. should verify sanitization certificate authenticity and document integrity", async () => {
    const originalText = "Financial memo with sensitive account info.";
    const sanitizedText = "Financial memo with [REDACTED] account info.";
    const key = "sec-key-audit";

    const cert = await coordinator.generateCertificate(
      "doc-fin-777",
      originalText,
      sanitizedText,
      [["ACCOUNT", 1]],
      key
    );

    const isValid = await coordinator.verifyCertificate(cert, sanitizedText, key);
    expect(isValid).toBe(true);
  });

  it("9. should append audit log entries into cryptographic hash chain", async () => {
    const initialCount = coordinator.getState().auditEntries.length;

    const entry = await coordinator.logAuditEvent(
      { type: "export-executed" },
      "analyst-bob",
      "doc-fin-777",
      "Exported encrypted PDF with watermarks"
    );

    expect(entry.sequence_number).toBe(initialCount + 1);
    expect(entry.user_id).toBe("analyst-bob");
    expect(entry.doc_id).toBe("doc-fin-777");
    expect(entry.prev_hash).toBeDefined();
    expect(entry.entry_hash).toBeDefined();

    const state = coordinator.getState();
    expect(state.auditEntries.length).toBe(initialCount + 1);
  });

  it("10. should verify unbroken hash-chain integrity of audit log", async () => {
    // Append multiple entries
    await coordinator.logAuditEvent(
      { type: "document-opened" },
      "user-1",
      "doc-a",
      "Opened document"
    );
    await coordinator.logAuditEvent(
      { type: "document-saved" },
      "user-1",
      "doc-a",
      "Saved document version 1"
    );

    const isChainValid = await coordinator.verifyAuditIntegrity();
    expect(isChainValid).toBe(true);

    const state = coordinator.getState();
    expect(state.isIntegrityValid).toBe(true);
    expect(state.totalVerifiedAuditEntries).toBe(state.auditEntries.length);
  });
});


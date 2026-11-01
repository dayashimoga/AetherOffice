/**
 * EnterpriseComplianceCoordinator.ts
 * GEC-v7.0 Stage 53 (Sub-Phase 53.1.1)
 *
 * Presentation Shell Coordinator for:
 * - Enterprise Data Loss Prevention (DLP) policy evaluation and pre-export gating
 * - PII detection, automated redaction, and cryptographic sanitization certificates
 * - Tamper-evident hash-chained audit logging and cryptographic integrity verification
 */

import {
  IPC,
  type AuditEventTypeDto,
  type AuditLogEntryDto,
  type DlpAction,
  type DlpEvaluationResultDto,
  type DlpMatchDto,
  type DlpPolicyDto,
  type DlpSeverity,
  type SanitizationCertificateDto,
} from "../ipc";

export {
  type AuditEventTypeDto,
  type AuditLogEntryDto,
  type DlpAction,
  type DlpEvaluationResultDto,
  type DlpMatchDto,
  type DlpPolicyDto,
  type DlpSeverity,
  type SanitizationCertificateDto,
};

export interface EnterpriseComplianceState {
  policy: DlpPolicyDto;
  lastEvaluation: DlpEvaluationResultDto | null;
  lastSanitizedText: string | null;
  certificates: Map<string, SanitizationCertificateDto>;
  auditEntries: AuditLogEntryDto[];
  isIntegrityValid: boolean;
  totalVerifiedAuditEntries: number;
  errorMessage: string | null;
}

export type ComplianceListener = (
  state: Readonly<EnterpriseComplianceState>
) => void;

function stringToBytes(s: string): Uint8Array {
  return new TextEncoder().encode(s);
}

export class EnterpriseComplianceCoordinator {
  private state: EnterpriseComplianceState;
  private listeners = new Set<ComplianceListener>();

  constructor() {
    this.state = {
      policy: {
        id: "ent-dlp-std",
        name: "Standard Enterprise Governance",
        version: 1,
        default_action: "allow",
        rules: [],
      },
      lastEvaluation: null,
      lastSanitizedText: null,
      certificates: new Map(),
      auditEntries: [],
      isIntegrityValid: true,
      totalVerifiedAuditEntries: 0,
      errorMessage: null,
    };
  }

  /**
   * Returns an immutable snapshot of current compliance coordinator state.
   */
  public getState(): Readonly<EnterpriseComplianceState> {
    return {
      ...this.state,
      certificates: new Map(this.state.certificates),
      auditEntries: [...this.state.auditEntries],
    };
  }

  /**
   * Subscribes a listener to compliance state mutations.
   * Returns an unsubscription function.
   */
  public subscribe(listener: ComplianceListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const snapshot = this.getState();
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error("EnterpriseComplianceCoordinator listener error:", err);
      }
    }
  }

  /**
   * Initializes coordinator by fetching active DLP policy and audit log state.
   */
  public async init(): Promise<void> {
    try {
      const policy = await IPC.dlpGetPolicy();
      this.state.policy = policy;
      const entries = await IPC.auditLogGetEntries();
      this.state.auditEntries = entries;
      const verified = await IPC.auditLogVerifyIntegrity();
      this.state.totalVerifiedAuditEntries = verified;
      this.state.isIntegrityValid = true;
      this.state.errorMessage = null;
      this.notify();
    } catch (err) {
      this.state.errorMessage = `Failed to initialize compliance coordinator: ${String(err)}`;
      this.notify();
    }
  }

  /**
   * Evaluates text content against active enterprise DLP policy.
   */
  public async evaluateContent(text: string): Promise<DlpEvaluationResultDto> {
    try {
      const result = await IPC.dlpEvaluateText(text);
      this.state.lastEvaluation = result;

      if (result.is_violation) {
        await this.logAuditEvent(
          { type: "dlp-violation-detected" },
          "current-user",
          result.policy_id,
          `Triggered rules: ${result.triggered_rule_ids.join(", ")} (Action: ${result.action})`
        );
      }

      this.notify();
      return result;
    } catch (err) {
      this.state.errorMessage = `DLP evaluation failed: ${String(err)}`;
      this.notify();
      throw err;
    }
  }

  /**
   * Evaluates content and performs automated redaction if required by policy.
   */
  public async evaluateAndRedactContent(
    text: string
  ): Promise<[DlpEvaluationResultDto, string]> {
    try {
      const [evalResult, redacted] = await IPC.dlpEvaluateAndRedact(text);
      this.state.lastEvaluation = evalResult;
      this.state.lastSanitizedText = redacted;

      if (evalResult.is_violation) {
        await this.logAuditEvent(
          { type: "document-sanitized" },
          "system-dlp",
          evalResult.policy_id,
          `Redacted ${evalResult.matches.length} sensitive item(s)`
        );
      }

      this.notify();
      return [evalResult, redacted];
    } catch (err) {
      this.state.errorMessage = `DLP evaluate & redact failed: ${String(err)}`;
      this.notify();
      throw err;
    }
  }

  /**
   * Updates active DLP governance policy.
   */
  public async setPolicy(policy: DlpPolicyDto): Promise<void> {
    try {
      await IPC.dlpSetPolicy(policy);
      this.state.policy = policy;
      await this.logAuditEvent(
        { type: "custom", payload: "DLP_POLICY_UPDATED" },
        "security-admin",
        policy.id,
        `Updated policy to version ${policy.version}`
      );
      this.notify();
    } catch (err) {
      this.state.errorMessage = `Failed to set DLP policy: ${String(err)}`;
      this.notify();
      throw err;
    }
  }

  /**
   * Generates a cryptographically signed sanitization certificate.
   */
  public async generateCertificate(
    docId: string,
    originalText: string,
    sanitizedText: string,
    redactions: Array<[string, number]>,
    secretKey: string | Uint8Array
  ): Promise<SanitizationCertificateDto> {
    try {
      const origBytes = stringToBytes(originalText);
      const sanBytes = stringToBytes(sanitizedText);
      const keyBytes =
        typeof secretKey === "string" ? stringToBytes(secretKey) : secretKey;
      const ts = Date.now();

      const cert = await IPC.sanitizationGenerateCertificate(
        docId,
        origBytes,
        sanBytes,
        this.state.policy.id,
        ts,
        redactions,
        keyBytes
      );

      this.state.certificates.set(cert.certificate_id, cert);

      await this.logAuditEvent(
        { type: "signature-applied" },
        "system-sanitizer",
        docId,
        `Generated Sanitization Certificate ${cert.certificate_id}`
      );

      this.notify();
      return cert;
    } catch (err) {
      this.state.errorMessage = `Failed to generate sanitization certificate: ${String(err)}`;
      this.notify();
      throw err;
    }
  }

  /**
   * Verifies certificate signature and optional post-sanitization document integrity.
   */
  public async verifyCertificate(
    cert: SanitizationCertificateDto,
    sanitizedText?: string,
    secretKey?: string | Uint8Array
  ): Promise<boolean> {
    try {
      const sanBytes = sanitizedText ? stringToBytes(sanitizedText) : null;
      const keyBytes =
        typeof secretKey === "string"
          ? stringToBytes(secretKey)
          : secretKey ?? undefined;

      return await IPC.sanitizationVerifyCertificate(cert, sanBytes, keyBytes);
    } catch (err) {
      this.state.errorMessage = `Certificate verification failed: ${String(err)}`;
      this.notify();
      return false;
    }
  }

  /**
   * Records an audit log event into the append-only cryptographic hash chain.
   */
  public async logAuditEvent(
    eventType: AuditEventTypeDto | string,
    userId: string,
    docId: string,
    details: string
  ): Promise<AuditLogEntryDto> {
    try {
      const entry = await IPC.auditLogAppend(
        Date.now(),
        eventType,
        userId,
        docId,
        details
      );
      this.state.auditEntries.push(entry);
      this.state.totalVerifiedAuditEntries =
        await IPC.auditLogVerifyIntegrity();
      this.state.isIntegrityValid = true;
      this.notify();
      return entry;
    } catch (err) {
      this.state.errorMessage = `Audit log append failed: ${String(err)}`;
      this.notify();
      throw err;
    }
  }

  /**
   * Verifies complete hash-chain integrity of the audit log.
   */
  public async verifyAuditIntegrity(): Promise<boolean> {
    try {
      const count = await IPC.auditLogVerifyIntegrity();
      this.state.totalVerifiedAuditEntries = count;
      this.state.isIntegrityValid = true;
      this.notify();
      return true;
    } catch (err) {
      this.state.isIntegrityValid = false;
      this.state.errorMessage = `Audit log integrity broken: ${String(err)}`;
      this.notify();
      return false;
    }
  }
}

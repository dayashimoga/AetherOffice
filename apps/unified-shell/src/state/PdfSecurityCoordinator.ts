/**
 * PdfSecurityCoordinator.ts
 * GEC-v7.0 Stage 49 (Sub-Phase 49.2.2)
 *
 * Industrial-grade presentation shell coordinator for:
 * - Cryptographic PAdES-LTV digital signatures (ETSI EN 319 142)
 * - Signature validation, trust chain discovery, and LTV / DSS verification
 * - Irreversible redaction stream purging with forensic sanitization verification
 * - ISO 19005-2 PDF/A-2b compliant archive export
 * - Reactive state synchronization with PDF Studio security and signing inspector
 */

import {
  IPC,
  type PdfaExportOptions,
  type PdfRedactionReportDto,
  type PdfRedactionTarget,
  type PdfSignatureValidationDto,
  type PdfSigningOptions,
} from "../ipc";

export {
  type PdfaExportOptions,
  type PdfRedactionReportDto,
  type PdfRedactionTarget,
  type PdfSignatureValidationDto,
  type PdfSigningOptions,
};

export interface PdfSecurityState {
  activeBytes: Uint8Array | null;
  signatures: PdfSignatureValidationDto[];
  pendingRedactions: PdfRedactionTarget[];
  lastRedactionReport: PdfRedactionReportDto | null;
  signingOptions: PdfSigningOptions;
  pdfaOptions: PdfaExportOptions;
  isLoading: boolean;
  error: string | null;
  lastSignedBytes: Uint8Array | null;
  lastRedactedBytes: Uint8Array | null;
  lastPdfaBytes: Uint8Array | null;
}

export type PdfSecurityListener = (state: Readonly<PdfSecurityState>) => void;

export class PdfSecurityCoordinator {
  private state: PdfSecurityState = {
    activeBytes: null,
    signatures: [],
    pendingRedactions: [],
    lastRedactionReport: null,
    signingOptions: {
      signer_name: "AetherOffice Verified Signer",
      reason: "Certified Document Approval",
      location: null,
      contact_info: null,
      add_ltv_dss: true,
      signing_time: null,
    },
    pdfaOptions: {
      conformance: "2b",
      title: "Archived Document",
      author: "AetherOffice PDF Studio",
      output_condition: "sRGB IEC61966-2.1",
    },
    isLoading: false,
    error: null,
    lastSignedBytes: null,
    lastRedactedBytes: null,
    lastPdfaBytes: null,
  };

  private listeners: Set<PdfSecurityListener> = new Set();

  constructor(initialBytes?: Uint8Array) {
    if (initialBytes) {
      this.state.activeBytes = initialBytes;
    }
  }

  public subscribe(listener: PdfSecurityListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private emit(): void {
    const currentState = this.getState();
    this.listeners.forEach((listener) => {
      try {
        listener(currentState);
      } catch (err) {
        console.error("PdfSecurityCoordinator listener error:", err);
      }
    });
  }

  public getState(): Readonly<PdfSecurityState> {
    return {
      ...this.state,
      signatures: [...this.state.signatures],
      pendingRedactions: [...this.state.pendingRedactions],
      signingOptions: { ...this.state.signingOptions },
      pdfaOptions: { ...this.state.pdfaOptions },
    };
  }

  public setActiveBytes(bytes: Uint8Array | null): void {
    this.state.activeBytes = bytes;
    this.state.error = null;
    this.emit();
  }

  public setSigningOptions(options: Partial<PdfSigningOptions>): void {
    this.state.signingOptions = { ...this.state.signingOptions, ...options };
    this.emit();
  }

  public setPdfaOptions(options: Partial<PdfaExportOptions>): void {
    this.state.pdfaOptions = { ...this.state.pdfaOptions, ...options };
    this.emit();
  }

  public addRedaction(target: PdfRedactionTarget): void {
    this.state.pendingRedactions.push(target);
    this.emit();
  }

  public removeRedaction(index: number): void {
    if (index >= 0 && index < this.state.pendingRedactions.length) {
      this.state.pendingRedactions.splice(index, 1);
      this.emit();
    }
  }

  public clearRedactions(): void {
    this.state.pendingRedactions = [];
    this.emit();
  }

  /**
   * Cryptographically signs the active document with PAdES-LTV.
   */
  public async signDocument(
    certPem: string,
    privateKeyPem: string,
    options?: Partial<PdfSigningOptions>
  ): Promise<Uint8Array> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      const err = new Error("No active PDF document loaded to sign");
      this.state.error = err.message;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }

    const effectiveOpts: PdfSigningOptions = {
      ...this.state.signingOptions,
      ...(options || {}),
    };

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const signed = await IPC.pdfSignPades(bytes, certPem, privateKeyPem, effectiveOpts);
      this.state.lastSignedBytes = signed;
      this.state.isLoading = false;
      this.emit();
      return signed;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Validates digital signatures present in the active document.
   */
  public async validateSignatures(): Promise<PdfSignatureValidationDto[]> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      const err = new Error("No active PDF document loaded to validate");
      this.state.error = err.message;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const sigs = await IPC.pdfValidateSignatures(bytes);
      this.state.signatures = sigs;
      this.state.isLoading = false;
      this.emit();
      return sigs;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Irreversibly purges pending redactions from the document content streams.
   */
  public async applyRedactions(): Promise<[Uint8Array, PdfRedactionReportDto]> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      const err = new Error("No active PDF document loaded for redaction");
      this.state.error = err.message;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }

    if (this.state.pendingRedactions.length === 0) {
      const err = new Error("No pending redaction targets specified");
      this.state.error = err.message;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const [redactedBytes, report] = await IPC.pdfApplyRedactions(
        bytes,
        this.state.pendingRedactions
      );
      this.state.lastRedactedBytes = redactedBytes;
      this.state.lastRedactionReport = report;
      this.state.pendingRedactions = [];
      this.state.isLoading = false;
      this.emit();
      return [redactedBytes, report];
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Exports the active document as ISO 19005-2 PDF/A-2b archive format.
   */
  public async exportPdfa(options?: Partial<PdfaExportOptions>): Promise<Uint8Array> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      const err = new Error("No active PDF document loaded to export as PDF/A");
      this.state.error = err.message;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }

    const effectiveOpts: PdfaExportOptions = {
      ...this.state.pdfaOptions,
      ...(options || {}),
    };

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const pdfaBytes = await IPC.pdfExportPdfa(bytes, effectiveOpts);
      this.state.lastPdfaBytes = pdfaBytes;
      this.state.isLoading = false;
      this.emit();
      return pdfaBytes;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }
}


import { describe, it, expect, beforeEach } from 'vitest';
import { PdfSecurityCoordinator } from '../state/PdfSecurityCoordinator';

describe('PdfSecurityCoordinator - PAdES-LTV, Redaction & PDF/A Export (Sub-Phase 49.2.2)', () => {
  let coordinator: PdfSecurityCoordinator;
  const dummyPdf = new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x37]);
  const dummyCertPem = '-----BEGIN CERTIFICATE-----\nMIIB...\n-----END CERTIFICATE-----';
  const dummyKeyPem = '-----BEGIN PRIVATE KEY-----\nMIIE...\n-----END PRIVATE KEY-----';

  beforeEach(() => {
    coordinator = new PdfSecurityCoordinator(dummyPdf);
  });

  it('1. Initializes cleanly with default signing options, PDF/A options, and empty queues', () => {
    const state = coordinator.getState();
    expect(state.activeBytes).toBe(dummyPdf);
    expect(state.signatures).toEqual([]);
    expect(state.pendingRedactions).toEqual([]);
    expect(state.lastRedactionReport).toBeNull();
    expect(state.signingOptions.add_ltv_dss).toBe(true);
    expect(state.pdfaOptions.conformance).toBe('2b');
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('2. Signs document with PAdES-LTV and records signed bytes', async () => {
    const signed = await coordinator.signDocument(dummyCertPem, dummyKeyPem, {
      signer_name: 'Verified Legal Signer',
    });

    expect(signed).toBeInstanceOf(Uint8Array);
    const state = coordinator.getState();
    expect(state.lastSignedBytes).toBe(signed);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('3. Validates digital signatures present in document', async () => {
    const sigs = await coordinator.validateSignatures();

    expect(Array.isArray(sigs)).toBe(true);
    expect(sigs.length).toBeGreaterThanOrEqual(1);
    expect(sigs[0].is_valid).toBe(true);
    expect(sigs[0].has_ltv_dss).toBe(true);

    const state = coordinator.getState();
    expect(state.signatures).toEqual(sigs);
    expect(state.isLoading).toBe(false);
  });

  it('4. Adds and removes pending redactions', () => {
    coordinator.addRedaction({
      page_index: 0,
      text_pattern: 'CONFIDENTIAL',
    });

    expect(coordinator.getState().pendingRedactions.length).toBe(1);

    coordinator.removeRedaction(0);
    expect(coordinator.getState().pendingRedactions.length).toBe(0);
  });

  it('5. Clears all pending redactions', () => {
    coordinator.addRedaction({ page_index: 0, text_pattern: 'A' });
    coordinator.addRedaction({ page_index: 0, text_pattern: 'B' });
    expect(coordinator.getState().pendingRedactions.length).toBe(2);

    coordinator.clearRedactions();
    expect(coordinator.getState().pendingRedactions.length).toBe(0);
  });

  it('6. Applies pending redactions and stores forensic report', async () => {
    coordinator.addRedaction({
      page_index: 0,
      text_pattern: 'TOP_SECRET_CODE',
    });

    const [redactedBytes, report] = await coordinator.applyRedactions();

    expect(redactedBytes).toBeInstanceOf(Uint8Array);
    expect(report.forensically_sanitized).toBe(true);
    expect(report.total_redactions_applied).toBeGreaterThanOrEqual(1);

    const state = coordinator.getState();
    expect(state.lastRedactedBytes).toBe(redactedBytes);
    expect(state.lastRedactionReport).toEqual(report);
    expect(state.pendingRedactions).toEqual([]);
    expect(state.isLoading).toBe(false);
  });

  it('7. Rejects redaction application when no targets are pending', async () => {
    coordinator.clearRedactions();
    await expect(coordinator.applyRedactions()).rejects.toThrow('No pending redaction targets');

    expect(coordinator.getState().error).toContain('No pending redaction targets');
  });

  it('8. Exports document as compliant PDF/A-2b archive', async () => {
    const pdfa = await coordinator.exportPdfa({
      title: 'Permanent Archive 2026',
    });

    expect(pdfa).toBeInstanceOf(Uint8Array);
    const state = coordinator.getState();
    expect(state.lastPdfaBytes).toBe(pdfa);
    expect(state.isLoading).toBe(false);
  });

  it('9. Updates signing options and PDF/A options', () => {
    coordinator.setSigningOptions({
      signer_name: 'Compliance Auditor',
      location: 'Zurich, Switzerland',
    });
    coordinator.setPdfaOptions({
      title: 'Audited Statement',
      conformance: '2b',
    });

    const state = coordinator.getState();
    expect(state.signingOptions.signer_name).toBe('Compliance Auditor');
    expect(state.signingOptions.location).toBe('Zurich, Switzerland');
    expect(state.pdfaOptions.title).toBe('Audited Statement');
  });

  it('10. Rejects signing when document bytes are null and captures error in state', async () => {
    coordinator.setActiveBytes(null);
    await expect(
      coordinator.signDocument(dummyCertPem, dummyKeyPem)
    ).rejects.toThrow('No active PDF document loaded to sign');

    const state = coordinator.getState();
    expect(state.error).toContain('No active PDF document loaded to sign');
    expect(state.isLoading).toBe(false);
  });
});


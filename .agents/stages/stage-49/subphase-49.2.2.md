# Sub-Phase 49.2.2: Cryptographic PAdES-LTV Signing, Redaction Stream Purging & PDF/A-2b Export

## Focus
`pdf-engine`, `crypto-security`, `render-core`, `unified-shell`

## Goals
1. Implement pure safe-Rust security and compliance engine in `pdf-engine::security`:
   - Cryptographic PAdES-LTV signing:
     - Integration with `crypto-security` (X.509, PKCS#7 detached signatures, RFC 3161 timestamps).
     - Long-Term Validation (LTV) `/DSS` dictionary injection with `/VRI` records.
     - Signature extraction and validation reports.
   - Irreversible redaction stream purging:
     - Physical removal of sensitive string bytes, coordinate operators, and glyph runs from page content streams.
     - Painting opaque redaction fill rectangles over affected regions.
     - Forensic sanitization verification guaranteeing zero residual traces in streams.
   - PDF/A-2b archive export (ISO 19005-2):
     - Injection of standard compliant XMP metadata packet with `<pdfaid:part>2</pdfaid:part>` and `<pdfaid:conformance>B</pdfaid:conformance>`.
     - OutputIntent dictionary injection with standard sRGB ICC profile identifier (`/GTS_PDFA1`).
     - Removal of forbidden elements (e.g. JavaScript, un-embedded attachments, encryption).
2. Register Tauri IPC commands:
   - `pdf_sign_pades`
   - `pdf_validate_signatures`
   - `pdf_apply_redactions`
   - `pdf_export_pdfa`
3. Build presentation shell coordinator:
   - `PdfSecurityCoordinator.ts`:
     - PAdES signing workflow & signature validation reports.
     - Redaction zone markup & irreversible content stream purging.
     - PDF/A-2b export configuration & verification.
     - Reactive state synchronization with PDF Studio security panels.
4. Comprehensive test coverage:
   - 10 Rust unit/integration tests in `crates/pdf-engine/tests/subphase_49_2_2_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/pdf_security_coordinator.test.ts`.
5. Pass `tools/verify-contract.ps1` with exit code 0.


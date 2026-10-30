# Subphase 17.1.1: PAdES-LTV, X.509 & PKCS#7 Digital Signatures for PDF and OOXML
- **Stage:** 17 | **Phase:** 17.1
- **Target Subsystem:** `crypto-security`
- **Primary Agent Skill:** `@security-architect`
- **Deliverable:** X.509 certificate validation, RFC 3161 cryptographic timestamping, PKCS#7 digital signature generation & verification for PDF ByteRange dictionaries and OOXML digital signature parts (`_xmlsignatures`).
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 17.1.1
Invoke @security-architect under GEC-v2.0:
1. Create new crate `crates/crypto-security` with strict `#![forbid(unsafe_code)]` and zero panic macros.
2. Implement X.509 certificate chain validation and SHA-256 / SHA-384 / SHA-512 digest verification.
3. Implement PKCS#7 detached signature builder and validator conforming to PAdES-LTV (PDF Advanced Electronic Signatures Long-Term Validation) with RFC 3161 timestamping support.
4. Implement OOXML XMLDSIG digital signature verification engine for signed OPC package parts.
5. Add unit and integration tests verifying valid certificates, tampering detection on signed ByteRanges, and signature metadata extraction.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```


# Subphase 17.1.3: Sanitize, Privacy Audit & Forensic Redaction Engine
- **Stage:** 17 | **Phase:** 17.1
- **Target Subsystem:** `crypto-security` & `unified-shell`
- **Primary Agent Skill:** `@security-architect`
- **Deliverable:** Irreversible vector and raster redaction engine, EXIF/XML metadata sanitizer, automated PII detector (regex/patterns), and UI Privacy Inspection Dialog.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 17.1.3
Invoke @security-architect under GEC-v2.0:
1. Implement irreversible document sanitizer stripping docProps metadata, revision history, comments, and hidden slide/sheet parts.
2. Implement PDF true redaction engine: physically removing underlying text operators, vector paths, and raster bitmap regions underneath redaction bounding boxes.
3. Add PII scanner identifying sensitive patterns (emails, credit card numbers, SSNs, phone numbers).
4. Implement `SanitizeModal.ts` in `unified-shell` offering one-click "Inspect & Clean Document" before sharing.
5. Add unit and UI integration tests verifying that redacted text bytes are completely expunged from the file payload and UI inspection functions correctly.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```


# Subphase 17.1.2: AES-256-GCM Envelope Encryption, Zero-Knowledge Protection & Document Permissions
- **Stage:** 17 | **Phase:** 17.1
- **Target Subsystem:** `crypto-security`
- **Primary Agent Skill:** `@security-architect`
- **Deliverable:** Agile Encryption for OOXML with PBKDF2/SHA-512 and AES-256-CBC/GCM, PDF Standard Security Handler (Revision 6 AES-256), fine-grained permission bitmasks.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 17.1.2
Invoke @security-architect under GEC-v2.0:
1. Implement OOXML Agile Encryption engine with key derivation (PBKDF2-HMAC-SHA512) and AES-256 payload encryption.
2. Implement PDF Standard Security Handler Rev 6 (ISO 32000-2 / PDF 2.0) with AES-256 password authentication (user and owner passwords).
3. Implement permission bitmask enforcement (disallowing printing, copying content, form fill, or page extraction).
4. Expose clean APIs for password-protecting and decrypting documents with zero heap leak of plaintext keys.
5. Add unit and integration tests verifying roundtrip encryption/decryption, wrong-password rejection, and permission enforcement.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

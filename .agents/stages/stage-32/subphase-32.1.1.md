# Subphase 32.1.1: PDF Open/Save, Multi-Page Organizer, Page Resize & Password Encrypt/Decrypt
- **Stage:** 32 | **Phase:** 32.1
- **Target Subsystem:** `pdf-engine`
- **Primary Agent Skill:** `@pdf-architect`
- **Deliverable:** 100% working PDF Open/Create/Save, Add/Delete/Duplicate/Rotate/Extract/Split/Merge pages, Resize PDF Page Dimensions, and Password Encrypt/Decrypt.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 32.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v4.0.
1. Fix the completely non-responsive PDF workspace so every single button works:
   - **Open & Render Existing PDFs + Create Blank PDFs:** Clicking Open (in Home or File) opens any .pdf file from disk, parses its pages/text/dimensions via pdf-engine (and browser PDF canvas/object renderer), and renders every page interactively in both the main viewport and the left Pages thumbnail strip!
   - **Page Tools Tab (Full Page Manipulation):**
     - + Insert Blank Page (inserts a new page before/after current page and updates page count).
     - Delete Page (removes selected page, with confirmation if >1 page).
     - Duplicate Page (clones selected page and all its elements).
     - Rotate CW 90Â° & Rotate CCW 90Â° (rotates the page dimensions and content).
     - Move Page Up / Down & drag-and-drop thumbnail reordering.
     - **Resize PDF Page Size (Requested Feature):** Modal & toolbar selector to change page dimensions (A4 210Ã—297mm, Letter 8.5Ã—11in, Legal 8.5Ã—14in, A3, A5, or Custom Width Ã— Height mm/px) with option to Scale Content to Fit New Page Size and apply to Current Page or All Pages!
     - Split PDF (extract page range into a new PDF) & Merge PDF (append another .pdf file's pages into the current document).
   - **Sign & Protect Tab (Password Encryption & Decryption â€” Requested Feature):**
     - **Encrypt with Password:** Opens a security modal allowing the user to set an **Open Password (User Password)** and **Permissions Password (Owner Password â€” restrict printing/editing/copying)**. When saved or reopened, the PDF requires the password to unlock!
     - **Decrypt / Remove Password:** Allows unlocking a password-protected PDF by entering its password and removing encryption restrictions.
     - **Compress / Optimize PDF Size:** Reduces image resolution / strips unused objects and displays the before/after file size in KB.
2. Write automated tests verifying PDF Open, + Insert Page, Delete Page, Rotate, Resize PDF Page Size, and Encrypt / Decrypt Password. Commit with [GEC-Verified].
```

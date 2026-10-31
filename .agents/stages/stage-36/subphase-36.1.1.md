# Subphase 36.1.1: Fix Word Header/Body Duplicate Typing Bug & Independent Header/Body/Footer Zones

- **Stage:** 36 | **Phase:** 36.1
- **Target Subsystem:** `word-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Strictly separated Header, Body, and Footer zones on every Word page; typing goes ONLY into Body by default; double-click to edit Header/Footer with Close button.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
/execute-subphase 36.1.1
Invoke @adaptive-ui-builder under GEC-v6.0 Article I.
1. **Fix the Critical Bug Seen in `image_97c287.png` & `image_97b3a4.png` (Typing Appearing in Header & Body Simultaneously):**
   - Inspect why `asdfasdfadsf...` appeared twice (once in the top header box and once in the body below the shadow):
     - Remove any duplicate rendering of `page.content` or shared state between the header box and the body box!
     - Give each `WordPage` three strictly independent properties:
       - `headerText: string` (default `""`)
       - `bodyHtml: string` (default `""`)
       - `footerText: string` (default `""`)
     - Maintain `editingZone: "body" | "header" | "footer"` (default `"body"`).
2. **Proper Microsoft Word Header / Body / Footer UX:**
   - **Main Page Body (`editingZone === "body"`):** Clicking anywhere on the main white page sheet focuses the **Body** editor (`min-height: 820px; padding: 24px 64px; outline: none;`). Everything typed goes **strictly into `bodyHtml`**—NEVER into the header!
   - **Header Zone (Top of Page) & Footer Zone (Bottom of Page):**
     - When `editingZone === "body"`, the Header and Footer areas are rendered softly (`opacity: 0.65`) at the top and bottom margins of the same unified paper sheet (NOT as a detached separate card with a shadow gap splitting the page in two!).
     - **Double-clicking** the Header or Footer (or clicking `Insert -> Header` / `Insert -> Footer` on the ribbon) sets `editingZone = "header"` (or `"footer"`), focuses that input, and displays a clear **`[✓ Exit Header/Footer (Esc)]`** pill button that switches `editingZone` back to `"body"` and focuses the document body!
3. Write an automated regression test in Vitest that:
   - Types `"Hello Body Text"` into the Word document body and asserts `headerText` is `""` (empty) and `"Hello Body Text"` appears ONLY once on the page (inside `.word-page-body`).
   - Double-clicks the Header zone, types `"Confidential Header"`, clicks `Exit Header/Footer`, and asserts `headerText === "Confidential Header"` while `bodyHtml` remains `"Hello Body Text"`.
   Commit with `[GEC-Verified]`.
````

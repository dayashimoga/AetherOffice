# Subphase 35.1.2: App-Scoped Multi-Instance Workspace Manager & Modern Visual Polish

- **Stage:** 35 | **Phase:** 35.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Clean per-app multi-document tab bar (defaulting to Current App Tabs), quick + New / Open per format, Full-App Dark/Light Theme, and Responsive Desktop/Tablet/Mobile layout.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

````text
/execute-subphase 35.1.2
Invoke @adaptive-ui-builder under GEC-v6.0 Article III.
1. **Fix Multi-Document Organization (Stop Mixing All 5 File Types by Default):**
   - Change the default tab filter mode from `All Open Tabs` to **`Current App Only`** (`showAllFormats = false` by default, with a toggle pill `App Tabs | All Tabs (N)`):
     - When **Word (`W`)** is selected on the Left App Rail, the top tab bar shows **ONLY `.docx` tabs** (`Untitled 1.docx`, `test.docx`, etc.) + a **`+ New .docx`** button and **`📂 Open .docx`** button right next to the tabs!
     - When **Excel (`X`)** is selected, the top tab bar shows **ONLY `.xlsx` tabs** (`Book1.xlsx`, `test.xlsx`) + **`+ New .xlsx`** and **`📂 Open .xlsx`**.
     - When **PowerPoint (`P`)** is selected, shows **ONLY `.pptx` tabs** + **`+ New .pptx`** and **`📂 Open .pptx`**.
     - When **PDF (`PDF`)** is selected, shows **ONLY `.pdf` tabs** + **`+ New .pdf`** and **`📂 Open .pdf`**.
     - When **Notes (`N`)** is selected, shows **ONLY `.txt/.md` tabs** + **`+ New .txt`** and **`📂 Open .txt`**.
   - This makes opening 5 PDFs, 4 Word docs, and 6 Excel sheets completely intuitive, fast, and clutter-free!
2. **Fix Partial Dark / Light Theme Across All Views (Seen in image_9756a4.png):**
   - In `image_9756a4.png`, PDF Studio's left `Pages / Outline / Notes` sidebar and canvas backdrop stayed bright white even when `🌙 Dark` mode was active!
   - Audit every component in `apps/unified-shell/src/` and replace all hardcoded `#ffffff`, `#f8fafc`, `#f1f5f9`, `#e2e8f0` sidebar/panel/backdrop colors with theme-aware CSS variables (`var(--bg-surface)`, `var(--bg-sidebar)`, `var(--bg-canvas-surround)`, `var(--text-primary)`, `var(--border-color)`).
3. **Remove Empty Ribbon Group Stubs (Seen in image_97c287.png):**
   - In `image_97c287.png`, Word's Home ribbon has empty `Styles` and `Copilot` group headers on row 2 with no buttons visible inside them! Ensure every ribbon group renders its buttons visibly inside the group container with zero vertical clipping.
4. Verify with automated DOM tests and commit with `[GEC-Verified]`.
````
